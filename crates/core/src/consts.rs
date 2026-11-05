//! Alle Spielkonstanten an einer Stelle (Werte wie in der Python-Version).

use std::f64::consts::PI;

pub const PORT: u16 = 5555;
/// Protokollversion, Server und Client müssen übereinstimmen
pub const PROTO: u16 = 8;
pub const MAX_PLAYERS: usize = 8;
/// mehr passen nicht sinnvoll auf eine Tastatur
pub const MAX_LOCAL: usize = 4;
/// ab so vielen Spielern wird das Labyrinth größer
pub const BIG_FROM: usize = 5;
pub const W: f64 = 1000.0;
pub const H: f64 = 680.0;
pub const ARENA_TOP: f64 = 15.0;
pub const ARENA_H: f64 = 560.0;
/// Höhe des Spielfelds auf dem Bildschirm (darunter HUD)
pub const VIEW_H: f64 = ARENA_TOP + ARENA_H + 10.0;
pub const TICK: f64 = 1.0 / 60.0;
/// Snapshots pro Sekunde vom Server
pub const SNAP_RATE: u32 = 60;
/// so weit in der Vergangenheit rendert der Client die anderen
pub const INTERP_DELAY: f64 = 0.075;

pub const CELL: f64 = 90.0;
pub const WALL: f64 = 8.0;
pub const TANK_R: f64 = 15.0;
pub const TANK_SPEED: f64 = 130.0;
pub const TANK_ROT: f64 = 200.0 * PI / 180.0;
/// Abstand Mitte -> Rohrende
pub const MUZZLE: f64 = 20.0;
pub const BULLET_R: f64 = 4.0;
pub const BULLET_SPEED: f64 = 220.0;
pub const BULLET_LIFE: f64 = 10.0;
pub const MAX_BULLETS: usize = 5;
pub const FIRE_COOLDOWN: f64 = 0.12;
/// so lange kann man sich nicht selbst treffen
pub const OWNER_GRACE: f64 = 0.1;
pub const ROUND_END_DELAY: f64 = 3.0;
/// so lange ist man nach dem Erscheinen unverwundbar
pub const SPAWN_SAFE: f64 = 2.0;

// Power-ups
pub const BOX_FIRST: f64 = 4.0;
pub const BOX_EVERY: (f64, f64) = (6.0, 11.0);
pub const BOX_MAX: usize = 3;

