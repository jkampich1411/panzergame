//! Kleiner Software-Rasterizer mit Kantenglättung (4×4 Abtastpunkte pro Pixel).
//! Damit entstehen beim Start alle Sprites (Panzer, Kisten, Symbole) und das Labyrinth –
//! wie früher mit pygame, nur ohne pygame.

use std::f32::consts::TAU;

pub type Rgba = [f32; 4];

pub fn rgb(c: [u8; 3]) -> Rgba {
    [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, 1.0]
}

pub fn rgba(c: [u8; 3], a: u8) -> Rgba {
    [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, a as f32 / 255.0]
}

#[derive(Clone)]
pub struct Canvas {
    pub w: usize,
    pub h: usize,
    /// vormultipliziertes RGBA
    pub px: Vec<Rgba>,
    /// Abtastpunkte pro Achse (1 = ohne Glättung, schneller fürs Labyrinth)
    pub ss: usize,
}

/// Transformation: Punkt in Zeichen-Einheiten -> Pixel
#[derive(Clone, Copy)]
pub struct Xf {
    pub ox: f32,
    pub oy: f32,
    pub k: f32,
    pub cos: f32,
    pub sin: f32,
}

impl Xf {
    pub fn rot(ox: f32, oy: f32, k: f32, a: f32) -> Xf {
        Xf { ox, oy, k, cos: a.cos(), sin: a.sin() }
    }
    pub fn p(&self, x: f32, y: f32) -> (f32, f32) {
        (self.ox + (x * self.cos - y * self.sin) * self.k, self.oy + (x * self.sin + y * self.cos) * self.k)
    }
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Canvas {
        Canvas { w, h, px: vec![[0.0; 4]; w * h], ss: 4 }
    }

    pub fn filled(w: usize, h: usize, c: Rgba) -> Canvas {
        let p = premul(c);
        Canvas { w, h, px: vec![p; w * h], ss: 4 }
    }

    /// Pixel mit Deckung cov (0..1) und Farbe c überblenden
    #[inline]
    pub fn blend(&mut self, x: usize, y: usize, c: Rgba, cov: f32) {
        let a = c[3] * cov;
        if a <= 0.0 {
            return;
        }
        let d = &mut self.px[y * self.w + x];
        let inv = 1.0 - a;
        d[0] = c[0] * a + d[0] * inv;
        d[1] = c[1] * a + d[1] * inv;
        d[2] = c[2] * a + d[2] * inv;
        d[3] = a + d[3] * inv;
    }

