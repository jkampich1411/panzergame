//! Panzerbewegung und Hilfen, die auf Server und Client gleich laufen müssen.

use crate::arena::Arena;
use crate::consts::*;
use crate::geom::{circle_hits_walls, hypot, push_out};
use crate::items::{Eff, EFF_N};

/// Aktive Effekte eines Panzers: Restzeit > 0 = aktiv
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Effects(pub [f64; EFF_N]);

impl Effects {
    #[inline]
    pub fn has(&self, e: Eff) -> bool {
        self.0[e as usize] > 0.0
    }
    #[inline]
    pub fn get(&self, e: Eff) -> f64 {
        self.0[e as usize]
    }
    #[inline]
    pub fn set(&mut self, e: Eff, t: f64) {
        self.0[e as usize] = t;
    }
    #[inline]
    pub fn remove(&mut self, e: Eff) -> f64 {
        std::mem::take(&mut self.0[e as usize])
    }
    pub fn clear(&mut self) {
        self.0 = [0.0; EFF_N];
    }
    pub fn any(&self) -> bool {
        self.0.iter().any(|&v| v > 0.0)
    }
    /// Bitmaske der aktiven Effekte
    pub fn mask(&self) -> u32 {
        let mut m = 0;
        for (i, &v) in self.0.iter().enumerate() {
            if v > 0.0 {
                m |= 1 << i;
            }
        }
        m
    }
}

/// Bewegbarer Panzer (Server-Tank oder vorhergesagter Client-Panzer)
#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub x: f64,
    pub y: f64,
    pub a: f64,
}

pub fn tank_scale(eff: &Effects) -> f64 {
    if eff.has(Eff::Mini) {
        MINI_SCALE
    } else if eff.has(Eff::Giant) {
        GIANT_SCALE
    } else {
        1.0
    }
}

pub fn tank_radius(eff: &Effects) -> f64 {
    TANK_R * tank_scale(eff)
}

/// Normale Kugeln je nach Sonderregel
#[derive(Clone, Copy)]
pub struct BulletParams {
    pub speed: f64,
    pub r: f64,
    pub life: f64,
    pub most: usize,
}

pub fn bullet_params(m: Option<Mutator>) -> BulletParams {
    let (speed, r, life, most) = match m {
        Some(Mutator::Hyper) => (BULLET_SPEED * 1.8, BULLET_R, BULLET_LIFE, MAX_BULLETS),
        Some(Mutator::Riesen) => (BULLET_SPEED * 0.85, 10.0, BULLET_LIFE, MAX_BULLETS),
        Some(Mutator::Sniper) => (BULLET_SPEED * 2.2, BULLET_R, 3.5, 1),
        _ => (BULLET_SPEED, BULLET_R, BULLET_LIFE, MAX_BULLETS),
    };
    BulletParams { speed, r, life, most }
}

pub fn clamp_arena(x: f64, y: f64, r: f64, arena: &Arena) -> (f64, f64) {
    let x0 = arena.ox + r;
    let y0 = arena.oy + r;
    let x1 = arena.ox + arena.cols as f64 * CELL - r;
    let y1 = arena.oy + arena.rows as f64 * CELL - r;
    (x.max(x0).min(x1), y.max(y0).min(y1))
}

