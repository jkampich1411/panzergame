//! UDP-Pakete zwischen Client und Server.
//!
//! Client -> Server: Hello (Name, Version), Input (mehrere Eingaben redundant, Acks,
//! zuverlässige Nachrichten wie Chat), Bye.
//! Server -> Client: Welcome/Reject, Snap (Delta-Snapshot + zuverlässige Nachrichten),
//! Frag (Stück eines großen Snap), Kick.

use crate::codec::{read_maze, write_maze, DecodeError, Reader, Res, Writer};
use crate::consts::{MAX_PLAYERS, PROTO};
use crate::maze::Maze;

pub const MAGIC: u16 = 0x505A; // "PZ"

pub const C_HELLO: u8 = 1;
pub const C_INPUT: u8 = 2;
pub const C_BYE: u8 = 3;
pub const S_WELCOME: u8 = 10;
pub const S_REJECT: u8 = 11;
pub const S_SNAP: u8 = 12;
pub const S_FRAG: u8 = 13;
pub const S_KICK: u8 = 14;

/// höchstens so viele Eingaben in einem Input-Paket
pub const MAX_CMDS: usize = 24;

/// Eine Eingabe des Spielers für einen Frame
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cmd {
    pub keys: u8,
    pub fires: u16,
    /// Framedauer in ms (Server begrenzt auf 50)
    pub dt_ms: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hello {
    pub proto: u16,
    pub name: String,
    /// zufällige Kennung pro Installation (für Bans und Wiederverbinden)
    pub token: u64,
    /// Host-Schlüssel: wer ihn kennt, ist Admin (der Host selbst)
    pub host_key: u64,
}

impl Hello {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(C_HELLO);
        w.u16(MAGIC);
        w.u16(self.proto);
        w.str(&self.name);
        w.u64(self.token);
        w.u64(self.host_key);
        w.buf
    }

    pub fn decode(r: &mut Reader) -> Res<Hello> {
        if r.u16()? != MAGIC {
            return Err(DecodeError("kein Panzer-Paket"));
        }
        Ok(Hello { proto: r.u16()?, name: r.str(64)?, token: r.u64()?, host_key: r.u64()? })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    pub session: u32,
    /// letzter vollständig empfangener Snapshot (Basis für Deltas)
    pub snap_ack: u32,
    /// höchste lückenlos empfangene zuverlässige Nachricht
    pub rel_ack: u32,
    /// Sequenznummer der ersten Eingabe in cmds
    pub first_seq: u32,
    pub cmds: Vec<Cmd>,
    pub rel: Vec<(u32, Vec<u8>)>,
}

impl Input {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(C_INPUT);
        w.u32(self.session);
        w.var(self.snap_ack as u64);
        w.var(self.rel_ack as u64);
        w.var(self.first_seq as u64);
        w.u8(self.cmds.len() as u8);
        for c in &self.cmds {
            w.u8(c.keys);
            w.u16(c.fires);
            w.u8(c.dt_ms);
        }
        w.u8(self.rel.len() as u8);
        for (id, d) in &self.rel {
            w.var(*id as u64);
            w.bytes(d);
        }
        w.buf
    }

    pub fn decode(r: &mut Reader) -> Res<Input> {
        let session = r.u32()?;
        let snap_ack = r.var32()?;
        let rel_ack = r.var32()?;
        let first_seq = r.var32()?;
        let n = r.u8()? as usize;
        if n > MAX_CMDS {
            return Err(DecodeError("zu viele Eingaben"));
        }
        let mut cmds = Vec::with_capacity(n);
        for _ in 0..n {
            cmds.push(Cmd { keys: r.u8()?, fires: r.u16()?, dt_ms: r.u8()? });
        }
        let n = r.u8()? as usize;
        if n > 16 {
            return Err(DecodeError("zu viele Nachrichten"));
        }
        let mut rel = Vec::with_capacity(n);
        for _ in 0..n {
            rel.push((r.var32()?, r.bytes(512)?.to_vec()));
        }
        Ok(Input { session, snap_ack, rel_ack, first_seq, cmds, rel })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Welcome {
    pub session: u32,
    pub slot: u8,
    pub admin: bool,
    pub name: String,
}

impl Welcome {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(S_WELCOME);
        w.u16(MAGIC);
        w.u16(PROTO);
        w.u32(self.session);
        w.u8(self.slot);
        w.u8(self.admin as u8);
        w.str(&self.name);
        w.buf
    }

    /// Err(Text) bei falscher Version
    pub fn decode(r: &mut Reader) -> Res<Result<Welcome, String>> {
        if r.u16()? != MAGIC {
            return Err(DecodeError("kein Panzer-Paket"));
        }
        let proto = r.u16()?;
        if proto != PROTO {
            return Ok(Err(format!("Server hat eine andere Spielversion ({proto}, du {PROTO}).")));
        }
        let session = r.u32()?;
        let slot = r.u8()?;
        if slot as usize >= MAX_PLAYERS {
            return Err(DecodeError("Slot"));
        }
        Ok(Ok(Welcome { session, slot, admin: r.u8()? != 0, name: r.str(64)? }))
    }
}

pub fn encode_reason(tag: u8, reason: &str) -> Vec<u8> {
    let mut w = Writer::new();
    w.u8(tag);
    w.str(reason);
    w.buf
}

/// Kopf eines Snap-Pakets (danach folgt der Snapshot)
#[derive(Clone, Debug, PartialEq)]
pub struct SnapHead {
    pub seq: u32,
    /// Basis-Snapshot für das Delta (0 = voll)
    pub base: u32,
    /// Server hat die zuverlässigen Nachrichten des Clients bis hier
    pub rel_ack: u32,
    pub rel: Vec<(u32, Vec<u8>)>,
}

impl SnapHead {
    pub fn write(&self, w: &mut Writer) {
        w.var(self.seq as u64);
        w.var(self.base as u64);
        w.var(self.rel_ack as u64);
        w.u8(self.rel.len() as u8);
        for (id, d) in &self.rel {
            w.var(*id as u64);
            w.bytes(d);
        }
    }

    pub fn read(r: &mut Reader) -> Res<SnapHead> {
        let seq = r.var32()?;
        let base = r.var32()?;
        let rel_ack = r.var32()?;
        let n = r.u8()? as usize;
        let mut rel = Vec::with_capacity(n);
        for _ in 0..n {
            rel.push((r.var32()?, r.bytes(16384)?.to_vec()));
        }
        Ok(SnapHead { seq, base, rel_ack, rel })
    }
}

// ---------------------------------------------------------------------------
// Zuverlässige Nachrichten

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerInfo {
    pub slot: u8,
    pub name: String,
    pub bot: bool,
    pub admin: bool,
    /// wartet auf die nächste Runde
    pub queued: bool,
    pub ping: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Msg {
    /// neues Labyrinth bzw. neue Wände
    Maze(Maze),
    Players(Vec<PlayerInfo>),
    /// from = Slot oder None (System)
    Chat {
        from: Option<u8>,
        text: String,
    },
    /// Client -> Server: Chatzeile oder /Befehl
    Say(String),
}

impl Msg {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        match self {
            Msg::Maze(m) => {
                w.u8(1);
                write_maze(&mut w, m);
            }
            Msg::Players(list) => {
                w.u8(2);
                w.u8(list.len() as u8);
                for p in list {
                    w.u8(p.slot);
                    w.str(&p.name);
                    w.u8(p.bot as u8 | (p.admin as u8) << 1 | (p.queued as u8) << 2);
                    w.u16(p.ping);
                }
            }
            Msg::Chat { from, text } => {
                w.u8(3);
                w.u8(from.unwrap_or(255));
                w.str(text);
            }
            Msg::Say(text) => {
                w.u8(10);
                w.str(text);
            }
        }
        w.buf
    }

    pub fn decode(data: &[u8]) -> Res<Msg> {
        let mut r = Reader::new(data);
        Ok(match r.u8()? {
            1 => Msg::Maze(read_maze(&mut r)?),
            2 => {
                let n = r.u8()? as usize;
                if n > MAX_PLAYERS {
                    return Err(DecodeError("Spielerliste"));
                }
                let mut list = Vec::with_capacity(n);
                for _ in 0..n {
                    let slot = r.u8()?;
                    let name = r.str(64)?;
                    let f = r.u8()?;
                    list.push(PlayerInfo {
                        slot,
                        name,
                        bot: f & 1 != 0,
                        admin: f & 2 != 0,
                        queued: f & 4 != 0,
                        ping: r.u16()?,
                    });
                }
                Msg::Players(list)
            }
            3 => {
                let from = r.u8()?;
                Msg::Chat { from: if from == 255 { None } else { Some(from) }, text: r.str(400)? }
            }
            10 => Msg::Say(r.str(400)?),
            _ => return Err(DecodeError("Nachricht")),
        })
    }
}
