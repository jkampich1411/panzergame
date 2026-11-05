//! Dedizierter Panzer-Server.
//!
//!     panzer-server [--port 5555] [--bots 0] [--admin-pass PW] [--bans bans.txt]
//!
//! Befehle auf der Konsole (ohne /): players, kick, ban, unban, bans, mute, unmute, bots, say

use std::io::BufRead;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc};

use panzer_server::{Config, Server};

fn usage() -> ! {
    eprintln!(
        "panzer-server – dedizierter Server für Panzer\n\n\
         Optionen:\n  \
           --port N          UDP-Port (Standard 5555)\n  \
           --bots N          freie Plätze mit N Bots füllen (0-7)\n  \
           --admin-pass PW   Passwort für /login im Spiel (Admin: kick, ban, bots …)\n  \
           --bans DATEI      Bannliste (Standard bans.txt)\n  \
           --quiet           keine Logausgabe\n\n\
         Konsolenbefehle: players, kick <spieler> [grund], ban <spieler> [grund],\n\
         unban <name|ip|nr>, bans, mute/unmute <spieler>, bots <n>, say <text>, help"
    );
    std::process::exit(2)
}

fn main() {
    let mut cfg = Config::default();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut val = || args.next().unwrap_or_else(|| usage());
        match a.as_str() {
            "--port" | "-p" => cfg.port = val().parse().unwrap_or_else(|_| usage()),
            "--bots" | "-b" => cfg.bots = val().parse::<usize>().unwrap_or_else(|_| usage()).min(7),
            "--admin-pass" => cfg.admin_pass = Some(val()),
            "--bans" => cfg.bans_file = Some(PathBuf::from(val())),
            "--quiet" | "-q" => cfg.verbose = false,
            _ => usage(),
        }
    }
    if cfg.admin_pass.is_none() {
        println!("Hinweis: ohne --admin-pass kann nur die Konsole kicken/bannen.");
    }
    let mut server = match Server::bind(cfg) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Port kann nicht geöffnet werden: {e}");
            std::process::exit(1);
        }
    };
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    server.set_console(rx);
    let stop = Arc::new(AtomicBool::new(false));
    server.run(&stop);
}