/// Panzerbewegung – identisch auf Server und Client (Prediction).
/// charging = Railgun lädt (bremst auf die Hälfte).
pub fn move_tank(t: &mut Body, eff: &Effects, charging: bool, mut keys: u8, dt: f64, arena: &Arena) {
    if eff.has(Eff::Frozen) || eff.has(Eff::Steer) {
        return; // eingefroren bzw. lenkt gerade die Rakete
    }
    if eff.has(Eff::Confuse) {
        // vorne/hinten und links/rechts vertauscht
        keys = ((keys & UP) << 1) | ((keys & DOWN) >> 1) | ((keys & LEFT) << 1) | ((keys & RIGHT) >> 1);
    }
    let (mut speed, mut rot) = (TANK_SPEED, TANK_ROT);
    if eff.has(Eff::Turbo) {
        speed *= TURBO_SPEED;
        rot *= 1.25;
    }
    if eff.has(Eff::Mini) {
        speed *= MINI_SPEED;
    }
    if eff.has(Eff::Giant) {
        speed *= 0.85;
    }
    if eff.has(Eff::Star) {
        speed *= 1.3;
    }
    if eff.has(Eff::Ram) {
        speed *= 1.5;
    }
    if eff.has(Eff::Slow) {
        speed *= 0.55;
        rot *= 0.7;
    }
    if eff.has(Eff::Slip) {
        // auf Öl: rutschig und schwer zu lenken
        speed *= 1.25;
        rot *= 1.7;
    }
    if charging {
        speed *= 0.5;
    }
    if arena.mutator == Some(Mutator::Turbo) {
        speed *= 1.4;
    }
    if keys & LEFT != 0 {
        t.a -= rot * dt;
    }
    if keys & RIGHT != 0 {
        t.a += rot * dt;
    }
    t.a = crate::geom::pmod(t.a, std::f64::consts::TAU);
    let mv = (if keys & UP != 0 { 1.0 } else { 0.0 }) - (if keys & DOWN != 0 { 0.7 } else { 0.0 });
    if mv != 0.0 {
        let r = tank_radius(eff);
        t.x += t.a.cos() * speed * mv * dt;
        t.y += t.a.sin() * speed * mv * dt;
        let (nx, ny) = if eff.has(Eff::Ghost) {
            clamp_arena(t.x, t.y, r, arena) // durch Wände, aber nicht aus dem Labyrinth raus
        } else {
            push_out(t.x, t.y, r, arena.near(t.x, t.y))
        };
        t.x = nx;
        t.y = ny;
    }
}

/// Strecke bis zur ersten Wand in Richtung a (höchstens length).
pub fn first_wall(arena: &Arena, x: f64, y: f64, a: f64, length: f64) -> f64 {
    let (pts, _) = arena.raycast(x, y, a, length, &[]);
    if pts.len() > 1 {
        hypot(pts[1].0 - x, pts[1].1 - y)
    } else {
        0.0
    }
}

/// Zielpunkt des Luftschlags: STRIKE_DIST voraus, aber vor der ersten Wand.
pub fn strike_target(arena: &Arena, x: f64, y: f64, a: f64) -> (f64, f64) {
    let d = (first_wall(arena, x, y, a, STRIKE_DIST) - 14.0).max(0.0);
    (x + a.cos() * d, y + a.sin() * d)
}

/// Punkt d voraus, Wände egal (fliegt drüber), aber im Labyrinth.
pub fn ahead(arena: &Arena, x: f64, y: f64, a: f64, d: f64, r: f64) -> (f64, f64) {
    clamp_arena(x + a.cos() * d, y + a.sin() * d, r, arena)
}

/// Railgun-Strahl: geradeaus durch bis zu RAIL_WALLS Wände. (Endpunkt, Durchschläge)
pub fn rail_path(arena: &Arena, x: f64, y: f64, a: f64) -> ((f64, f64), Vec<(i32, i32)>) {
    let (dx, dy) = (a.cos(), a.sin());
    let (x0, y0, x1, y1) = arena.bounds();
    let mut holes = Vec::new();
    let mut inside = false;
    let mut d = MUZZLE;
    while d < RAIL_LEN {
        let (px, py) = (x + dx * d, y + dy * d);
        if !(x0 < px && px < x1 && y0 < py && py < y1) {
            break;
        }
        let hit = circle_hits_walls(px, py, 1.5, arena.near(px, py));
        if hit && !inside {
            if holes.len() == RAIL_WALLS {
                break;
            }
            holes.push((px.round() as i32, py.round() as i32));
        }
        inside = hit;
        d += 3.0;
    }
    ((x + dx * d, y + dy * d), holes)
}
