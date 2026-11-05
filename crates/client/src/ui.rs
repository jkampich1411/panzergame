//! Knöpfe, Textfelder, Chat und Spielerliste (Tab) mit Kick/Bann für den Host.

use macroquad::prelude::*;
use panzer_core::consts::*;

use crate::gfx::*;
use crate::net::NetClient;
use crate::raster::mix;
use crate::render::Roster;

pub fn hover(r: Rect, m: Vec2) -> bool {
    r.contains(m)
}

/// Knopf: dunkles Glas, beim Drüberfahren leuchtet der Rand. true = angeklickt.
pub fn button(g: &mut Gfx, r: Rect, label: &str, m: Vec2, accent: [u8; 3], size: f32) -> bool {
    let h = hover(r, m);
    let base = mix([30, 34, 46], [255, 255, 255], if h { 0.08 } else { 0.0 });
    draw_rectangle(r.x, r.y + 4.0, r.w, r.h, col([5, 6, 9]));
    draw_rectangle(r.x, r.y, r.w, r.h, col(base));
    let edge = if h { accent } else { mix(accent, [34, 38, 50], 0.55) };
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, col(edge));
    if h {
        g.light(accent, r.x + r.w / 2.0, r.y + r.h / 2.0, r.w * 0.42, 2.0);
        g.flush_light();
    }
    g.fancy(label, r.x + r.w / 2.0, r.y + r.h / 2.0, size, [255, 255, 255], 2.0, false, 1.0, 1.0);
    h && is_mouse_button_pressed(MouseButton::Left)
}

/// Einzeiliges Textfeld
pub struct Field {
    pub text: String,
    pub max: usize,
    pub focus: bool,
}

impl Field {
    pub fn new(text: &str, max: usize) -> Field {
        Field { text: text.to_string(), max, focus: false }
    }

    /// Tastatureingaben übernehmen (nur wenn fokussiert). allowed filtert Zeichen.
    pub fn typing(&mut self, allowed: impl Fn(char) -> bool) {
        if !self.focus {
            return;
        }
        while let Some(ch) = get_char_pressed() {
            if !ch.is_control() && allowed(ch) && self.text.chars().count() < self.max {
                self.text.push(ch);
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            self.text.pop();
        }
    }

    pub fn draw(&self, g: &Gfx, r: Rect, placeholder: &str, m: Vec2) {
        draw_rectangle(r.x, r.y, r.w, r.h, col([8, 9, 13]));
        let edge = if self.focus {
            ACCENT_C
        } else if hover(r, m) {
            [120, 128, 150]
        } else {
            [70, 78, 100]
        };
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, col(edge));
        let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
        if self.text.is_empty() && !self.focus {
            g.text(placeholder, cx, cy, 17.0, col([110, 116, 132]), 0.5);
        } else {
            let caret = if self.focus && (get_time() * 2.0) as i64 % 2 == 0 { "|" } else { " " };
            g.text(&format!("{}{caret}", self.text), cx, cy, 19.0, col(TEXT_C), 0.5);
        }
    }
}

/// Alle angestauten Zeichen verwerfen (sonst landen Spieltasten später im Chat)
pub fn drain_chars() {
    while get_char_pressed().is_some() {}
}

pub struct Chat {
    pub open: bool,
    pub field: Field,
    history: Vec<String>,
    hist_pos: usize,
}

impl Chat {
    pub fn new() -> Chat {
        Chat { open: false, field: Field::new("", 120), history: Vec::new(), hist_pos: 0 }
    }

    /// Tasten verarbeiten. Gibt den Text zurück, wenn abgeschickt.
    pub fn update(&mut self) -> Option<String> {
        if !self.open {
            let opener =
                [KeyCode::Enter, KeyCode::KpEnter, KeyCode::T, KeyCode::Slash].into_iter().find(|&k| is_key_pressed(k));
            if let Some(k) = opener {
                drain_chars();
                self.open = true;
                self.field.focus = true;
                self.field.text = if k == KeyCode::Slash { "/".into() } else { String::new() };
                self.hist_pos = self.history.len();
            } else {
                drain_chars();
            }
            return None;
        }
        self.field.typing(|_| true);
        if is_key_pressed(KeyCode::Up) && self.hist_pos > 0 {
            self.hist_pos -= 1;
            self.field.text = self.history[self.hist_pos].clone();
        }
        if is_key_pressed(KeyCode::Escape) {
            self.open = false;
            return None;
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
            self.open = false;
            let text = self.field.text.trim().to_string();
            self.field.text.clear();
            if !text.is_empty() {
                self.history.push(text.clone());
                return Some(text);
            }
        }
        None
    }