    /// Form über eine Innen-Prüfung zeichnen (bbox in Pixeln)
    pub fn fill_fn(&mut self, bbox: (f32, f32, f32, f32), c: Rgba, inside: impl Fn(f32, f32) -> bool) {
        let x0 = bbox.0.floor().max(0.0) as i64;
        let y0 = bbox.1.floor().max(0.0) as i64;
        let x1 = (bbox.2.ceil() as i64).min(self.w as i64 - 1);
        let y1 = (bbox.3.ceil() as i64).min(self.h as i64 - 1);
        if x1 < x0 || y1 < y0 {
            return;
        }
        let ss = self.ss.max(1);
        let n = (ss * ss) as f32;
        let step = 1.0 / ss as f32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let mut hits = 0;
                for sy in 0..ss {
                    let fy = y as f32 + (sy as f32 + 0.5) * step;
                    for sx in 0..ss {
                        let fx = x as f32 + (sx as f32 + 0.5) * step;
                        if inside(fx, fy) {
                            hits += 1;
                        }
                    }
                }
                if hits > 0 {
                    self.blend(x as usize, y as usize, c, hits as f32 / n);
                }
            }
        }
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, c: Rgba) {
        if r <= 0.0 {
            return;
        }
        let r2 = r * r;
        self.fill_fn((cx - r, cy - r, cx + r, cy + r), c, |x, y| (x - cx).powi(2) + (y - cy).powi(2) <= r2);
    }

    /// Ring mit Breite w (nach innen, wie pygame)
    pub fn ring(&mut self, cx: f32, cy: f32, r: f32, w: f32, c: Rgba) {
        let (ro, ri) = (r * r, (r - w).max(0.0).powi(2));
        self.fill_fn((cx - r, cy - r, cx + r, cy + r), c, |x, y| {
            let d = (x - cx).powi(2) + (y - cy).powi(2);
            d <= ro && d >= ri
        });
    }

    /// Halbkreis oben (für Pilzköpfe)
    pub fn half_circle_top(&mut self, cx: f32, cy: f32, r: f32, c: Rgba) {
        let r2 = r * r;
        self.fill_fn((cx - r, cy - r, cx + r, cy), c, |x, y| y <= cy && (x - cx).powi(2) + (y - cy).powi(2) <= r2);
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgba) {
        self.fill_fn((x, y, x + w, y + h), c, |px, py| px >= x && px <= x + w && py >= y && py <= y + h);
    }

    /// Rechteck mit runden Ecken (Radius rad)
    pub fn rrect(&mut self, x: f32, y: f32, w: f32, h: f32, rad: f32, c: Rgba) {
        let rad = rad.min(w / 2.0).min(h / 2.0).max(0.0);
        self.fill_fn((x, y, x + w, y + h), c, |px, py| {
            if px < x || px > x + w || py < y || py > y + h {
                return false;
            }
            let cx = px.min(x + w - rad).max(x + rad);
            let cy = py.min(y + h - rad).max(y + rad);
            (px - cx).powi(2) + (py - cy).powi(2) <= rad * rad
        });
    }

    /// Rand eines runden Rechtecks
    pub fn rrect_line(&mut self, x: f32, y: f32, w: f32, h: f32, rad: f32, lw: f32, c: Rgba) {
        let outer = |px: f32, py: f32, x: f32, y: f32, w: f32, h: f32, rad: f32| {
            if px < x || px > x + w || py < y || py > y + h {
                return false;
            }
            let rad = rad.min(w / 2.0).min(h / 2.0).max(0.0);
            let cx = px.min(x + w - rad).max(x + rad);
            let cy = py.min(y + h - rad).max(y + rad);
            (px - cx).powi(2) + (py - cy).powi(2) <= rad * rad
        };
        self.fill_fn((x, y, x + w, y + h), c, |px, py| {
            outer(px, py, x, y, w, h, rad)
                && !outer(px, py, x + lw, y + lw, w - 2.0 * lw, h - 2.0 * lw, (rad - lw).max(0.0))
        });
    }

    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, c: Rgba) {
        if rx <= 0.0 || ry <= 0.0 {
            return;
        }
        self.fill_fn((cx - rx, cy - ry, cx + rx, cy + ry), c, |x, y| {
            ((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2) <= 1.0
        });
    }

    pub fn ellipse_line(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, w: f32, c: Rgba) {
        let (ix, iy) = ((rx - w).max(0.01), (ry - w).max(0.01));
        self.fill_fn((cx - rx, cy - ry, cx + rx, cy + ry), c, |x, y| {
            ((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2) <= 1.0
                && ((x - cx) / ix).powi(2) + ((y - cy) / iy).powi(2) >= 1.0
        });
    }

    /// Dicke Linie mit runden Enden
    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, w: f32, c: Rgba) {
        let r = (w / 2.0).max(0.35);
        let (dx, dy) = (x1 - x0, y1 - y0);
        let l2 = dx * dx + dy * dy;
        let bbox = (x0.min(x1) - r, y0.min(y1) - r, x0.max(x1) + r, y0.max(y1) + r);
        self.fill_fn(bbox, c, |x, y| {
            let t = if l2 == 0.0 { 0.0 } else { (((x - x0) * dx + (y - y0) * dy) / l2).clamp(0.0, 1.0) };
            (x - x0 - dx * t).powi(2) + (y - y0 - dy * t).powi(2) <= r * r
        });
    }

    pub fn polyline(&mut self, pts: &[(f32, f32)], w: f32, c: Rgba) {
        for p in pts.windows(2) {
            self.line(p[0].0, p[0].1, p[1].0, p[1].1, w, c);
        }
    }

    /// Gefülltes Polygon (gerade/ungerade-Regel)
    pub fn poly(&mut self, pts: &[(f32, f32)], c: Rgba) {
        if pts.len() < 3 {
            return;
        }
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for &(x, y) in pts {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
        let pts = pts.to_vec();
        self.fill_fn((x0, y0, x1, y1), c, move |x, y| {
            let mut inside = false;
            let mut j = pts.len() - 1;
            for i in 0..pts.len() {
                let (xi, yi) = pts[i];
                let (xj, yj) = pts[j];
                if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
                    inside = !inside;
                }
                j = i;
            }
            inside
        });
    }

    pub fn poly_line(&mut self, pts: &[(f32, f32)], w: f32, c: Rgba) {
        for i in 0..pts.len() {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            self.line(a.0, a.1, b.0, b.1, w, c);
        }
    }

    /// Kreisbogen von a0 bis a1 (Bogenmaß, pygame-Richtung: gegen den Uhrzeigersinn, y nach oben)
    pub fn arc(&mut self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, w: f32, c: Rgba) {
        let n = (((a1 - a0).abs() * r / 2.0).ceil() as usize).clamp(4, 64);
        let pts: Vec<(f32, f32)> = (0..=n)
            .map(|i| {
                let a = a0 + (a1 - a0) * i as f32 / n as f32;
                (cx + a.cos() * (r - w / 2.0), cy - a.sin() * (r - w / 2.0))
            })
            .collect();
        self.polyline(&pts, w, c);
    }

    /// Andere Leinwand an (x, y) drüberlegen
    pub fn blit(&mut self, other: &Canvas, x: i64, y: i64) {
        for oy in 0..other.h {
            let ty = y + oy as i64;
            if ty < 0 || ty >= self.h as i64 {
                continue;
            }
            for ox in 0..other.w {
                let tx = x + ox as i64;
                if tx < 0 || tx >= self.w as i64 {
                    continue;
                }
                let s = other.px[oy * other.w + ox];
                if s[3] <= 0.0 {
                    continue;
                }
                let d = &mut self.px[ty as usize * self.w + tx as usize];
                let inv = 1.0 - s[3];
                d[0] = s[0] + d[0] * inv;
                d[1] = s[1] + d[1] * inv;
                d[2] = s[2] + d[2] * inv;
                d[3] = s[3] + d[3] * inv;
            }
        }
    }

    /// Additiv drüberlegen (Licht, Glühen)
    pub fn add(&mut self, other: &Canvas, k: f32) {
        for (d, s) in self.px.iter_mut().zip(other.px.iter()) {
            d[0] = (d[0] + s[0] * k).min(1.0);
            d[1] = (d[1] + s[1] * k).min(1.0);
            d[2] = (d[2] + s[2] * k).min(1.0);
        }
    }

    /// Kastenweichzeichner (3 Durchgänge ≈ Gauß)
    pub fn blur(&mut self, r: usize) {
        if r == 0 {
            return;
        }
        for _ in 0..3 {
            self.blur_pass(r, true);
            self.blur_pass(r, false);
        }
    }

    fn blur_pass(&mut self, r: usize, horizontal: bool) {
        let (w, h) = (self.w, self.h);
        let (outer, inner) = if horizontal { (h, w) } else { (w, h) };
        let idx = |o: usize, i: usize| if horizontal { o * w + i } else { i * w + o };
        let mut line = vec![[0.0f32; 4]; inner];
        let k = 1.0 / (2 * r + 1) as f32;
        for o in 0..outer {
            let mut acc = [0.0f32; 4];
            for i in 0..=r.min(inner - 1) {
                let p = self.px[idx(o, i)];
                for c in 0..4 {
                    acc[c] += p[c];
                }
            }
            for i in 0..inner {
                for c in 0..4 {
                    line[i][c] = acc[c] * k;
                }
                let add = i + r + 1;
                if add < inner {
                    let p = self.px[idx(o, add)];
                    for c in 0..4 {
                        acc[c] += p[c];
                    }
                }
                if i >= r {
                    let p = self.px[idx(o, i - r)];
                    for c in 0..4 {
                        acc[c] -= p[c];
                    }
                }
            }
            for i in 0..inner {
                self.px[idx(o, i)] = line[i];
            }
        }
    }

    /// Bytes für eine Textur (nicht vormultipliziert)
    pub fn to_rgba8(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.w * self.h * 4);
        for p in &self.px {
            let a = p[3];
            if a <= 0.0 {
                out.extend_from_slice(&[0, 0, 0, 0]);
                continue;
            }
            for c in 0..3 {
                out.push(((p[c] / a).clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
            }
            out.push((a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
        }
        out
    }
}

#[inline]
fn premul(c: Rgba) -> Rgba {
    [c[0] * c[3], c[1] * c[3], c[2] * c[3], c[3]]
}

/// Punkte rundherum (abwechselnd außen/innen, wenn r_in gesetzt ist: Stern, Zähne)
pub fn round_pts(n: usize, r_out: f32, r_in: Option<f32>, rot: f32) -> Vec<(f32, f32)> {
    let m = if r_in.is_some() { 2 * n } else { n };
    let step = TAU / m as f32;
    (0..m)
        .map(|j| {
            let r = match r_in {
                Some(ri) if j % 2 == 1 => ri,
                _ => r_out,
            };
            ((rot + j as f32 * step).cos() * r, (rot + j as f32 * step).sin() * r)
        })
        .collect()
}

pub fn mix(a: [u8; 3], b: [u8; 3], k: f32) -> [u8; 3] {
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * k) as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * k) as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * k) as u8,
    ]
}

