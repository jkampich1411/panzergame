//! Snapshot-Typen. NetSnap ist die quantisierte Form, die übers Netz geht (ganze Zahlen,
//! damit Server und Client beim Delta-Vergleich exakt dasselbe sehen). View ist die
//! entpackte Form mit Gleitkommazahlen, die der Renderer zeichnet (online wie lokal).

use crate::consts::*;
use crate::geom::lerp_angle;
use crate::items::{Eff, Item, Kind, ALL_EFFS, EFF_N};
use crate::physics::Effects;

/// Große Ansagen in der Bildmitte
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ann {
    Multi(u8),
    Trick(u8),
    FirstBlood,
    Disco,
    Meteors,
}

/// Kleine Texte über einem Panzer
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopText {
    OwnGoal,
    Spare,
}

pub type P16 = (i16, i16);

/// Effekte, die der Server meldet (Daten schon gerundet wie früher in der JSON-Version)
#[derive(Clone, Debug, PartialEq)]
pub enum FxData {
    /// size in Hundertsteln
    Boom {
        x: i16,
        y: i16,
        size: u8,
    },
    /// angle in Hundertsteln rad
    Die {
        x: i16,
        y: i16,
        slot: u8,
        angle: i16,
    },
    Laser {
        slot: u8,
        pts: Vec<P16>,
    },
    Zap {
        slot: u8,
        pts: Vec<P16>,
    },
    Tele {
        x: i16,
        y: i16,
    },
    Pick {
        x: i16,
        y: i16,
        item: Item,
        mystery: bool,
    },
    Block {
        x: i16,
        y: i16,
    },
    Freeze {
        x: i16,
        y: i16,
    },
    Nuke {
        x: i16,
        y: i16,
    },
    Ann {
        ann: Ann,
        slot: u8,
    },
    Pop {
        x: i16,
        y: i16,
        text: PopText,
        slot: u8,
    },
    Rail {
        slot: u8,
        p0: P16,
        p1: P16,
        holes: Vec<P16>,
    },
    Prism {
        slot: u8,
        beams: Vec<Vec<P16>>,
    },
    Push {
        slot: u8,
        x: i16,
        y: i16,
        a: i16,
    },
    Frost {
        x: i16,
        y: i16,
    },
    Jump {
        x0: i16,
        y0: i16,
        x1: i16,
        y1: i16,
    },
    Disco {
        x: i16,
        y: i16,
    },
    Mirror {
        x: i16,
        y: i16,
    },
    Build {
        x: i16,
        y: i16,
    },
    Trap {
        x: i16,
        y: i16,
    },
    Fw {
        x: i16,
        y: i16,
    },
}

#[inline]
pub fn q16(v: f64) -> i16 {
    v.round().clamp(-32768.0, 32767.0) as i16
}

#[inline]
pub fn qp(x: f64, y: f64) -> P16 {
    (q16(x), q16(y))
}

// ---------------------------------------------------------------------------
// Quantisierte Netzform

