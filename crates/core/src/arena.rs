//! Labyrinth mit Broadphase-Gitter und Zell-Nachbarschaft. Läuft auf Server und Client.

use std::collections::VecDeque;

use crate::consts::*;
use crate::geom::{ray_box, ray_circle, Axis, Wall};
use crate::maze::Maze;

const MARGIN: f64 = 24.0;

pub type Cell = (usize, usize);

#[derive(Clone, Debug)]
pub struct Arena {
    pub id: u32,
    pub rev: u16,
    pub cols: usize,
    pub rows: usize,
    pub ox: f64,
    pub oy: f64,
    pub ww: f64,
    pub wh: f64,
    pub walls: Vec<Wall>,
    pub mutator: Option<Mutator>,
    pub theme: usize,
    /// pro Zelle die Indizes der Wände in der Nähe
    grid: Vec<Vec<u16>>,
    /// offen verbundene Nachbarzellen (für Lenkrakete und Bots)
    nbrs: Vec<Vec<Cell>>,
}

impl Arena {
    pub fn new(maze: &Maze) -> Arena {
        let walls = maze.walls();
        let (cols, rows) = (maze.cols, maze.rows);
        let (ox, oy) = (maze.ox as f64, maze.oy as f64);
        let mut grid = Vec::with_capacity(rows * cols);
        for r in 0..rows {
            for c in 0..cols {
                let x0 = ox + c as f64 * CELL - MARGIN;
                let y0 = oy + r as f64 * CELL - MARGIN;
                let x1 = x0 + CELL + 2.0 * MARGIN;
                let y1 = y0 + CELL + 2.0 * MARGIN;
                grid.push(
                    walls
                        .iter()
                        .enumerate()
                        .filter(|(_, w)| w[0] < x1 && w[0] + w[2] > x0 && w[1] < y1 && w[1] + w[3] > y0)
                        .map(|(i, _)| i as u16)
                        .collect(),
                );
            }
        }
        let mut nbrs = vec![Vec::new(); rows * cols];
        for r in 0..rows {
            for c in 0..cols {
                for (dr, dc) in [(0usize, 1usize), (1, 0)] {
                    let (r2, c2) = (r + dr, c + dc);
                    if r2 >= rows || c2 >= cols {
                        continue;
                    }
                    let (px, py) = if dc == 1 {
                        (ox + c2 as f64 * CELL, oy + r as f64 * CELL + CELL / 2.0)
                    } else {
                        (ox + c as f64 * CELL + CELL / 2.0, oy + r2 as f64 * CELL)
                    };
                    let blocked =
                        walls.iter().any(|w| w[0] <= px && px <= w[0] + w[2] && w[1] <= py && py <= w[1] + w[3]);
                    if !blocked {
                        nbrs[r * cols + c].push((r2, c2));
                        nbrs[r2 * cols + c2].push((r, c));
                    }
                }
            }
        }
        Arena {
            id: maze.id,
            rev: maze.rev,
            cols,
            rows,
            ox,
            oy,
            ww: maze.ww as f64,
            wh: maze.wh as f64,
            walls,
            mutator: maze.mutator,
            theme: maze.theme as usize % THEMES.len(),
            grid,
            nbrs,
        }
    }

