//! Regel-Auswahl und die neuen Waffen: Querschläger, Schlangenkugel, Zeitbombe, Napalm,
//! Laserschranke, Kugelsprinkler, Enterhaken, Selbstzerstörung.

use std::f64::consts::TAU;

use panzer_core::bot::Bot;
use panzer_core::codec::{read_snap, write_snap, Reader, Writer};
use panzer_core::consts::*;
use panzer_core::geom::{circle_hits_walls, hypot};
use panzer_core::items::{Eff, Item, Kind};
use panzer_core::physics::{fence_ends, first_wall};
use panzer_core::rng::Rng;
use panzer_core::snap::NetSnap;
use panzer_core::text::kill_text;
use panzer_core::Game;

/// Spiel ohne Spawn-Schutz, mit zufälligen Kisten abgeschaltet
fn game(players: usize, seed: u64) -> Game {
    let slots: Vec<usize> = (0..players).collect();
    let mut g = Game::with_rng(&slots, Rng::new(seed));
    for t in g.tanks.iter_mut() {
        t.eff.remove(Eff::Spawn);
    }
    g.box_timer = 1e9;
    g
}

/// Panzer kann nicht sterben (hält die Runde offen, wenn er nur Schütze ist)
fn protect(g: &mut Game, slot: usize) {
    g.tanks[slot].eff.set(Eff::Inv, 1e6);
}

fn run(g: &mut Game, seconds: f64) {
    for _ in 0..(seconds / TICK).round() as usize {
        g.step(TICK);
    }
}

/// Waffe geben und einmal abdrücken
fn fire(g: &mut Game, slot: usize, item: Item) {
    let t = &mut g.tanks[slot];
    t.weapon = Some(item);
    t.ammo = item.ammo();
    t.cooldown = 0.0;
    let seen = t.fire_seen.unwrap_or(0);
    g.apply_input(slot, 0, seen, TICK); // der erste Input merkt sich nur den Zähler
    g.apply_input(slot, 0, seen.wrapping_add(1), TICK);
}

fn count(g: &Game, kind: Kind) -> usize {
    g.shots.iter().filter(|s| s.kind == kind && !s.dead).count()
}

fn shot(g: &Game, kind: Kind) -> &panzer_core::game::Shot {
    g.shots.iter().find(|s| s.kind == kind).expect("Geschoss fehlt")
}

/// Winkel, in dem von Panzer slot aus die nächste Wand zwischen min und max entfernt ist
fn free_angle(g: &Game, slot: usize, min: f64, max: f64) -> f64 {
    let t = &g.tanks[slot];
    (0..360)
        .map(|d| d as f64 * TAU / 360.0)
        .find(|&a| (min..max).contains(&first_wall(&g.arena, t.x, t.y, a, 2000.0)))
        .expect("kein passender Winkel im Labyrinth")
}

#[test]
fn rule_only_gilt_fuer_jede_runde() {
    let mut g = Game::with_rule(&[0, 1], RoundRule::Only(Mutator::Kisten));
    for _ in 0..30 {
        assert_eq!(g.mutator, Some(Mutator::Kisten));
        assert_eq!(g.arena.mutator, Some(Mutator::Kisten));
        g.new_round();
    }
}

#[test]
fn rule_off_hat_nie_eine_sonderregel() {
    let mut g = Game::with_rule(&[0, 1], RoundRule::Off);
    for _ in 0..200 {
        assert_eq!(g.mutator, None);
        g.new_round();
    }
}

#[test]
fn rule_random_wie_bisher() {
    let mut g = game(2, 11);
    let (mut with, mut without, mut last) = (0, 0, None);
    for _ in 0..500 {
        match g.mutator {
            Some(m) => {
                with += 1;
                assert_ne!(last, Some(m), "nie zweimal dieselbe hintereinander");
            }
            None => without += 1,
        }
        last = g.mutator;
        g.new_round();
    }
    assert!(with > 100 && without > 200, "{with} mit, {without} ohne");
}

