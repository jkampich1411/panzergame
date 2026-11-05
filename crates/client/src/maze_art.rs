//! Labyrinth-Grafik: Boden mit Platten und Struktur, Wände mit Kanten und weichem Schatten,
//! Vignette. Wird einmal pro Runde gebaut, bei weggesprengten Wänden nur der Ausschnitt.

use std::f32::consts::{PI, TAU};

use panzer_core::consts::*;
use panzer_core::rng::Rng;
use panzer_core::Arena;

use crate::raster::*;

/// Boden ohne Wände (ändert sich während einer Runde nie)
pub fn build_floor(arena: &Arena) -> Canvas {
    let th = arena.theme();
    let (ww, wh) = (arena.ww as usize, arena.wh as usize);
    let mut rnd = Rng::new(arena.id as u64 * 7919 + arena.cols as u64 * 31 + arena.rows as u64);
    let mut u = |a: f32, b: f32| rnd.uniform(a as f64, b as f64) as f32;
    let mut cv = Canvas::filled(ww, wh, rgb(shade(th.floor, 0.4)));
    cv.ss = 1;
    let (x0, y0) = (arena.ox as f32, arena.oy as f32);
    let cell = CELL as f32;
    let (fw, fh) = (arena.cols as f32 * cell, arena.rows as f32 * cell);
    for r in 0..arena.rows {
        for c in 0..arena.cols {
            // Bodenplatten, jede etwas anders hell
            let k = u(0.93, 1.07);
            cv.rect(x0 + c as f32 * cell, y0 + r as f32 * cell, cell, cell, rgb(shade(th.floor, k)));
        }
    }
    for _ in 0..(fw * fh / 240.0) as usize {
        // feine Sprenkel
        let (px, py) = (u(x0, x0 + fw), u(y0, y0 + fh));
        let size = if u(0.0, 1.0) < 0.75 { 1.0 } else { 2.0 };
        let k = u(0.8, 1.22);
        cv.rect(px, py, size, size, rgb(shade(th.floor, k)));
    }
    cv.ss = 2;
    match th.amb {
        Ambient::Dust | Ambient::Neon => {
            // Metallplatten bzw. Gitter
            for r in 0..arena.rows {
                for c in 0..arena.cols {
                    let (cx, cy) = (x0 + c as f32 * cell, y0 + r as f32 * cell);
                    let mid = cell / 2.0;
                    cv.rect(cx + mid, cy, 1.0, cell, rgb(th.line));
                    cv.rect(cx, cy + mid, cell, 1.0, rgb(th.line));
                    if th.amb == Ambient::Dust {
                        for (dx, dy) in [(8.0, 8.0), (cell - 8.0, 8.0), (8.0, cell - 8.0), (cell - 8.0, cell - 8.0)] {
                            cv.circle(cx + dx, cy + dy, 1.6, rgb(shade(th.floor, 1.35)));
                        }
                    }
                }
            }
        }
        Ambient::Sand => {
            // Rippel
            for _ in 0..(fw * fh / 2600.0) as usize {
                let (x, y) = (u(x0, x0 + fw - 40.0), u(y0, y0 + fh));
                let n = u(6.0, 13.0) as usize;
                let pts: Vec<(f32, f32)> =
                    (0..n).map(|i| (x + i as f32 * 4.0, y + (i as f32 * 0.7).sin() * 2.0)).collect();
                cv.polyline(&pts, 1.0, rgb(shade(th.floor, 1.12)));
                let low: Vec<(f32, f32)> = pts.iter().map(|&(px, py)| (px, py + 2.0)).collect();
                cv.polyline(&low, 1.0, rgb(shade(th.floor, 0.88)));
            }
        }
        Ambient::Snow => {
            // Frost
            for _ in 0..(fw * fh / 1800.0) as usize {
                let (x, y) = (u(x0, x0 + fw), u(y0, y0 + fh));
                let (a, d) = (u(0.0, PI), u(6.0, 22.0));
                cv.line(x, y, x + a.cos() * d, y + a.sin() * d, 1.0, rgb(shade(th.floor, 1.25)));
            }
        }
        Ambient::Firefly => {
            // Laub
            for _ in 0..(fw * fh / 700.0) as usize {
                let (x, y) = (u(x0, x0 + fw), u(y0, y0 + fh));
                let (lw, lh) = (u(3.0, 8.0), u(2.0, 5.0));
                let k = [0.75, 1.25, 1.4][(u(0.0, 2.99)) as usize];
                cv.ellipse(x, y, lw / 2.0, lh / 2.0, rgb(shade(th.floor, k)));
            }
        }
        Ambient::Ember => {}
    }
    // Fugen zwischen den Zellen
    let joint = rgb(shade(th.line, 1.15));
    for c in 0..=arena.cols {
        cv.rect(x0 + c as f32 * cell, y0, 1.0, fh, joint);
    }
    for r in 0..=arena.rows {
        cv.rect(x0, y0 + r as f32 * cell, fw, 1.0, joint);
    }
    // ein paar Ölflecken
    let stain = shade(th.floor, 0.45);
    for _ in 0..((fw * fh / 90000.0) as usize).max(2) {
        let (x, y) = (u(x0, x0 + fw), u(y0, y0 + fh));
        let r = u(10.0, 22.0);
        for i in 0..5 {
            let q = r * (1.0 - i as f32 / 5.0);
            cv.circle(x, y, q, rgba(stain, 22));
        }
    }
    if th.cracks {
        // glühende Risse im Boden
        let glow = th.glow.unwrap_or([255, 80, 15]);
        let mut cracks = Vec::new();
        for _ in 0..((fw * fh / 15000.0) as usize).max(5) {
            let (mut x, mut y) = (u(x0, x0 + fw), u(y0, y0 + fh));
            let mut a = u(0.0, TAU);
            let mut pts = vec![(x, y)];
            for _ in 0..u(3.0, 8.0) as usize {
                a += u(-0.9, 0.9);
                let d = u(8.0, 24.0);
                x += a.cos() * d;
                y += a.sin() * d;
                pts.push((x.clamp(x0, x0 + fw), y.clamp(y0, y0 + fh)));
            }
            cracks.push(pts);
        }
        let mut layer = Canvas::new(ww, wh);
        layer.ss = 1;
        for pts in &cracks {
            layer.polyline(pts, 5.0, rgb(glow));
        }
        layer.blur(4);
        cv.add(&layer, 120.0 / 255.0 * 1.6);
        cv.ss = 2;
        for pts in &cracks {
            cv.polyline(pts, 2.0, rgb([190, 80, 30]));
            cv.polyline(pts, 1.0, rgb([255, 190, 110]));
        }
    }
    cv
}

