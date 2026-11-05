//! Labyrinth: zufällig per DFS erzeugt und danach aufgebrochen (offener wie im Original).

use crate::consts::*;
use crate::geom::Wall;
use crate::rng::Rng;

#[derive(Clone, Debug, PartialEq)]
pub struct Maze {
    pub id: u32,
    /// steigt, wenn sich mitten in der Runde Wände ändern (Atombombe, Wandbauer)
    pub rev: u16,
    pub cols: usize,
    pub rows: usize,
    pub ox: i32,
    pub oy: i32,
    /// Weltgröße, gleiches Seitenverhältnis wie das Spielfeld
    pub ww: i32,
    pub wh: i32,
    /// h[r][c]: Wand oberhalb von Zelle (r, c), rows + 1 Zeilen
    pub h: Vec<Vec<bool>>,
    /// v[r][c]: Wand links von Zelle (r, c), cols + 1 Spalten
    pub v: Vec<Vec<bool>>,
    /// gebaute Wände (Wandbauer)
    pub extra: Vec<[i32; 4]>,
    pub mutator: Option<Mutator>,
    pub theme: u8,
}

/// Zellen (cols, rows). Bis 4 Spieler wie gehabt, danach pro Spieler deutlich größer.
pub fn maze_size(players: usize, rng: &mut Rng) -> (usize, usize) {
    if players < BIG_FROM {
        return (rng.randint(6, 10) as usize, rng.randint(4, 6) as usize);
    }
    let extra = (players - BIG_FROM + 1) as i64;
    let cols = rng.randint(11 + 2 * extra, 13 + 2 * extra);
    let rows = ((cols as f64 / 1.75).round() as i64 + rng.randint(-1, 0)).max(6);
    (cols as usize, rows as usize)
}

impl Maze {
    pub fn generate(id: u32, players: usize, rng: &mut Rng) -> Maze {
        let (cols, rows) = maze_size(players, rng);
        let mut h = vec![vec![true; cols]; rows + 1];
        let mut v = vec![vec![true; cols + 1]; rows];

        // Perfektes Labyrinth per DFS ...
        let mut seen = vec![vec![false; cols]; rows];
        let start = (rng.below(rows), rng.below(cols));
        seen[start.0][start.1] = true;
        let mut stack = vec![start];
        while let Some(&(r, c)) = stack.last() {
            let mut nbrs: Vec<(usize, usize, i32, i32)> = Vec::with_capacity(4);
            for (dr, dc) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                let (nr, nc) = (r as i32 + dr, c as i32 + dc);
                if nr >= 0 && nr < rows as i32 && nc >= 0 && nc < cols as i32 {
                    if !seen[nr as usize][nc as usize] {
                        nbrs.push((nr as usize, nc as usize, dr, dc));
                    }
                }
            }
            if nbrs.is_empty() {
                stack.pop();
                continue;
            }
            let (nr, nc, dr, dc) = nbrs[rng.below(nbrs.len())];
            match (dr, dc) {
                (-1, _) => h[r][c] = false,
                (1, _) => h[r + 1][c] = false,
                (_, -1) => v[r][c] = false,
                _ => v[r][c + 1] = false,
            }
            seen[nr][nc] = true;
            stack.push((nr, nc));
        }

        // ... dann aufbrechen, damit es offener wird wie im Original
        for row in h.iter_mut().take(rows).skip(1) {
            for cell in row.iter_mut() {
                if *cell && rng.random() < 0.4 {
                    *cell = false;
                }
            }
        }
        for row in v.iter_mut() {
            for cell in row.iter_mut().take(cols).skip(1) {
                if *cell && rng.random() < 0.4 {
                    *cell = false;
                }
            }
        }

        let (ox, oy, ww, wh) = layout(cols, rows);
        Maze { id, rev: 0, cols, rows, ox, oy, ww, wh, h, v, extra: Vec::new(), mutator: None, theme: 0 }
    }

    /// Alle Wandrechtecke: Raster (gemergt) plus gebaute Wände
    pub fn walls(&self) -> Vec<Wall> {
        let mut walls = merge_walls(&self.h, &self.v, self.ox, self.oy);
        walls.extend(self.extra.iter().map(|e| [e[0] as f64, e[1] as f64, e[2] as f64, e[3] as f64]));
        walls
    }

    pub fn scale(&self) -> f64 {
        W / self.ww as f64
    }
}

/// Passt das Labyrinth nicht ins Fenster, wird die Welt größer und beim Zeichnen
/// verkleinert. (ox, oy, ww, wh)
pub fn layout(cols: usize, rows: usize) -> (i32, i32, i32, i32) {
    let scale = 1.0f64.min(W / (cols as f64 * CELL + 60.0)).min(ARENA_H / (rows as f64 * CELL + 20.0));
    let ww = (W / scale).round() as i32;
    let wh = (VIEW_H / scale).round() as i32;
    let ox = (ww - (cols as f64 * CELL) as i32).div_euclid(2);
    let oy = (ARENA_TOP / scale).round() as i32
        + ((ARENA_H / scale).round() as i32 - (rows as f64 * CELL) as i32).div_euclid(2);
    (ox, oy, ww, wh)
}

/// Wandstücke aus dem Raster als Rechtecke, zusammenhängende Stücke gemergt.
pub fn merge_walls(h: &[Vec<bool>], v: &[Vec<bool>], ox: i32, oy: i32) -> Vec<Wall> {
    let rows = v.len();
    let cols = h[0].len();
    let half = (WALL / 2.0) as i32;
    let cell = CELL as i32;
    let wall = WALL as i32;
    let mut walls = Vec::new();
    for (r, row) in h.iter().enumerate().take(rows + 1) {
        let mut c = 0;
        while c < cols {
            if row[c] {
                let start = c;
                while c < cols && row[c] {
                    c += 1;
                }
                walls.push([
                    (ox + start as i32 * cell - half) as f64,
                    (oy + r as i32 * cell - half) as f64,
                    ((c - start) as i32 * cell + wall) as f64,
                    wall as f64,
                ]);
            } else {
                c += 1;
            }
        }
    }
    for c in 0..=cols {
        let mut r = 0;
        while r < rows {
            if v[r][c] {
                let start = r;
                while r < rows && v[r][c] {
                    r += 1;
                }
                walls.push([
                    (ox + c as i32 * cell - half) as f64,
                    (oy + start as i32 * cell - half) as f64,
                    wall as f64,
                    ((r - start) as i32 * cell + wall) as f64,
                ]);
            } else {
                r += 1;
            }
        }
    }
    walls
}