#[test]
fn rule_key_uebersteht_speichern() {
    for r in RoundRule::all() {
        assert_eq!(RoundRule::from_key(r.key()), r);
        assert!(!r.name().is_empty() && !r.desc().is_empty());
    }
    assert_eq!(RoundRule::all().len(), MUTATORS.len() + 2);
    assert_eq!(RoundRule::from_key("gibtsnicht"), RoundRule::Random);
}

#[test]
fn querschlaeger_teilt_sich_und_bleibt_begrenzt() {
    let mut g = game(2, 3);
    protect(&mut g, 0);
    protect(&mut g, 1);
    fire(&mut g, 0, Item::Split);
    assert_eq!(count(&g, Kind::Split), 1);
    let mut most = 0;
    for _ in 0..(SPLIT_LIFE / TICK) as usize {
        g.step(TICK);
        most = most.max(count(&g, Kind::Split));
    }
    assert!(most >= 4, "sollte sich mehrfach teilen, höchstens {most}");
    assert!(most <= 1 << SPLIT_GENS, "zu viele Kugeln: {most}");
    run(&mut g, 0.2);
    assert_eq!(count(&g, Kind::Split), 0, "alle Kopien leben nur so lange wie das Original");
}

#[test]
fn schlangenkugel_pendelt_bei_gleichem_tempo() {
    let mut g = game(2, 5);
    protect(&mut g, 0);
    protect(&mut g, 1);
    let a = free_angle(&g, 0, 200.0, 2000.0);
    g.tanks[0].a = a;
    fire(&mut g, 0, Item::Wave);
    let (mut left, mut right) = (false, false);
    for _ in 0..30 {
        g.step(TICK);
        let s = shot(&g, Kind::Wave);
        assert!((hypot(s.vx, s.vy) - WAVE_SPEED).abs() < 1.0, "Tempo muss gleich bleiben");
        let diff = panzer_core::geom::angle_diff(s.heading(), a);
        left |= diff > 0.15;
        right |= diff < -0.15;
    }
    assert!(left && right, "Kurs sollte in beide Richtungen pendeln");
}

#[test]
fn zeitbombe_trifft_erst_nach_ablauf_und_nur_im_radius() {
    let mut g = game(3, 7);
    protect(&mut g, 0);
    fire(&mut g, 0, Item::Timebomb);
    let (bx, by) = (shot(&g, Kind::Timebomb).x, shot(&g, Kind::Timebomb).y);
    g.tanks[1].x = bx + TIMEBOMB_BLAST - 10.0;
    g.tanks[1].y = by;
    g.tanks[2].x = bx + TIMEBOMB_BLAST + 60.0;
    g.tanks[2].y = by;
    run(&mut g, TIMEBOMB_FUSE - 0.2);
    assert!(g.tanks[1].alive, "vor Ablauf passiert nichts");
    run(&mut g, 0.4);
    assert!(!g.tanks[1].alive, "im Radius");
    assert!(g.tanks[2].alive, "außerhalb des Radius");
    assert_eq!(count(&g, Kind::Timebomb), 0);
}

