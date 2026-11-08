//! Headless authoritative room, shared by the standalone server and socket tests.
use crate::{
    party::*,
    party_net::{Connection, MAX_COMMAND},
    simulation::{Input, STEP},
};
use std::{
    io,
    net::{SocketAddr, TcpListener},
    time::{Duration, Instant},
};

struct Peer {
    id: u64,
    connection: Connection,
    joined: bool,
    closing: bool,
    input: Input,
    last_input: Instant,
    last_message: Instant,
    rate_since: Instant,
    messages: usize,
    blocked_since: Option<Instant>,
}

pub struct PartyServer {
    listener: TcpListener,
    pub room: Room,
    peers: Vec<Peer>,
    next_id: u64,
    tick: u64,
    previous: Instant,
    accumulator: f64,
}

impl PartyServer {
    pub fn bind(address: SocketAddr, room: Room) -> io::Result<Self> {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            room,
            peers: vec![],
            next_id: 1,
            tick: 0,
            previous: Instant::now(),
            accumulator: 0.0,
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    fn broadcast_lobby(&mut self) {
        for peer in &mut self.peers {
            if peer.joined
                && peer
                    .connection
                    .send(&ServerMessage::Lobby(self.room.lobby.clone()))
                    .is_err()
            {
                peer.closing = true;
            }
        }
    }

    fn message(&mut self, index: usize, message: ClientMessage, now: Instant) {
        let id = self.peers[index].id;
        let result = match message {
            ClientMessage::Join { protocol, name } if !self.peers[index].joined => {
                if protocol != PROTOCOL {
                    Err("Game/server versions do not match. Update both builds.".into())
                } else {
                    self.room.join(id, &name).and_then(|()| {
                        self.peers[index].joined = true;
                        self.peers[index]
                            .connection
                            .send(&ServerMessage::Welcome {
                                id,
                                protocol: PROTOCOL,
                            })
                            .map_err(|e| e.to_string())
                    })
                }
            }
            ClientMessage::Ping(stamp) if self.peers[index].joined => {
                if self.peers[index]
                    .connection
                    .send(&ServerMessage::Pong(stamp))
                    .is_err()
                {
                    self.peers[index].closing = true;
                }
                return;
            }
            ClientMessage::Input(input) if self.peers[index].joined => {
                match sanitize_input(input) {
                    Ok(mut input) => {
                        // Keep one-shot presses until a simulation tick consumes them.
                        input.dash |= self.peers[index].input.dash;
                        input.reload |= self.peers[index].input.reload;
                        input.weapon = input.weapon.or(self.peers[index].input.weapon);
                        self.peers[index].input = input;
                        self.peers[index].last_input = now;
                        return;
                    }
                    Err(e) => Err(e),
                }
            }
            command if self.peers[index].joined => {
                let was_running = self.room.game.is_some();
                let result = self.room.command(id, &command);
                if result.is_ok() && !was_running && self.room.game.is_some() {
                    self.tick = 0;
                    self.accumulator = 0.0;
                    let started = ServerMessage::Started {
                        map: self.room.map_text(),
                        name: self.room.level.name.clone(),
                        endless: self.room.lobby.endless,
                        members: self.room.lobby.members.clone(),
                        difficulty: self.room.lobby.difficulty,
                    };
                    for peer in &mut self.peers {
                        peer.input = Input::default();
                        if peer.joined && peer.connection.send(&started).is_err() {
                            peer.closing = true;
                        }
                    }
                }
                result
            }
            _ => Err("Join the room first".into()),
        };
        match result {
            Ok(()) => self.broadcast_lobby(),
            Err(error) => {
                let peer = &mut self.peers[index];
                if peer.connection.send(&ServerMessage::Error(error)).is_err() || !peer.joined {
                    peer.closing = true;
                }
            }
        }
    }

    /// One bounded nonblocking service pass; call frequently from the process loop.
    pub fn poll(&mut self) -> io::Result<()> {
        let now = Instant::now();
        for _ in 0..16 {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    if self.peers.len() >= 16 {
                        continue;
                    }
                    let connection = Connection::new(stream, MAX_COMMAND)?;
                    self.peers.push(Peer {
                        id: self.next_id,
                        connection,
                        joined: false,
                        closing: false,
                        input: Input::default(),
                        last_input: now,
                        last_message: now,
                        rate_since: now,
                        messages: 0,
                        blocked_since: None,
                    });
                    self.next_id += 1;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
        for index in 0..self.peers.len() {
            if self.peers[index].closing {
                continue;
            }
            match self.peers[index].connection.receive::<ClientMessage>() {
                Ok(messages) => {
                    let peer = &mut self.peers[index];
                    if now.duration_since(peer.rate_since) >= Duration::from_secs(1) {
                        peer.messages = 0;
                        peer.rate_since = now;
                    }
                    peer.messages += messages.len();
                    if peer.messages > 256 {
                        peer.closing = true;
                        continue;
                    }
                    if !messages.is_empty() {
                        peer.last_message = now;
                    }
                    for message in messages {
                        self.message(index, message, now);
                        if self.peers[index].closing {
                            break;
                        }
                    }
                }
                Err(_) => self.peers[index].closing = true,
            }
        }
        self.accumulator += now.duration_since(self.previous).as_secs_f64().min(0.2);
        self.previous = now;
        while self.accumulator >= STEP as f64 {
            self.accumulator -= STEP as f64;
            self.tick += 1;
            if let Some(game) = &mut self.room.game {
                let inputs: Vec<_> = game
                    .players()
                    .map(|player| {
                        let member = self
                            .room
                            .lobby
                            .members
                            .iter()
                            .find(|m| m.slot == player.slot);
                        member
                            .and_then(|m| self.peers.iter().find(|p| p.id == m.id && !p.closing))
                            .filter(|p| {
                                now.duration_since(p.last_input) < Duration::from_millis(250)
                            })
                            .map_or_else(Input::default, |p| p.input)
                    })
                    .collect();
                game.update_party(&inputs, STEP);
                for peer in &mut self.peers {
                    peer.input.dash = false;
                    peer.input.reload = false;
                    peer.input.weapon = None;
                }
                if self.tick.is_multiple_of(6) {
                    let snapshot =
                        ServerMessage::Snapshot(Box::new(Snapshot::capture(game, self.tick)));
                    for peer in &mut self.peers {
                        // Skip obsolete snapshots behind a slow socket instead of building an unbounded queue.
                        if peer.joined
                            && !peer.closing
                            && !peer.connection.pending()
                            && peer.connection.send(&snapshot).is_err()
                        {
                            peer.closing = true;
                        }
                    }
                    game.sounds.clear();
                }
            }
        }
        let mut departed = vec![];
        self.peers.retain_mut(|peer| {
            let timeout = if peer.joined { 15 } else { 5 };
            let flushed = peer.connection.flush().is_ok();
            if peer.connection.pending() {
                peer.blocked_since.get_or_insert(now);
            } else {
                peer.blocked_since = None;
            }
            let stalled = peer
                .blocked_since
                .is_some_and(|t| now.duration_since(t) > Duration::from_secs(3));
            let keep = flushed
                && !stalled
                && !(peer.closing && !peer.connection.pending())
                && now.duration_since(peer.last_message) < Duration::from_secs(timeout);
            if !keep && peer.joined {
                departed.push(peer.id);
            }
            keep
        });
        if !departed.is_empty() {
            for id in departed {
                self.room.leave(id);
            }
            self.broadcast_lobby();
        }
        Ok(())
    }
}
