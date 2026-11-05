//! Zuverlässige, geordnete Nachrichten über UDP (Maze, Chat, Spielerliste) und das
//! Zerlegen großer Snapshots in mehrere Pakete.

use std::collections::{BTreeMap, VecDeque};
use std::time::{Duration, Instant};

/// so lange wird auf ein Ack gewartet, bevor eine Nachricht nochmal mitgeht
const RESEND: Duration = Duration::from_millis(120);
/// höchstens so viele ungelesene Nachrichten puffern (danach Verbindung kaputt)
pub const MAX_PENDING: usize = 512;
/// Nutzlast pro UDP-Paket (unter der typischen MTU, auch über VPNs wie Tailscale)
pub const MAX_PAYLOAD: usize = 1100;

struct OutMsg {
    id: u32,
    data: Vec<u8>,
    sent: Option<Instant>,
}

/// Ausgehende zuverlässige Nachrichten: gehen in jedem Paket mit, bis sie bestätigt sind.
#[derive(Default)]
pub struct RelOut {
    next: u32,
    queue: VecDeque<OutMsg>,
}

impl RelOut {
    pub fn new() -> Self {
        RelOut { next: 1, queue: VecDeque::new() }
    }

    pub fn push(&mut self, data: Vec<u8>) {
        if self.next == 0 {
            self.next = 1;
        }
        self.queue.push_back(OutMsg { id: self.next, data, sent: None });
        self.next = self.next.wrapping_add(1);
    }

    /// Gegenstelle hat alles bis einschließlich upto
    pub fn ack(&mut self, upto: u32) {
        while self.queue.front().is_some_and(|m| m.id <= upto) {
            self.queue.pop_front();
        }
    }

    pub fn backlog(&self) -> usize {
        self.queue.len()
    }

    /// Nachrichten, die jetzt (wieder) mitgehen sollen, höchstens budget Bytes
    pub fn collect(&mut self, now: Instant, budget: usize) -> Vec<(u32, Vec<u8>)> {
        let mut out = Vec::new();
        let mut used = 0;
        for m in self.queue.iter_mut() {
            if m.sent.is_some_and(|t| now.duration_since(t) < RESEND) {
                continue;
            }
            if used + m.data.len() + 8 > budget && !out.is_empty() {
                break;
            }
            used += m.data.len() + 8;
            m.sent = Some(now);
            out.push((m.id, m.data.clone()));
        }
        out
    }
}

/// Eingehende zuverlässige Nachrichten: in Reihenfolge ausliefern, Doppelte verwerfen.
pub struct RelIn {
    next: u32,
    pending: BTreeMap<u32, Vec<u8>>,
}

impl Default for RelIn {
    fn default() -> Self {
        RelIn { next: 1, pending: BTreeMap::new() }
    }
}

impl RelIn {
    /// Ok(fertige Nachrichten) oder Err, wenn die Gegenstelle uns zuschüttet
    pub fn receive(&mut self, id: u32, data: Vec<u8>) -> Result<Vec<Vec<u8>>, ()> {
        if id < self.next {
            return Ok(Vec::new()); // schon gehabt
        }
        if id - self.next > MAX_PENDING as u32 || self.pending.len() >= MAX_PENDING {
            return Err(());
        }
        self.pending.insert(id, data);
        let mut out = Vec::new();
        while let Some(d) = self.pending.remove(&self.next) {
            out.push(d);
            self.next += 1;
        }
        Ok(out)
    }

    /// höchste lückenlos empfangene id
    pub fn ack(&self) -> u32 {
        self.next - 1
    }
}

/// Großes Paket in Stücke zerlegen
pub fn split(body: &[u8]) -> Vec<&[u8]> {
    body.chunks(MAX_PAYLOAD).collect()
}

/// Setzt zerlegte Snapshots wieder zusammen (nur der neueste zählt)
#[derive(Default)]
pub struct Reassembly {
    seq: u32,
    parts: Vec<Option<Vec<u8>>>,
}

impl Reassembly {
    pub fn add(&mut self, seq: u32, idx: u8, count: u8, data: &[u8]) -> Option<Vec<u8>> {
        if count == 0 || idx >= count || count > 64 {
            return None;
        }
        if seq != self.seq || self.parts.len() != count as usize {
            if seq < self.seq && !self.parts.is_empty() {
                return None; // älter als das, was wir gerade bauen
            }
            self.seq = seq;
            self.parts = vec![None; count as usize];
        }
        self.parts[idx as usize] = Some(data.to_vec());
        if self.parts.iter().all(|p| p.is_some()) {
            let mut out = Vec::new();
            for p in self.parts.drain(..) {
                out.extend_from_slice(&p.unwrap_or_default());
            }
            return Some(out);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reliable_in_order() {
        let mut out = RelOut::new();
        let mut inn = RelIn::default();
        for i in 0..5u8 {
            out.push(vec![i]);
        }
        let now = Instant::now();
        let mut msgs = out.collect(now, 10_000);
        msgs.reverse(); // falsche Reihenfolge
        let mut got = Vec::new();
        for (id, d) in msgs {
            got.extend(inn.receive(id, d).unwrap());
        }
        assert_eq!(got, (0..5u8).map(|i| vec![i]).collect::<Vec<_>>());
        out.ack(inn.ack());
        assert_eq!(out.backlog(), 0);
    }
}
