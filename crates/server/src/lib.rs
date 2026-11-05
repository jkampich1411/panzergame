//! Panzer-Server als Bibliothek: wird vom dedizierten Server (main.rs) und vom Spiel
//! selbst (Menü "Spiel hosten") benutzt.

pub mod bans;
pub mod guard;
mod server;

pub use server::{Config, Server};