pub const T_ALIVE: u8 = 1;
/// Position wird nicht verraten (Tarnkappe ohne Radar, außerhalb vom Nebel)
pub const T_HIDDEN: u8 = 2;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct NetTank {
    pub slot: u8,
    pub flags: u8,
    /// Viertelpixel
    pub x: u16,
    pub y: u16,
    /// 65536 = 2 pi
    pub a: u16,
    /// Echtzeit (ms) des Abschusses
    pub died: u32,
    /// 255 = keine
    pub weapon: u8,
    /// Spielzeit (Zehntelsekunden), bis zu der der Schild hält, 0 = keiner
    pub shield: u32,
    pub ammo: u8,
    /// Zehntelsekunden, bis wieder eine Kugel frei ist
    pub reload: u8,
    pub eff_mask: u32,
    /// Ablauf je aktivem Effekt (Spielzeit in Zehntelsekunden), in Reihenfolge der Bits
    pub eff_exp: Vec<u32>,
    /// Railgun-Ladung in Hundertstelsekunden
    pub charge: u8,
    pub taunt: u8,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct NetShot {
    pub id: u32,
    pub kind: u8,
    pub owner: u8,
    pub x: u16,
    pub y: u16,
    pub ang: u16,
    /// Spielzeit (ms), zu der das Geschoss "geboren" ist (Alter = jetzt - born)
    pub born: u32,
    /// Lebensdauer in ms (u32::MAX = ewig)
    pub life: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetBox {
    pub x: u16,
    pub y: u16,
    pub item: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetFx {
    pub id: u32,
    /// Spielzeit (ms)
    pub born: u32,
    pub life: u16,
    pub data: FxData,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetFeed {
    /// 255 = niemand (Umwelt)
    pub by: u8,
    pub victim: u8,
    pub how: u8,
    pub variant: u8,
    /// Echtzeit (ms)
    pub born: u32,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct NetWin {
    /// 255 = unentschieden
    pub slot: u8,
    pub streak: u8,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct NetSnap {
    /// Echtzeit des Servers (ms)
    pub time: u32,
    /// Spielzeit (ms, läuft in der Zeitlupe langsamer)
    pub gtime: u32,
    /// zuletzt abgearbeitete Eingabe des Empfängers
    pub ack: u32,
    pub maze_id: u32,
    pub maze_rev: u16,
    /// Zeitfaktor * 255
    pub ts: u8,
    /// Rest der Rundenend-Zeit (ms), u16::MAX = keine
    pub et: u16,
    pub win: Option<NetWin>,
    pub scores: [u16; MAX_PLAYERS],
    pub bots: u8,
    /// Empfänger wartet auf die nächste Runde (schaut nur zu)
    pub queued: bool,
    pub tanks: Vec<NetTank>,
    pub shots: Vec<NetShot>,
    pub boxes: Vec<NetBox>,
    pub fx: Vec<NetFx>,
    pub feed: Vec<NetFeed>,
}

// ---------------------------------------------------------------------------
// Entpackte Form für den Renderer

#[derive(Clone, Debug)]
pub struct TankView {
    pub slot: usize,
    pub x: f64,
    pub y: f64,
    pub a: f64,
    pub alive: bool,
    pub hidden: bool,
    pub dead_age: f64,
    pub weapon: Option<Item>,
    pub shield: f64,
    pub ammo: i32,
    pub reload: f64,
    /// Restzeiten
    pub eff: Effects,
    pub charge: f64,
    pub taunt: u8,
}

#[derive(Clone, Debug)]
pub struct ShotView {
    pub id: u32,
    pub kind: Kind,
    pub owner: usize,
    pub x: f64,
    pub y: f64,
    pub ang: f64,
    pub age: f64,
    pub life: f64,
}

#[derive(Clone, Debug)]
pub struct BoxView {
    pub x: f64,
    pub y: f64,
    pub item: Item,
}

#[derive(Clone, Debug)]
pub struct FxView {
    pub id: u32,
    pub age: f64,
    pub life: f64,
    pub data: FxData,
}

#[derive(Clone, Debug)]
pub struct FeedView {
    pub by: Option<usize>,
    pub victim: usize,
    pub how: Kind,
    pub variant: u8,
    pub age: f64,
}

#[derive(Clone, Debug)]
pub struct WinView {
    pub slot: Option<usize>,
    pub streak: u32,
}

#[derive(Clone, Debug)]
pub struct View {
    /// Serverzeit in Sekunden
    pub time: f64,
    pub gtime: f64,
    pub ack: u32,
    pub maze_id: u32,
    pub maze_rev: u16,
    pub ts: f64,
    pub et: Option<f64>,
    pub win: Option<WinView>,
    pub scores: [u32; MAX_PLAYERS],
    pub bots: u8,
    pub queued: bool,
    pub tanks: Vec<TankView>,
    pub shots: Vec<ShotView>,
    pub boxes: Vec<BoxView>,
    pub fx: Vec<FxView>,
    pub feed: Vec<FeedView>,
}

impl NetTank {
    pub fn alive(&self) -> bool {
        self.flags & T_ALIVE != 0
    }
}

impl NetSnap {
    /// Entpacken zum Zeichnen
    pub fn view(&self) -> View {
        let time = self.time as f64 / 1000.0;
        let gtime = self.gtime as f64 / 1000.0;
        let gdeci = self.gtime as f64 / 100.0;
        let tanks = self
            .tanks
            .iter()
            .map(|t| {
                let mut eff = Effects::default();
                let mut k = 0;
                for (i, e) in ALL_EFFS.iter().enumerate().take(EFF_N) {
                    if t.eff_mask & (1 << i) != 0 {
                        let exp = t.eff_exp.get(k).copied().unwrap_or(0) as f64;
                        // aktiv heißt > 0, auch wenn die gerundete Restzeit schon 0 ist
                        eff.set(*e, ((exp - gdeci) / 10.0).max(0.001));
                        k += 1;
                    }
                }
                TankView {
                    slot: t.slot as usize,
                    x: t.x as f64 / 4.0,
                    y: t.y as f64 / 4.0,
                    a: t.a as f64 / 65536.0 * std::f64::consts::TAU,
                    alive: t.alive(),
                    hidden: t.flags & T_HIDDEN != 0,
                    dead_age: if t.alive() { 0.0 } else { (self.time.saturating_sub(t.died)) as f64 / 1000.0 },
                    weapon: if t.weapon == 255 { None } else { Item::from_u8(t.weapon) },
                    shield: if t.shield == 0 { 0.0 } else { ((t.shield as f64 - gdeci) / 10.0).max(0.0) },
                    ammo: t.ammo as i32,
                    reload: t.reload as f64 / 10.0,
                    eff,
                    charge: t.charge as f64 / 100.0,
                    taunt: t.taunt,
                }
            })
            .collect();
        let shots = self
            .shots
            .iter()
            .filter_map(|s| {
                Some(ShotView {
                    id: s.id,
                    kind: Kind::from_u8(s.kind)?,
                    owner: (s.owner as usize).min(MAX_PLAYERS - 1),
                    x: s.x as f64 / 4.0,
                    y: s.y as f64 / 4.0,
                    ang: s.ang as f64 / 65536.0 * std::f64::consts::TAU,
                    age: (self.gtime.wrapping_sub(s.born) as i32).max(0) as f64 / 1000.0,
                    life: if s.life == u32::MAX { 1e9 } else { s.life as f64 / 1000.0 },
                })
            })
            .collect();
        View {
            time,
            gtime,
            ack: self.ack,
            maze_id: self.maze_id,
            maze_rev: self.maze_rev,
            ts: self.ts as f64 / 255.0,
            et: if self.et == u16::MAX { None } else { Some(self.et as f64 / 1000.0) },
            win: self.win.as_ref().map(|w| WinView {
                slot: if w.slot == 255 { None } else { Some(w.slot as usize) },
                streak: w.streak as u32,
            }),
            scores: self.scores.map(|s| s as u32),
            bots: self.bots,
            queued: self.queued,
            tanks,
            shots,
            boxes: self
                .boxes
                .iter()
                .filter_map(|b| Some(BoxView { x: b.x as f64, y: b.y as f64, item: Item::from_u8(b.item)? }))
                .collect(),
            fx: self
                .fx
                .iter()
                .map(|f| FxView {
                    id: f.id,
                    age: (self.gtime.wrapping_sub(f.born) as i32).max(0) as f64 / 1000.0,
                    life: f.life as f64 / 1000.0,
                    data: f.data.clone(),
                })
                .collect(),
            feed: self
                .feed
                .iter()
                .filter_map(|f| {
                    Some(FeedView {
                        by: if f.by == 255 { None } else { Some(f.by as usize) },
                        victim: f.victim as usize,
                        how: Kind::from_u8(f.how)?,
                        variant: f.variant,
                        age: (self.time.wrapping_sub(f.born) as i32).max(0) as f64 / 1000.0,
                    })
                })
                .collect(),
        }
    }
}

impl View {
    pub fn tank(&self, slot: usize) -> Option<&TankView> {
        self.tanks.iter().find(|t| t.slot == slot)
    }

    pub fn tank_mut(&mut self, slot: usize) -> Option<&mut TankView> {
        self.tanks.iter_mut().find(|t| t.slot == slot)
    }

    /// Runde entschieden (Banner mit Sieger)
    pub fn decided(&self) -> bool {
        self.win.is_some()
    }

    pub fn is_bot(&self, slot: usize) -> bool {
        self.bots & (1 << slot) != 0
    }

    /// Spielzustand zwischen zwei Snapshots (render_t = Serverzeit). Positionen werden
    /// interpoliert, alles andere kommt aus dem neueren.
    pub fn interpolate(a: &View, b: &View, render_t: f64) -> View {
        if a.maze_id != b.maze_id || b.time <= a.time {
            return b.clone();
        }
        let k = ((render_t - a.time) / (b.time - a.time)).clamp(0.0, 1.0);
        let mut out = b.clone();
        out.gtime = a.gtime + (b.gtime - a.gtime) * k;
        out.time = a.time + (b.time - a.time) * k;
        for t in out.tanks.iter_mut() {
            if let Some(ta) = a.tank(t.slot) {
                if ta.hidden || t.hidden || ta.alive != t.alive {
                    continue;
                }
                t.x = ta.x + (t.x - ta.x) * k;
                t.y = ta.y + (t.y - ta.y) * k;
                t.a = lerp_angle(ta.a, t.a, k);
            }
        }
        for s in out.shots.iter_mut() {
            if let Some(q) = a.shots.iter().find(|q| q.id == s.id) {
                s.x = q.x + (s.x - q.x) * k;
                s.y = q.y + (s.y - q.y) * k;
                s.ang = lerp_angle(q.ang, s.ang, k);
                s.age = q.age + (s.age - q.age) * k;
            }
        }
        for f in out.fx.iter_mut() {
            if let Some(q) = a.fx.iter().find(|q| q.id == f.id) {
                f.age = q.age + (f.age - q.age) * k;
            }
        }
        out
    }
}

/// Effekt-Liste in Bit-Reihenfolge (für Debug/HUD)
pub fn active_effects(eff: &Effects) -> impl Iterator<Item = (Eff, f64)> + '_ {
    ALL_EFFS.iter().filter(|e| eff.has(**e)).map(|e| (*e, eff.get(*e)))
}