/// Boden plus Wände plus Vignette für einen Ausschnitt (x, y, w, h) der Welt
pub fn build_area(arena: &Arena, floor: &Canvas, area: (usize, usize, usize, usize)) -> Canvas {
    let th = arena.theme();
    let (ax, ay, aw, ah) = area;
    let mut cv = Canvas::new(aw, ah);
    for y in 0..ah {
        let src = (ay + y) * floor.w + ax;
        cv.px[y * aw..(y + 1) * aw].copy_from_slice(&floor.px[src..src + aw]);
    }
    // Wände in der Nähe, verschoben in den Ausschnitt
    let walls: Vec<[f32; 4]> = arena
        .walls
        .iter()
        .filter(|w| {
            w[0] - 20.0 < (ax + aw) as f64
                && w[0] + w[2] + 20.0 > ax as f64
                && w[1] - 20.0 < (ay + ah) as f64
                && w[1] + w[3] + 20.0 > ay as f64
        })
        .map(|w| [w[0] as f32 - ax as f32, w[1] as f32 - ay as f32, w[2] as f32, w[3] as f32])
        .collect();
    // Wände als Maske: so gibt es an Kreuzungen keine Nähte
    let mut mask = vec![false; aw * ah];
    for w in &walls {
        let x0 = w[0].max(0.0) as usize;
        let y0 = w[1].max(0.0) as usize;
        let x1 = ((w[0] + w[2]).max(0.0) as usize).min(aw);
        let y1 = ((w[1] + w[3]).max(0.0) as usize).min(ah);
        for y in y0..y1 {
            for x in x0..x1 {
                mask[y * aw + x] = true;
            }
        }
    }
    let inside = |x: i64, y: i64| {
        x >= 0 && y >= 0 && (x as usize) < aw && (y as usize) < ah && mask[y as usize * aw + x as usize]
    };
    // Schlagschatten nach rechts unten
    let mut shadow = Canvas::new(aw, ah);
    shadow.ss = 1;
    for w in &walls {
        shadow.rect(w[0] + 4.0, w[1] + 7.0, w[2] + 2.0, w[3], [0.0, 0.0, 0.0, 150.0 / 255.0]);
    }
    shadow.blur(2);
    cv.blit(&shadow, 0, 0);
    if let (Some(glow), false) = (th.glow, th.cracks) {
        // Neon: die Wände leuchten
        let mut g = Canvas::new(aw, ah);
        let col = rgb(glow);
        for (i, &m) in mask.iter().enumerate() {
            if m {
                g.px[i] = col;
            }
        }
        g.blur(5);
        cv.add(&g, 2.0);
    }
    let (wall, lo, hi) = (rgb(th.wall), rgb(th.lo), rgb(th.hi));
    for y in 0..ah as i64 {
        for x in 0..aw as i64 {
            if !inside(x, y) {
                continue;
            }
            // untere/rechte Kante: 2 px weiter rechts unten ist keine Wand mehr
            let c = if !inside(x - 2, y - 2) {
                hi
            } else if !inside(x + 2, y + 2) {
                lo
            } else {
                wall
            };
            cv.px[y as usize * aw + x as usize] = c;
        }
    }
    // Vignette (abdunkeln zu den Rändern)
    let (ww, wh) = (arena.ww as f32, arena.wh as f32);
    for y in 0..ah {
        for x in 0..aw {
            let gx = (ax + x) as f32 / ww * 48.0 - 0.5;
            let gy = (ay + y) as f32 / wh * 30.0 - 0.5;
            let d = (((gx - 23.5) / 24.0).hypot((gy - 14.5) / 15.0) / 1.3).min(1.0);
            let v = 1.0 - 0.5 * d.powf(2.4);
            let p = &mut cv.px[y * aw + x];
            p[0] *= v;
            p[1] *= v;
            p[2] *= v;
        }
    }
    cv
}