pub const FRAG_R: f64 = 7.0;
pub const FRAG_SPEED: f64 = 170.0;
pub const FRAG_FUSE: f64 = 5.0;
pub const FRAG_PIECES: usize = 26;
pub const MG_RATE: f64 = 0.07;
pub const MG_SPREAD: f64 = 0.12;
pub const MG_R: f64 = 3.0;
pub const MG_LIFE: f64 = 4.0;
pub const LASER_LEN: f64 = 1400.0;
pub const LASER_SHOW: f64 = 0.45;
pub const ROCKET_R: f64 = 5.0;
pub const ROCKET_SPEED: f64 = 170.0;
pub const ROCKET_TURN: f64 = 4.0;
pub const ROCKET_LIFE: f64 = 10.0;
pub const ROCKET_DELAY: f64 = 0.4;
pub const ROCKET_GRACE: f64 = 0.8;
pub const SMOKE_LIFE: f64 = 0.7;
pub const SHIELD_TIME: f64 = 8.0;
pub const SG_PELLETS: usize = 7;
pub const SG_SPREAD: f64 = 0.32;
pub const SG_LIFE: f64 = 1.1;
pub const BOUNCY_R: f64 = 6.0;
pub const BOUNCY_SPEED: f64 = 150.0;
pub const BOUNCY_GAIN: f64 = 1.15;
pub const BOUNCY_MAX: f64 = 560.0;
pub const BOUNCY_LIFE: f64 = 7.0;
pub const MINE_R: f64 = 7.0;
pub const MINE_ARM: f64 = 1.0;
pub const MINE_BLAST: f64 = 44.0;
pub const SWAP_R: f64 = 6.0;
pub const SWAP_SPEED: f64 = 300.0;
pub const SWAP_LIFE: f64 = 2.5;
pub const RANG_R: f64 = 6.0;
pub const RANG_SPEED: f64 = 250.0;
pub const RANG_CURVE: f64 = 2.0;
pub const RANG_TURN: f64 = 5.0;
pub const RANG_BACK: f64 = 0.8;
pub const RANG_LIFE: f64 = 5.0;
pub const ICE_R: f64 = 5.0;
pub const ICE_SPEED: f64 = 290.0;
pub const ICE_LIFE: f64 = 3.0;
pub const FREEZE_TIME: f64 = 3.0;
pub const HOLE_R: f64 = 7.0;
pub const HOLE_SPEED: f64 = 130.0;
pub const HOLE_TRAVEL: f64 = 1.2;
pub const HOLE_TIME: f64 = 4.5;
pub const HOLE_RANGE: f64 = 210.0;
pub const HOLE_PULL: f64 = 175.0;
pub const HOLE_KILL: f64 = 16.0;
pub const HOLE_SHOT_PULL: f64 = 650.0;
pub const STRIKE_DIST: f64 = 280.0;
pub const STRIKE_DELAY: f64 = 1.5;
pub const STRIKE_BLAST: f64 = 62.0;
// Flammenwerfer: Feuer halten, jede Flamme wird schnell langsamer und verglüht
pub const FLAME_RATE: f64 = 0.03;
pub const FLAME_SPREAD: f64 = 0.16;
pub const FLAME_SPEED: f64 = 340.0;
pub const FLAME_LIFE: f64 = 0.42;
pub const FLAME_R: f64 = 7.0;
pub const FLAME_DRAG: f64 = 2.2;
// Blitz: springt auf den nächsten Panzer vor einem und von dort weiter
pub const ZAP_RANGE: f64 = 250.0;
pub const ZAP_CHAIN: f64 = 200.0;
pub const ZAP_JUMPS: usize = 3;
pub const ZAP_CONE: f64 = 1.0;
pub const ZAP_SHOW: f64 = 0.45;
// Atombombe (nur aus der Wundertüte): rollt aus und geht nach NUKE_FUSE s hoch
pub const NUKE_R: f64 = 10.0;
pub const NUKE_SPEED: f64 = 150.0;
pub const NUKE_DRAG: f64 = 1.3;
pub const NUKE_FUSE: f64 = 4.0;
pub const NUKE_BLAST: f64 = 230.0;
pub const NUKE_CHANCE: f64 = 0.15;
// Railgun: Feuer halten zum Aufladen, loslassen feuert einen Strahl durch bis zu 2 Wände
pub const RAIL_CHARGE: f64 = 0.9;
pub const RAIL_LEN: f64 = 1500.0;
pub const RAIL_WALLS: usize = 2;
pub const RAIL_SHOW: f64 = 0.5;
// Fernlenkrakete: der Panzer steht, Links/Rechts lenken die Rakete, 2. Druck zündet
pub const RC_SPEED: f64 = 185.0;
pub const RC_TURN: f64 = 3.4;
pub const RC_LIFE: f64 = 6.0;
pub const RC_BLAST: f64 = 36.0;
pub const SENTRY_LIFE: f64 = 9.0;
pub const SENTRY_RANGE: f64 = 380.0;
pub const SENTRY_RATE: f64 = 0.55;
pub const SENTRY_TURN: f64 = 5.0;
pub const SENTRY_R: f64 = 10.0;
/// Wandbauer: Länge und Haltbarkeit der Wand
pub const BUILD_LEN: f64 = 66.0;
pub const BUILD_TIME: f64 = 10.0;
/// Öl brennt, wenn Feuer drankommt
pub const OIL_R: f64 = 34.0;
pub const OIL_LIFE: f64 = 14.0;
pub const OIL_BURN: f64 = 2.5;
pub const STICKY_SPEED: f64 = 260.0;
pub const STICKY_FUSE: f64 = 1.5;
pub const STICKY_BLAST: f64 = 40.0;
/// Granate fliegt über Wände
pub const GL_SPEED: f64 = 320.0;
pub const GL_DIST: f64 = 240.0;
pub const GL_BLAST: f64 = 38.0;
pub const SWARM_N: usize = 5;
pub const SWARM_SPEED: f64 = 210.0;
pub const SWARM_TURN: f64 = 3.0;
pub const SWARM_LIFE: f64 = 3.2;
pub const SAW_R: f64 = 9.0;
pub const SAW_SPEED: f64 = 175.0;
pub const SAW_BOUNCES: u32 = 7;
pub const SAW_LIFE: f64 = 8.0;
pub const PLASMA_R: f64 = 8.0;
pub const PLASMA_SPEED: f64 = 150.0;
pub const PLASMA_WALLS: u32 = 3;
pub const PLASMA_LIFE: f64 = 6.0;
pub const FW_SPEED: f64 = 260.0;
pub const FW_FUSE: f64 = 0.7;
pub const FW_SPARKS: usize = 16;
pub const PUSH_RANGE: f64 = 170.0;
pub const PUSH_CONE: f64 = 0.65;
pub const PUSH_DIST: f64 = 120.0;
pub const SPIDER_SENSE: f64 = 230.0;
pub const SPIDER_SPEED: f64 = 85.0;
pub const SPIDER_BLAST: f64 = 42.0;
pub const SPIDER_LIFE: f64 = 20.0;
pub const RAM_TIME: f64 = 4.0;
pub const NOVA_N: usize = 12;
pub const SEEK_SPEED: f64 = 200.0;
pub const SEEK_TURN: f64 = 2.4;
pub const SEEK_LIFE: f64 = 6.0;
pub const PHASE_SPEED: f64 = 300.0;
pub const PHASE_LIFE: f64 = 2.2;
pub const CANNON_R: f64 = 11.0;
pub const CANNON_SPEED: f64 = 160.0;
pub const CANNON_BOUNCES: u32 = 2;
pub const ARTY_DIST: [f64; 3] = [110.0, 190.0, 270.0];
pub const ARTY_BLAST: f64 = 44.0;
pub const TRAP_TIME: f64 = 4.0;
pub const JUMP_DIST: f64 = 95.0;
pub const FROST_R: f64 = 130.0;
pub const ORBIT_DIST: f64 = 240.0;
pub const ORBIT_DELAY: f64 = 1.6;
pub const ORBIT_TIME: f64 = 1.6;
pub const ORBIT_SPEED: f64 = 120.0;
pub const ORBIT_R: f64 = 24.0;
pub const PRISM_SPLIT: f64 = 0.42;
pub const PRISM_LEN: f64 = 900.0;
pub const METEOR_N: usize = 9;
pub const METEOR_BLAST: f64 = 42.0;
pub const GOLD_SPEED: f64 = 330.0;
pub const GOLD_LIFE: f64 = 6.0;
pub const GIANT_SCALE: f64 = 1.5;
pub const TURBO_SPEED: f64 = 1.65;
pub const MINI_SCALE: f64 = 0.55;
pub const MINI_SPEED: f64 = 1.15;
pub const POTATO_TIME: f64 = 10.0;
pub const POTATO_MIN: f64 = 3.0;
pub const POTATO_BLAST: f64 = 55.0;
pub const POTATO_CHANCE: f64 = 0.25;
// Zeitlupe, sobald die Runde entschieden ist (Panzer fahren normal, alles andere langsam)
pub const SLOWMO_TIME: f64 = 1.5;
pub const SLOWMO_SCALE: f64 = 0.25;
// Ansagen: Mehrfachkill innerhalb dieser Zeit, Trickshot ab so vielen Abprallern
pub const MULTI_WINDOW: f64 = 1.6;
pub const TRICK_BOUNCES: u32 = 3;
pub const ANN_TIME: f64 = 1.9;
pub const MUTATOR_CHANCE: f64 = 0.35;
pub const FOG_R: f64 = 165.0;
pub const TAUNT_TIME: f64 = 2.5;
pub const FEED_TIME: f64 = 5.0;
pub const FEED_MAX: usize = 4;
/// Abtastschritt, mit dem ein Strahl die Wände seiner Zellen sammelt
pub const RAY_STEP: f64 = 20.0;

