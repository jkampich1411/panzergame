//! Tastenbelegung -> (Tasten-Bits, Schuss-Zähler)

use macroquad::prelude::*;
use panzer_core::consts::*;

pub struct KeyInput {
    up: Vec<KeyCode>,
    down: Vec<KeyCode>,
    left: Vec<KeyCode>,
    right: Vec<KeyCode>,
    fire: Vec<KeyCode>,
    /// Tasten für die Sprüche 1..n
    taunts: Vec<KeyCode>,
    pub fires: u16,
}

impl KeyInput {
    pub fn new(
        up: &[KeyCode],
        down: &[KeyCode],
        left: &[KeyCode],
        right: &[KeyCode],
        fire: &[KeyCode],
        taunts: &[KeyCode],
    ) -> KeyInput {
        KeyInput {
            up: up.to_vec(),
            down: down.to_vec(),
            left: left.to_vec(),
            right: right.to_vec(),
            fire: fire.to_vec(),
            taunts: taunts.to_vec(),
            fires: 0,
        }
    }

    /// Steuerung im Netzspiel und solo: Pfeile/WASD, Leertaste, Sprüche 1–7
    pub fn net() -> KeyInput {
        use KeyCode::*;
        KeyInput::new(
            &[Up, W],
            &[Down, S],
            &[Left, A],
            &[Right, D],
            &[Space, M, Q],
            &[Key1, Key2, Key3, Key4, Key5, Key6, Key7],
        )
    }

    /// Lokal: bis zu 4 Spieler an einer Tastatur
    pub fn local(n: usize) -> Vec<KeyInput> {
        use KeyCode::*;
        let all = vec![
            KeyInput::new(&[E], &[D], &[S], &[F], &[Q], &[]),
            KeyInput::new(&[Up], &[Down], &[Left], &[Right], &[M], &[]),
            KeyInput::new(&[I], &[K], &[J], &[L], &[U], &[]),
            KeyInput::new(&[Kp8], &[Kp5], &[Kp4], &[Kp6], &[Kp0, KpEnter], &[]),
        ];
        all.into_iter().take(n).collect()
    }

    /// (Bits, Zähler). enabled = false: alles losgelassen (z. B. Chat offen)
    pub fn state(&mut self, enabled: bool) -> (u8, u16) {
        if !enabled {
            return (0, self.fires);
        }
        if self.fire.iter().any(|&k| is_key_pressed(k)) {
            self.fires = self.fires.wrapping_add(1);
        }
        let down = |keys: &[KeyCode]| keys.iter().any(|&k| is_key_down(k));
        let mut bits = 0;
        if down(&self.up) {
            bits |= UP;
        }
        if down(&self.down) {
            bits |= DOWN;
        }
        if down(&self.left) {
            bits |= LEFT;
        }
        if down(&self.right) {
            bits |= RIGHT;
        }
        if down(&self.fire) {
            bits |= SHOOT;
        }
        for (n, &k) in self.taunts.iter().enumerate() {
            if is_key_down(k) {
                bits |= ((n + 1) as u8) << TAUNT_SHIFT;
                break;
            }
        }
        (bits, self.fires)
    }
}
