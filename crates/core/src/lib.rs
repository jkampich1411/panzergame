//! Panzer – Labyrinth-Panzerschlacht. Dieses Crate enthält alles ohne Grafik:
//! Labyrinth, Physik, die autoritative Spiellogik, Bots und das Netzwerkprotokoll.
//! Server und Client benutzen denselben Code, deshalb stimmt die Prediction exakt.

pub mod arena;
pub mod bot;
pub mod codec;
pub mod consts;
pub mod game;
pub mod geom;
pub mod items;
pub mod maze;
pub mod net;
pub mod physics;
pub mod proto;
pub mod rng;
pub mod snap;
pub mod text;

pub use arena::Arena;
pub use game::Game;
pub use items::{Eff, Item, Kind};
pub use maze::Maze;