// Tasten-Bits. Nicht FIRE nennen: so heißt auch die Partikelart Feuer
pub const UP: u8 = 1;
pub const DOWN: u8 = 2;
pub const LEFT: u8 = 4;
pub const RIGHT: u8 = 8;
pub const SHOOT: u8 = 16;
/// Bits 5-7 der Tasten: Nummer des Spruchs (1-7), 0 = keiner
pub const TAUNT_SHIFT: u8 = 5;

pub type Rgb = [u8; 3];

pub const COLORS: [Rgb; MAX_PLAYERS] = [
    [220, 35, 35],
    [35, 190, 45],
    [45, 95, 225],
    [240, 140, 15],
    [150, 60, 200],
    [20, 175, 185],
    [235, 85, 170],
    [130, 85, 40],
];
/// Farbnamen: für lokale Spieler und Bots
pub const COLOR_NAMES: [&str; MAX_PLAYERS] = ["Rot", "Grün", "Blau", "Orange", "Lila", "Türkis", "Pink", "Braun"];

pub const BG: Rgb = [11, 12, 17];
pub const TEXT_C: Rgb = [232, 236, 244];
pub const DIM_C: Rgb = [138, 146, 164];
/// dunkle Schrift auf hellem Grund
pub const INK_C: Rgb = [28, 30, 38];
pub const SHOT_C: Rgb = [30, 30, 30];
pub const ACCENT_C: Rgb = [255, 196, 60];

/// Sprüche über die Tasten 1–7
pub const TAUNTS: [&str; 7] = ["Hehe!", "Ups …", "GG", "Zu langsam!", "Komm her!", "JUlIAN SPRING", "SCHEIß JUNKY"];