    /// Chat unten links: letzte Zeilen, verblassen nach 10 s (offen: alle sichtbar)
    pub fn draw(&self, g: &Gfx, net: &NetClient) {
        let now = net.clock();
        let lines: Vec<_> = net.chat.iter().rev().take(if self.open { 10 } else { 6 }).collect();
        let mut y = VIEW - 44.0;
        for l in lines {
            let age = (now - l.time) as f32;
            let alpha = if self.open { 1.0 } else { ((10.0 - age) / 1.5).clamp(0.0, 1.0) };
            if alpha <= 0.0 {
                y -= 20.0;
                continue;
            }
            let (name, c) = match l.from {
                Some(s) => (
                    format!("{}: ", net.roster.name(s as usize)),
                    mix(COLORS[(s as usize).min(7)], [255, 255, 255], 0.35),
                ),
                None => (String::new(), ACCENT_C),
            };
            let wn = if name.is_empty() { 0.0 } else { g.measure(&name, 15.0, false).width };
            let wt = g.measure(&l.text, 15.0, false).width;
            draw_rectangle(8.0, y - 10.0, wn + wt + 12.0, 20.0, Color::new(0.04, 0.045, 0.06, 0.62 * alpha));
            if !name.is_empty() {
                g.text(&name, 14.0, y, 15.0, cola(c, alpha), 0.0);
            }
            let tc = if l.from.is_none() { [255, 214, 120] } else { [232, 236, 244] };
            g.text(&l.text, 14.0 + wn, y, 15.0, cola(tc, alpha), 0.0);
            y -= 20.0;
        }
        if self.open {
            let r = Rect::new(8.0, VIEW - 30.0, 520.0, 26.0);
            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.03, 0.035, 0.05, 0.9));
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, col(ACCENT_C));
            let caret = if (get_time() * 2.0) as i64 % 2 == 0 { "|" } else { "" };
            g.text("Chat:", r.x + 8.0, r.y + 13.0, 15.0, col(DIM_C), 0.0);
            g.text(&format!("{}{caret}", self.field.text), r.x + 50.0, r.y + 13.0, 15.0, col(TEXT_C), 0.0);
        }
    }
}

/// Spielerliste (Tab gedrückt halten). Der Host kann hier kicken und bannen.
/// Gibt einen Befehl zurück ("/kick #3"), wenn geklickt wurde.
pub fn scoreboard(
    g: &mut Gfx,
    roster: &Roster,
    scores: &[u32; MAX_PLAYERS],
    me: usize,
    admin: bool,
    queued: &[bool; MAX_PLAYERS],
    m: Vec2,
) -> Option<String> {
    let slots: Vec<usize> = (0..MAX_PLAYERS).filter(|&s| !roster.names[s].is_empty()).collect();
    let h = 70.0 + slots.len() as f32 * 34.0;
    let r = Rect::new(VW / 2.0 - 300.0, 80.0, 600.0, h);
    draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.05, 0.055, 0.08, 0.94));
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, col([70, 78, 100]));
    g.fancy("SPIELER", r.x + r.w / 2.0, r.y + 22.0, 26.0, ACCENT_C, 2.0, true, 1.0, 1.0);
    g.text("Punkte", r.x + 330.0, r.y + 50.0, 13.0, col(DIM_C), 0.5);
    g.text("Ping", r.x + 400.0, r.y + 50.0, 13.0, col(DIM_C), 0.5);
    let mut cmd = None;
    for (k, &s) in slots.iter().enumerate() {
        let y = r.y + 72.0 + k as f32 * 34.0;
        if s == me {
            draw_rectangle(r.x + 6.0, y - 14.0, r.w - 12.0, 28.0, Color::new(1.0, 1.0, 1.0, 0.05));
        }
        draw_circle(r.x + 24.0, y, 7.0, col(COLORS[s]));
        g.text(&format!("#{}", s + 1), r.x + 38.0, y, 14.0, col(DIM_C), 0.0);
        let bot = roster.bots & (1 << s) != 0;
        let mut name = roster.names[s].clone();
        if roster.admin[s] {
            name.push_str("  (Host)");
        }
        if queued[s] {
            name.push_str("  – wartet");
        }
        g.text(&name, r.x + 70.0, y, 16.0, col(mix(COLORS[s], [255, 255, 255], 0.4)), 0.0);
        g.text(&scores[s].to_string(), r.x + 330.0, y, 16.0, col(TEXT_C), 0.5);
        if !bot {
            g.text(&format!("{} ms", roster.ping[s]), r.x + 400.0, y, 14.0, col(TEXT_C), 0.5);
        }
        if admin && s != me && !bot {
            if button(g, Rect::new(r.x + 452.0, y - 12.0, 64.0, 24.0), "Kick", m, [255, 170, 60], 15.0) {
                cmd = Some(format!("/kick #{}", s + 1));
            }
            if button(g, Rect::new(r.x + 524.0, y - 12.0, 64.0, 24.0), "Bann", m, [255, 70, 60], 15.0) {
                cmd = Some(format!("/ban #{}", s + 1));
            }
        }
    }
    let hint = if admin {
        "Chat-Befehle: /kick, /ban, /unban, /bans, /mute, /bots <n>, /help"
    } else {
        "Chat mit T oder Enter  ·  /help zeigt Befehle"
    };
    g.text(hint, r.x + r.w / 2.0, r.y + r.h + 14.0, 13.0, col(DIM_C), 0.5);
    cmd
}