pub fn shade(c: [u8; 3], k: f32) -> [u8; 3] {
    [
        (c[0] as f32 * k).clamp(0.0, 255.0) as u8,
        (c[1] as f32 * k).clamp(0.0, 255.0) as u8,
        (c[2] as f32 * k).clamp(0.0, 255.0) as u8,
    ]
}

/// Leinwand um a drehen (bilinear), für gelandete Türme und das App-Symbol
pub fn rotate_canvas(cv: &Canvas, a: f32) -> Canvas {
    let n = ((cv.w * cv.w + cv.h * cv.h) as f32).sqrt().ceil() as usize + 2;
    let mut out = Canvas::new(n, n);
    let (c, s) = (a.cos(), a.sin());
    let (cx, cy) = (n as f32 / 2.0, n as f32 / 2.0);
    let (sx0, sy0) = (cv.w as f32 / 2.0, cv.h as f32 / 2.0);
    for y in 0..n {
        for x in 0..n {
            let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            let (u, v) = (dx * c + dy * s + sx0 - 0.5, -dx * s + dy * c + sy0 - 0.5);
            if u < -1.0 || v < -1.0 || u >= cv.w as f32 || v >= cv.h as f32 {
                continue;
            }
            let (u0, v0) = (u.floor(), v.floor());
            let (fu, fv) = (u - u0, v - v0);
            let mut acc = [0.0f32; 4];
            for (ox, oy, wgt) in
                [(0, 0, (1.0 - fu) * (1.0 - fv)), (1, 0, fu * (1.0 - fv)), (0, 1, (1.0 - fu) * fv), (1, 1, fu * fv)]
            {
                let (px, py) = (u0 as i64 + ox, v0 as i64 + oy);
                if px < 0 || py < 0 || px >= cv.w as i64 || py >= cv.h as i64 {
                    continue;
                }
                let p = cv.px[py as usize * cv.w + px as usize];
                for k in 0..4 {
                    acc[k] += p[k] * wgt;
                }
            }
            out.px[y * n + x] = acc;
        }
    }
    out
}