/// Sonderregel einer Runde
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Mutator {
    Kisten,
    Hyper,
    Riesen,
    Turbo,
    Nebel,
    Sniper,
    Dunkel,
}

pub const MUTATORS: [Mutator; 7] = [
    Mutator::Kisten,
    Mutator::Hyper,
    Mutator::Riesen,
    Mutator::Turbo,
    Mutator::Nebel,
    Mutator::Sniper,
    Mutator::Dunkel,
];

impl Mutator {
    pub fn from_u8(v: u8) -> Option<Mutator> {
        MUTATORS.get(v as usize).copied()
    }

    pub fn name(self) -> &'static str {
        match self {
            Mutator::Kisten => "Kistenregen",
            Mutator::Hyper => "Hyperkugeln",
            Mutator::Riesen => "Riesenkugeln",
            Mutator::Turbo => "Alle auf Turbo",
            Mutator::Nebel => "Nebel",
            Mutator::Sniper => "Scharfschützen",
            Mutator::Dunkel => "Stromausfall",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            Mutator::Kisten => "Power-ups ohne Ende",
            Mutator::Hyper => "Kugeln fast doppelt so schnell",
            Mutator::Riesen => "dicke, langsame Kugeln",
            Mutator::Turbo => "alle Panzer fahren schneller",
            Mutator::Nebel => "man sieht nur, was direkt vor einem liegt",
            Mutator::Sniper => "nur 1 Kugel, aber pfeilschnell",
            Mutator::Dunkel => "Licht gibt es nur von Scheinwerfern und Feuer",
        }
    }
}

/// Umgebungspartikel je Look
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ambient {
    Dust,
    Sand,
    Ember,
    Snow,
    Firefly,
    Neon,
}

/// Arenen-Look: der Server würfelt pro Runde einen aus (rein optisch)
pub struct Theme {
    pub name: &'static str,
    pub floor: Rgb,
    pub line: Rgb,
    pub wall: Rgb,
    pub hi: Rgb,
    pub lo: Rgb,
    pub track: Rgb,
    pub dust: Rgb,
    pub accent: Rgb,
    pub glow: Option<Rgb>,
    pub cracks: bool,
    pub amb: Ambient,
}

pub const THEMES: [Theme; 6] = [
    Theme {
        name: "Nachtbasis",
        floor: [34, 38, 49],
        line: [45, 51, 65],
        wall: [84, 96, 120],
        hi: [152, 170, 204],
        lo: [36, 42, 56],
        track: [25, 28, 37],
        dust: [128, 138, 160],
        accent: [110, 180, 255],
        glow: None,
        cracks: false,
        amb: Ambient::Dust,
    },
    Theme {
        name: "Wüste",
        floor: [70, 55, 40],
        line: [79, 63, 46],
        wall: [156, 122, 82],
        hi: [220, 186, 132],
        lo: [86, 64, 42],
        track: [54, 42, 30],
        dust: [196, 166, 120],
        accent: [255, 170, 80],
        glow: None,
        cracks: false,
        amb: Ambient::Sand,
    },
    Theme {
        name: "Vulkan",
        floor: [36, 27, 27],
        line: [47, 34, 33],
        wall: [72, 54, 50],
        hi: [150, 104, 84],
        lo: [30, 21, 21],
        track: [24, 17, 17],
        dust: [96, 74, 68],
        accent: [255, 110, 40],
        glow: Some([255, 80, 15]),
        cracks: true,
        amb: Ambient::Ember,
    },
    Theme {
        name: "Gletscher",
        floor: [30, 45, 62],
        line: [40, 58, 78],
        wall: [122, 172, 212],
        hi: [226, 244, 255],
        lo: [60, 96, 134],
        track: [22, 34, 48],
        dust: [205, 228, 248],
        accent: [150, 220, 255],
        glow: None,
        cracks: false,
        amb: Ambient::Snow,
    },
    Theme {
        name: "Dschungel",
        floor: [30, 43, 31],
        line: [38, 53, 38],
        wall: [72, 100, 60],
        hi: [132, 172, 104],
        lo: [34, 50, 29],
        track: [21, 31, 22],
        dust: [112, 144, 92],
        accent: [170, 240, 110],
        glow: None,
        cracks: false,
        amb: Ambient::Firefly,
    },
    Theme {
        name: "Neon-City",
        floor: [19, 16, 33],
        line: [52, 28, 82],
        wall: [34, 26, 64],
        hi: [90, 235, 255],
        lo: [255, 60, 200],
        track: [13, 11, 24],
        dust: [150, 100, 230],
        accent: [255, 80, 210],
        glow: Some([60, 150, 255]),
        cracks: false,
        amb: Ambient::Neon,
    },
];
