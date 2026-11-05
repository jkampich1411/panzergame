//! Gespeicherte Einstellungen: Name, letzte IP, Bots, Ton. Dazu eine zufällige Kennung
//! pro Installation (der Server bannt damit auch über IP-Wechsel hinweg).

use std::fs;
use std::path::PathBuf;

use panzer_core::consts::RoundRule;

pub struct Config {
    pub name: String,
    pub last_ip: String,
    pub bots: usize,
    /// Sonderregeln beim Hosten, solo und lokal
    pub rule: RoundRule,
    pub sfx: bool,
    pub music: bool,
    pub token: u64,
}

fn path() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    }?;
    Some(base.join("panzer").join("config.txt"))
}

impl Config {
    pub fn load() -> Config {
        let mut c = Config {
            name: String::new(),
            last_ip: String::new(),
            bots: 3,
            rule: RoundRule::default(),
            sfx: true,
            music: true,
            token: 0,
        };
        if let Some(text) = path().and_then(|p| fs::read_to_string(p).ok()) {
            for line in text.lines() {
                let Some((k, v)) = line.split_once('=') else { continue };
                let v = v.trim();
                match k.trim() {
                    "name" => c.name = v.chars().take(16).collect(),
                    "ip" => c.last_ip = v.chars().take(60).collect(),
                    "bots" => c.bots = v.parse().unwrap_or(3).min(7),
                    "rule" => c.rule = RoundRule::from_key(v),
                    "sfx" => c.sfx = v != "0",
                    "music" => c.music = v != "0",
                    "token" => c.token = u64::from_str_radix(v, 16).unwrap_or(0),
                    _ => {}
                }
            }
        }
        if c.token == 0 {
            c.token = panzer_core::rng::Rng::from_entropy().next_u64() | 1;
            c.save();
        }
        c
    }

    pub fn save(&self) {
        let Some(p) = path() else { return };
        if let Some(dir) = p.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let text = format!(
            "name={}\nip={}\nbots={}\nrule={}\nsfx={}\nmusic={}\ntoken={:016x}\n",
            self.name,
            self.last_ip,
            self.bots,
            self.rule.key(),
            self.sfx as u8,
            self.music as u8,
            self.token
        );
        let _ = fs::write(p, text);
    }
}