#[test]
fn napalm_brennt_aber_nicht_durch_waende() {
    let mut g = game(3, 9);
    protect(&mut g, 0);
    // weit genug von der Wand, damit das Geschoss in der Luft platzt
    g.tanks[0].a = free_angle(&g, 0, 60.0, 2000.0);
    fire(&mut g, 0, Item::Napalm);
    assert_eq!(count(&g, Kind::Napalm), 1);
    g.tanks[1].x = -500.0;
    g.tanks[2].x = -500.0;
    run(&mut g, NAPALM_FUSE + 0.1);
    assert_eq!(count(&g, Kind::Napalm), 0, "muss geplatzt sein");
    assert_eq!(count(&g, Kind::Burn), 1, "hinterlässt eine Feuerfläche");
    let (bx, by) = (shot(&g, Kind::Burn).x, shot(&g, Kind::Burn).y);

    // Eine Stelle im Radius, aber hinter einer Wand
    let behind = (0..72).find_map(|k| {
        let a = k as f64 * TAU / 72.0;
        let (x, y) = (bx + a.cos() * NAPALM_R * 0.9, by + a.sin() * NAPALM_R * 0.9);
        (!g.arena.clear(bx, by, x, y)).then_some((x, y))
    });
    if let Some((x, y)) = behind {
        g.tanks[2].x = x;
        g.tanks[2].y = y;
    }
    g.tanks[1].x = bx;
    g.tanks[1].y = by;
    run(&mut g, 0.1);
    assert!(!g.tanks[1].alive, "mitten im Feuer");
    if behind.is_some() {
        assert!(g.tanks[2].alive, "hinter der Wand sicher");
    }
    run(&mut g, NAPALM_BURN);
    assert_eq!(count(&g, Kind::Burn), 0, "brennt nicht ewig");
}

#[test]
fn laserschranke_wird_scharf_und_endet_an_der_wand() {
    let mut g = game(3, 13);
    protect(&mut g, 0);
    fire(&mut g, 0, Item::Fence);
    let (x, y, a) = (shot(&g, Kind::Fence).x, shot(&g, Kind::Fence).y, shot(&g, Kind::Fence).ang);
    let ((x0, y0), (x1, y1)) = fence_ends(&g.arena, x, y, a);
    assert!(hypot(x1 - x0, y1 - y0) > 5.0, "Strahl muss eine Länge haben");

    // Panzer 2 steht hinter dem Ende des Strahls
    g.tanks[2].x = x0 + a.cos() * (TANK_R + 30.0);
    g.tanks[2].y = y0 + a.sin() * (TANK_R + 30.0);
    g.tanks[1].x = (x0 + x1) / 2.0;
    g.tanks[1].y = (y0 + y1) / 2.0;
    run(&mut g, FENCE_ARM - 0.2);
    assert!(g.tanks[1].alive, "noch nicht scharf");
    run(&mut g, 0.4);
    assert!(!g.tanks[1].alive, "scharf: wer den Strahl berührt, ist weg");
    assert!(g.tanks[2].alive, "hinter dem Ende passiert nichts");
    run(&mut g, FENCE_TIME);
    assert_eq!(count(&g, Kind::Fence), 0, "geht wieder aus");
}

#[test]
fn kugelsprinkler_schiesst_erst_spaeter_und_hoert_auf() {
    let mut g = game(2, 17);
    protect(&mut g, 0);
    protect(&mut g, 1);
    fire(&mut g, 0, Item::Sprinkler);
    run(&mut g, SPRINKLER_ARM - 0.2);
    assert_eq!(count(&g, Kind::Spb), 0, "wartet erst");
    let mut most = 0;
    for _ in 0..((SPRINKLER_TIME + 0.4) / TICK) as usize {
        g.step(TICK);
        most = most.max(count(&g, Kind::Spb));
    }
    assert!(most > 8, "schießt dauerhaft, höchstens {most}");
    assert!(most < 60, "aber nicht unbegrenzt: {most}");
    assert_eq!(count(&g, Kind::Sprinkler), 0);
    run(&mut g, SPRINKLER_LIFE + 0.1);
    assert_eq!(count(&g, Kind::Spb), 0);
}

#[test]
fn enterhaken_zieht_zur_wand() {
    let mut g = game(2, 19);
    protect(&mut g, 0);
    let a = free_angle(&g, 0, 120.0, 450.0);
    g.tanks[0].a = a;
    let (x0, y0) = (g.tanks[0].x, g.tanks[0].y);
    let d = first_wall(&g.arena, x0, y0, a, 2000.0);
    fire(&mut g, 0, Item::Hook);
    run(&mut g, 1.6);
    let t = &g.tanks[0];
    let moved = hypot(t.x - x0, t.y - y0);
    assert!(moved > d * 0.5, "nur {moved:.0} von {d:.0} px gezogen");
    assert!(!circle_hits_walls(t.x, t.y, t.radius() - 0.5, g.arena.near(t.x, t.y)), "steckt in der Wand");
    assert_eq!(count(&g, Kind::Hook), 0, "der Haken ist danach weg");
}

