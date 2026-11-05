//! Bannliste: nach IP und nach Installations-Token, gespeichert als Textdatei.

use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Ban {
    pub ip: IpAddr,
    pub token: u64,
    pub name: String,
    pub reason: String,
}

#[derive(Default)]
pub struct Bans {
    pub list: Vec<Ban>,
    file: Option<PathBuf>,
}

impl Bans {
    /// Lädt die Liste (fehlende Datei = leer). Format pro Zeile: ip TAB token TAB name TAB grund
    pub fn load(file: Option<PathBuf>) -> Bans {
        let mut list = Vec::new();
        if let Some(path) = &file {
            if let Ok(text) = fs::read_to_string(path) {
                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    let parts: Vec<&str> = line.split('\t').collect();
                    let Some(ip) = parts.first().and_then(|p| p.parse().ok()) else { continue };
                    let token = parts.get(1).and_then(|t| u64::from_str_radix(t, 16).ok()).unwrap_or(0);
                    list.push(Ban {
                        ip,
                        token,
                        name: parts.get(2).unwrap_or(&"").to_string(),
                        reason: parts.get(3).unwrap_or(&"").to_string(),
                    });
                }
            }
        }
        Bans { list, file }
    }

    fn save(&self) {
        let Some(path) = &self.file else { return };
        let mut out = String::from("# Panzer-Bannliste: ip<TAB>token<TAB>name<TAB>grund\n");
        for b in &self.list {
            out.push_str(&format!(
                "{}\t{:016x}\t{}\t{}\n",
                b.ip,
                b.token,
                b.name.replace('\t', " "),
                b.reason.replace('\t', " ")
            ));
        }
        if let Err(e) = fs::write(path, out) {
            eprintln!("Bannliste konnte nicht gespeichert werden: {e}");
        }
    }

    pub fn find(&self, ip: IpAddr, token: u64) -> Option<&Ban> {
        // Loopback nie per IP bannen (sonst sperrt sich der Host beim Testen selbst aus)
        self.list.iter().find(|b| (b.ip == ip && !ip.is_loopback()) || (token != 0 && b.token == token))
    }

    pub fn add(&mut self, ban: Ban) {
        self.list.retain(|b| !(b.ip == ban.ip && b.token == ban.token));
        self.list.push(ban);
        self.save();
    }

    /// Entbannen per Name, IP oder Nummer aus /bans. Gibt die entfernten zurück.
    pub fn remove(&mut self, what: &str) -> Vec<Ban> {
        let what = what.trim();
        let mut gone = Vec::new();
        if let Ok(n) = what.parse::<usize>() {
            if n >= 1 && n <= self.list.len() {
                gone.push(self.list.remove(n - 1));
            }
        } else {
            let lower = what.to_lowercase();
            let (out, keep): (Vec<Ban>, Vec<Ban>) =
                self.list.drain(..).partition(|b| b.ip.to_string() == what || b.name.to_lowercase() == lower);
            self.list = keep;
            gone = out;
        }
        if !gone.is_empty() {
            self.save();
        }
        gone
    }
}