    pub fn theme(&self) -> &'static Theme {
        &THEMES[self.theme]
    }

    pub fn cell(&self, x: f64, y: f64) -> Cell {
        let c = (((x - self.ox) / CELL).floor() as i64).clamp(0, self.cols as i64 - 1);
        let r = (((y - self.oy) / CELL).floor() as i64).clamp(0, self.rows as i64 - 1);
        (r as usize, c as usize)
    }

    pub fn cell_index(&self, cell: Cell) -> usize {
        cell.0 * self.cols + cell.1
    }

    pub fn cells(&self) -> Vec<Cell> {
        let mut v = Vec::with_capacity(self.rows * self.cols);
        for r in 0..self.rows {
            for c in 0..self.cols {
                v.push((r, c));
            }
        }
        v
    }

    /// Wände in der Nähe eines Punkts
    pub fn near(&self, x: f64, y: f64) -> impl Iterator<Item = &Wall> + Clone {
        let cell = self.cell(x, y);
        self.near_cell(cell)
    }

    pub fn near_cell(&self, cell: Cell) -> impl Iterator<Item = &Wall> + Clone {
        self.grid[self.cell_index(cell)].iter().map(move |&i| &self.walls[i as usize])
    }

    pub fn center(&self, cell: Cell) -> (f64, f64) {
        (self.ox + cell.1 as f64 * CELL + CELL / 2.0, self.oy + cell.0 as f64 * CELL + CELL / 2.0)
    }

    pub fn neighbours(&self, cell: Cell) -> &[Cell] {
        &self.nbrs[self.cell_index(cell)]
    }

    /// Kürzester Weg von start nach goal (inklusive beider), [start] wenn es keinen gibt.
    pub fn path(&self, start: Cell, goal: Cell) -> Vec<Cell> {
        let n = self.rows * self.cols;
        let mut prev: Vec<Option<usize>> = vec![None; n];
        let mut seen = vec![false; n];
        let si = self.cell_index(start);
        let gi = self.cell_index(goal);
        seen[si] = true;
        let mut q = VecDeque::from([start]);
        while let Some(cur) = q.pop_front() {
            if cur == goal {
                break;
            }
            for &nb in self.neighbours(cur) {
                let ni = self.cell_index(nb);
                if !seen[ni] {
                    seen[ni] = true;
                    prev[ni] = Some(self.cell_index(cur));
                    q.push_back(nb);
                }
            }
        }
        if !seen[gi] {
            return vec![start];
        }
        let mut out = vec![goal];
        let mut cur = gi;
        while let Some(p) = prev[cur] {
            out.push((p / self.cols, p % self.cols));
            cur = p;
        }
        out.reverse();
        out
    }

    /// Schritte von start zu jeder Zelle und Vorgänger (für die Wege der Bots).
    /// dist[i] = u32::MAX, wenn nicht erreichbar.
    pub fn bfs(&self, start: Cell) -> (Vec<u32>, Vec<Option<usize>>) {
        let n = self.rows * self.cols;
        let mut dist = vec![u32::MAX; n];
        let mut prev = vec![None; n];
        dist[self.cell_index(start)] = 0;
        let mut q = VecDeque::from([start]);
        while let Some(cur) = q.pop_front() {
            let ci = self.cell_index(cur);
            for &nb in self.neighbours(cur) {
                let ni = self.cell_index(nb);
                if dist[ni] == u32::MAX {
                    dist[ni] = dist[ci] + 1;
                    prev[ni] = Some(ci);
                    q.push_back(nb);
                }
            }
        }
        (dist, prev)
    }

    /// Erste Wand auf dem Strahl: (Strecke, Achse) bzw. (length, None).
    /// Sammelt nur die Wände der Zellen entlang des Strahls statt alle zu prüfen. Jeder
    /// Auftreffpunkt liegt höchstens RAY_STEP/2 neben einem Abtastpunkt, also im Rand
    /// (MARGIN) von dessen Zelle -> gleiches Ergebnis wie die Suche über alle Wände.
    pub fn first_wall_dir(&self, x: f64, y: f64, dx: f64, dy: f64, length: f64) -> (f64, Option<Axis>) {
        let mut best = length;
        let mut axis = None;
        let mut seen: [u16; 64] = [u16::MAX; 64];
        let mut n_seen = 0usize;
        let mut last: Option<Cell> = None;
        let mut d = 0.0;
        while d <= best + RAY_STEP {
            let cell = self.cell(x + dx * d, y + dy * d);
            if last != Some(cell) {
                last = Some(cell);
                for &wi in &self.grid[self.cell_index(cell)] {
                    if seen[..n_seen.min(64)].contains(&wi) {
                        continue;
                    }
                    if n_seen < 64 {
                        seen[n_seen] = wi;
                    }
                    n_seen += 1;
                    if let Some((t, ax)) = ray_box(x, y, dx, dy, &self.walls[wi as usize]) {
                        if t < best {
                            best = t;
                            axis = Some(ax);
                        }
                    }
                }
            }
            d += RAY_STEP;
        }
        (best, axis)
    }

    /// Freie Sicht zwischen zwei Punkten (keine Wand dazwischen)?
    pub fn clear(&self, x0: f64, y0: f64, x1: f64, y1: f64) -> bool {
        let d = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        if d < 1e-6 {
            return true;
        }
        self.first_wall_dir(x0, y0, (x1 - x0) / d, (y1 - y0) / d, d).1.is_none()
    }

    /// Abprallender Strahl. Gibt (Punkte, Index des getroffenen Kreises) zurück.
    pub fn raycast(
        &self,
        mut x: f64,
        mut y: f64,
        a: f64,
        mut length: f64,
        circles: &[(f64, f64, f64)],
    ) -> (Vec<(f64, f64)>, Option<usize>) {
        let (mut dx, mut dy) = (a.cos(), a.sin());
        let mut pts = vec![(x, y)];
        for _ in 0..40 {
            if length <= 0.5 {
                break;
            }
            let (t_wall, axis) = self.first_wall_dir(x, y, dx, dy, length);
            let mut best = t_wall;
            let mut hit = None;
            for (i, &(cx, cy, cr)) in circles.iter().enumerate() {
                if let Some(t) = ray_circle(x, y, dx, dy, cx, cy, cr) {
                    if t < best {
                        best = t;
                        hit = Some(i);
                    }
                }
            }
            if hit.is_some() {
                pts.push((x + dx * best, y + dy * best));
                return (pts, hit);
            }
            x += dx * t_wall;
            y += dy * t_wall;
            pts.push((x, y));
            length -= t_wall;
            let Some(axis) = axis else { break };
            if axis.has_x() {
                dx = -dx;
            }
            if axis.has_y() {
                dy = -dy;
            }
            x += dx * 0.01;
            y += dy * 0.01;
        }
        (pts, None)
    }

    /// Innenfläche des Labyrinths (x0, y0, x1, y1)
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        (self.ox, self.oy, self.ox + self.cols as f64 * CELL, self.oy + self.rows as f64 * CELL)
    }
}