#[test]
fn enterhaken_holt_den_gegner_und_lockert_nicht_den_schuetzen() {
    let mut g = game(2, 23);
    protect(&mut g, 0);
    let a = free_angle(&g, 0, 240.0, 2000.0);
    g.tanks[0].a = a;
    let (x0, y0) = (g.tanks[0].x, g.tanks[0].y);
    g.tanks[1].x = x0 + a.cos() * 180.0;
    g.tanks[1].y = y0 + a.sin() * 180.0;
    fire(&mut g, 0, Item::Hook);
    let mut frozen = false;
    for _ in 0..(1.2 / TICK) as usize {
        g.step(TICK);
        frozen |= g.tanks[1].has(Eff::Frozen);
    }
    let d = hypot(g.tanks[1].x - g.tanks[0].x, g.tanks[1].y - g.tanks[0].y);
    assert!(frozen, "der Getroffene kann kurz nichts machen");
    assert!(g.tanks[1].alive, "der Haken selbst tötet nicht");
    assert!(d < 130.0, "sollte herangezogen sein, Abstand {d:.0}");
    assert!(hypot(g.tanks[0].x - x0, g.tanks[0].y - y0) < 1.0, "der Schütze bleibt stehen");
}

#[test]
fn selbstzerstoerung_faehrt_mit_und_nimmt_nachbarn_mit() {
    let mut g = game(3, 29);
    fire(&mut g, 0, Item::Destruct);
    assert_eq!(count(&g, Kind::Destruct), 1);
    let (x, y) = (g.tanks[0].x, g.tanks[0].y);
    g.tanks[1].x = x + DESTRUCT_BLAST - 20.0;
    g.tanks[1].y = y;
    g.tanks[2].x = x + DESTRUCT_BLAST + 80.0;
    g.tanks[2].y = y;
    run(&mut g, DESTRUCT_FUSE - 0.2);
    assert!(g.tanks[0].alive && g.tanks[1].alive, "erst der Countdown");
    let s = shot(&g, Kind::Destruct);
    assert_eq!((s.x, s.y), (g.tanks[0].x, g.tanks[0].y), "sitzt auf dem Panzer");
    run(&mut g, 0.4);
    assert!(!g.tanks[0].alive, "der Schütze selbst");
    assert!(!g.tanks[1].alive, "der Nachbar");
    assert!(g.tanks[2].alive, "außerhalb des Radius");
}

#[test]
fn killfeed_hat_eigene_texte() {
    let name = |s: usize| if s == 0 { "A".to_string() } else { "B".to_string() };
    for k in [Kind::Split, Kind::Wave, Kind::Timebomb, Kind::Burn, Kind::Fence, Kind::Spb, Kind::Destruct] {
        let other = kill_text(Some(0), 1, k, 0, &name);
        assert!(!other.contains("abgeschossen"), "{k:?}: {other}");
        assert!(other.contains('A') && other.contains('B'), "{k:?}: {other}");
        let own = kill_text(Some(1), 1, k, 0, &name);
        assert!(own.contains('B') && !own.contains('{'), "{k:?}: {own}");
        assert!(k.icon().is_some(), "{k:?} braucht ein Symbol");
    }
}

