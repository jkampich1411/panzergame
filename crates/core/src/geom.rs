//! Geometrie: Kreis gegen Rechtecke, Strahlen, Winkel.

use std::f64::consts::{PI, TAU};

/// Wandrechteck x, y, w, h
pub type Wall = [f64; 4];

/// wie Pythons a % m (Ergebnis hat das Vorzeichen von m)
#[inline]
pub fn pmod(a: f64, m: f64) -> f64 {
    a - m * (a / m).floor()
}

/// Kürzester Winkel von b nach a (-pi..pi).
#[inline]
pub fn angle_diff(a: f64, b: f64) -> f64 {
    pmod(a - b + PI, TAU) - PI
}

#[inline]
pub fn lerp_angle(a: f64, b: f64, k: f64) -> f64 {
    a + (pmod(b - a + PI, TAU) - PI) * k
}

#[inline]
pub fn hypot(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt()
}

pub fn circle_hits_wall(cx: f64, cy: f64, r: f64, w: &Wall) -> bool {
    let [x, y, ww, hh] = *w;
    let nx = if cx < x {
        x
    } else if cx > x + ww {
        x + ww
    } else {
        cx
    };
    let ny = if cy < y {
        y
    } else if cy > y + hh {
        y + hh
    } else {
        cy
    };
    (cx - nx) * (cx - nx) + (cy - ny) * (cy - ny) < r * r
}

pub fn circle_hits_walls<'a>(cx: f64, cy: f64, r: f64, walls: impl IntoIterator<Item = &'a Wall>) -> bool {
    walls.into_iter().any(|w| circle_hits_wall(cx, cy, r, w))
}

/// Schiebt einen Kreis aus allen Wänden heraus.
pub fn push_out<'a, I>(mut cx: f64, mut cy: f64, r: f64, walls: I) -> (f64, f64)
where
    I: IntoIterator<Item = &'a Wall> + Clone,
{
    for _ in 0..4 {
        let mut moved = false;
        for w in walls.clone() {
            let [x, y, ww, hh] = *w;
            let nx = cx.max(x).min(x + ww);
            let ny = cy.max(y).min(y + hh);
            let (dx, dy) = (cx - nx, cy - ny);
            let d2 = dx * dx + dy * dy;
            if d2 >= r * r {
                continue;
            }
            moved = true;
            if d2 > 1e-9 {
                let d = d2.sqrt();
                cx += dx / d * (r - d);
                cy += dy / d * (r - d);
            } else {
                // Mittelpunkt steckt in der Wand: zur nächsten Seite raus
                let pens = [cx - x, x + ww - cx, cy - y, y + hh - cy];
                let mut side = 0;
                for k in 1..4 {
                    if pens[k] < pens[side] {
                        side = k;
                    }
                }
                match side {
                    0 => cx = x - r,
                    1 => cx = x + ww + r,
                    2 => cy = y - r,
                    _ => cy = y + hh + r,
                }
            }
        }
        if !moved {
            break;
        }
    }
    (cx, cy)
}

/// Getroffene Achse eines Strahls
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axis {
    X,
    Y,
    XY,
}

impl Axis {
    pub fn has_x(self) -> bool {
        matches!(self, Axis::X | Axis::XY)
    }
    pub fn has_y(self) -> bool {
        matches!(self, Axis::Y | Axis::XY)
    }
}

/// Eintritts-Distanz eines Strahls in ein Rechteck und die getroffene Achse.
pub fn ray_box(x: f64, y: f64, dx: f64, dy: f64, w: &Wall) -> Option<(f64, Axis)> {
    let [bx, by, bw, bh] = *w;
    let (tx0, tx1);
    if dx.abs() < 1e-12 {
        if !(bx <= x && x <= bx + bw) {
            return None;
        }
        tx0 = f64::NEG_INFINITY;
        tx1 = f64::INFINITY;
    } else {
        let t1 = (bx - x) / dx;
        let t2 = (bx + bw - x) / dx;
        tx0 = t1.min(t2);
        tx1 = t1.max(t2);
    }
    let (ty0, ty1);
    if dy.abs() < 1e-12 {
        if !(by <= y && y <= by + bh) {
            return None;
        }
        ty0 = f64::NEG_INFINITY;
        ty1 = f64::INFINITY;
    } else {
        let t1 = (by - y) / dy;
        let t2 = (by + bh - y) / dy;
        ty0 = t1.min(t2);
        ty1 = t1.max(t2);
    }
    let t_in = tx0.max(ty0);
    let t_out = tx1.min(ty1);
    if t_in > t_out || t_in < 1e-9 {
        return None; // verfehlt, dahinter oder wir stecken drin
    }
    let axis = if tx0 > ty0 + 1e-9 {
        Axis::X
    } else if ty0 > tx0 + 1e-9 {
        Axis::Y
    } else {
        Axis::XY
    };
    Some((t_in, axis))
}

pub fn ray_circle(x: f64, y: f64, dx: f64, dy: f64, cx: f64, cy: f64, r: f64) -> Option<f64> {
    let (fx, fy) = (x - cx, y - cy);
    let b = fx * dx + fy * dy;
    let c = fx * fx + fy * fy - r * r;
    if c < 0.0 {
        return None; // Start im Kreis -> ignorieren (eigener Panzer)
    }
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let t = -b - disc.sqrt();
    if t > 0.0 {
        Some(t)
    } else {
        None
    }
}

/// Abstand von (px, py) zur Strecke (x0, y0)-(x1, y1).
pub fn seg_dist(px: f64, py: f64, x0: f64, y0: f64, x1: f64, y1: f64) -> f64 {
    let (dx, dy) = (x1 - x0, y1 - y0);
    let l = dx * dx + dy * dy;
    let k = if l == 0.0 { 0.0 } else { (((px - x0) * dx + (py - y0) * dy) / l).clamp(0.0, 1.0) };
    hypot(px - x0 - dx * k, py - y0 - dy * k)
}

/// Rechtecke überlappen sich
pub fn rects_overlap(a: &Wall, b: &Wall) -> bool {
    a[0] < b[0] + b[2] && a[0] + a[2] > b[0] && a[1] < b[1] + b[3] && a[1] + a[3] > b[1]
}
