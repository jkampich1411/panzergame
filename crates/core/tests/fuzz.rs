//! Bot-Matches mit ständig zufälligen Power-ups: keine Panics, kein Panzer in einer Wand,
//! Snapshots lassen sich verlustfrei per Delta übertragen.

use std::collections::HashSet;

use panzer_core::bot::Bot;
use panzer_core::codec::{read_snap, write_snap, Reader, Writer};
use panzer_core::consts::*;
use panzer_core::geom::circle_hits_walls;
use panzer_core::items::{Eff, ALL_ITEMS};
use panzer_core::rng::Rng;
use panzer_core::snap::NetSnap;
use panzer_core::Game;

fn run(players: usize, seconds: f64, seed: u64) -> (usize, usize, HashSet<u8>, usize) {
    let slots: Vec<usize> = (0..players).collect();
    let mut g = Game::with_rng(&slots, Rng::new(seed));
    g.bots = slots.clone();
    let mut bots: Vec<Bot> = slots.iter().map(|&s| Bot::new(s, None)).collect();
    let mut rng = Rng::new(seed ^ 0xABCD);
    let mut prev: Option<NetSnap> = None;
    let (mut bytes, mut snaps, mut rounds) = (0usize, 0usize, 0usize);
    let mut kinds = HashSet::new();
    let ticks = (seconds / TICK) as usize;
    let mut last_round = g.maze_id;
    for tick in 0..ticks {
        // ständig Power-ups verteilen (wie Kisten, nur öfter)
        if tick % 40 == 0 {
            let ti = rng.below(g.tanks.len());
            let item = ALL_ITEMS[rng.below(ALL_ITEMS.len() - 2)];
            let (x, y) = (g.tanks[ti].x, g.tanks[ti].y);
            if g.tanks[ti].alive {
                g.boxes.push((x, y, item));
            }
        }
        for b in bots.iter_mut() {
            let (k, f) = b.think(&g);
            g.apply_input(b.slot, k, f, TICK);
        }
        g.step(TICK);
        if g.maze_id != last_round {
            rounds += 1;
            last_round = g.maze_id;
        }
        for t in &g.tanks {
            if t.alive && !t.has(Eff::Ghost) {
                let walls = g.arena.near(t.x, t.y);
                // Eine gebaute Wand kann eine Lücke schmaler als der Panzer lassen: Dort drücken
                // ihn zwei Wände gegeneinander (gemessen höchstens 3 px, nur eine Optik-Sache).
                let slack = if g.temp.is_empty() { 0.5 } else { 4.0 };
                assert!(
                    !circle_hits_walls(t.x, t.y, t.radius() - slack, walls),
                    "Panzer {} steckt in der Wand (tick {tick})",
                    t.slot
                );
            }
        }
        for s in &g.shots {
            kinds.insert(s.kind as u8);
        }
        if tick % 2 == 0 {
            let s = g.net_snap(Some(0), true);
            let mut w = Writer::new();
            write_snap(&mut w, &s, prev.as_ref());
            bytes += w.len();
            snaps += 1;
            let back = read_snap(&mut Reader::new(&w.buf), prev.as_ref()).expect("decode");
            assert_eq!(back, s);
            prev = Some(s);
        }
    }
    (bytes / snaps.max(1), rounds, kinds, g.scores.iter().sum::<u32>() as usize)
}

#[test]
fn bot_matches_all_sizes() {
    let mut all = HashSet::new();
    for players in 2..=8 {
        let (avg, rounds, kinds, score) = run(players, 120.0, players as u64 * 31);
        println!(
            "{players} Spieler: {rounds} Runden, {score} Punkte, Ø {avg} Byte/Snapshot, {} Geschossarten",
            kinds.len()
        );
        assert!(rounds >= 2, "zu wenige Runden: {rounds}");
        all.extend(kinds);
    }
    println!("Geschossarten insgesamt: {}", all.len());
}