/// Bot-Match nur mit den neuen Waffen: keine Panics, kein Panzer in einer Wand,
/// Delta-Snapshots bleiben verlustfrei.
#[test]
fn bot_match_nur_mit_neuen_waffen() {
    const NEW: [Item; 8] = [
        Item::Split,
        Item::Wave,
        Item::Timebomb,
        Item::Napalm,
        Item::Fence,
        Item::Sprinkler,
        Item::Hook,
        Item::Destruct,
    ];
    // Art -> Geschoss-Kennungen: wie oft Bots die Waffe wirklich abgefeuert haben
    let mut used: std::collections::HashMap<Kind, std::collections::HashSet<(usize, u32)>> = Default::default();
    const KINDS: [Kind; 8] = [
        Kind::Split,
        Kind::Wave,
        Kind::Timebomb,
        Kind::Napalm,
        Kind::Fence,
        Kind::Sprinkler,
        Kind::Hook,
        Kind::Destruct,
    ];
    for players in [2, 4, 8].into_iter().cycle().take(24) {
        if players == 2 && KINDS.iter().all(|k| used.contains_key(k)) {
            break; // alle kamen vor, mehr braucht es nicht
        }
        let slots: Vec<usize> = (0..players).collect();
        let mut g = Game::with_rng(&slots, Rng::new(players as u64 * 101));
        g.bots = slots.clone();
        let mut bots: Vec<Bot> = slots.iter().map(|&s| Bot::new(s, None)).collect();
        let mut rng = Rng::new(players as u64);
        let mut prev: Option<NetSnap> = None;
        let mut most_shots = 0;
        for tick in 0..(120.0 / TICK) as usize {
            if tick % 20 == 0 {
                let ti = rng.below(g.tanks.len());
                let (x, y) = (g.tanks[ti].x, g.tanks[ti].y);
                // begrenzt wie im echten Spiel: Bots mit Waffe lassen Kisten liegen
                if g.tanks[ti].alive && g.boxes.len() < 8 {
                    g.boxes.push((x, y, NEW[rng.below(NEW.len())]));
                }
            }
            for b in bots.iter_mut() {
                let (k, f) = b.think(&g);
                g.apply_input(b.slot, k, f, TICK);
            }
            g.step(TICK);
            for t in &g.tanks {
                if t.alive && !t.has(Eff::Ghost) {
                    // Kisten liefern auch den Wandbauer, siehe fuzz.rs
                    let slack = if g.temp.is_empty() { 0.5 } else { 4.0 };
                    assert!(
                        !circle_hits_walls(t.x, t.y, t.radius() - slack, g.arena.near(t.x, t.y)),
                        "Panzer {} steckt in der Wand (tick {tick}, {players} Spieler) bei ({:.1}, {:.1}), r={:.1}, Effekte {:?}, Geschosse: {:?}",
                        t.slot,
                        t.x,
                        t.y,
                        t.radius(),
                        panzer_core::items::ALL_EFFS.iter().filter(|&&e| t.has(e)).collect::<Vec<_>>(),
                        g.shots
                            .iter()
                            .filter(|s| !s.kind.is_static() || s.owner == t.slot)
                            .map(|s| (s.kind, s.owner, s.stuck, s.x.round(), s.y.round()))
                            .collect::<Vec<_>>()
                    );
                }
            }
            for s in &g.shots {
                used.entry(s.kind).or_default().insert((players, s.id));
            }
            if tick % 2 == 0 {
                let s = g.net_snap(Some(0), true);
                let mut w = Writer::new();
                write_snap(&mut w, &s, prev.as_ref());
                let back = read_snap(&mut Reader::new(&w.buf), prev.as_ref()).unwrap_or_else(|e| {
                    panic!(
                        "Snapshot nicht lesbar: {e:?} bei {} Geschossen, {} Kisten, {} Fx",
                        s.shots.len(),
                        s.boxes.len(),
                        s.fx.len()
                    )
                });
                assert_eq!(back, s);
                most_shots = most_shots.max(s.shots.len());
                prev = Some(s);
            }
        }
        println!("{players} Spieler: höchstens {most_shots} Geschosse gleichzeitig");
    }
    for k in KINDS {
        let n = used.get(&k).map_or(0, |s| s.len());
        println!("{k:?}: {n}-mal");
        assert!(n > 0, "Bots haben {k:?} nie benutzt");
    }
}
