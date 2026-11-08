//! Nonblocking desktop/test client. The server owns simulation and hit detection.
use crate::{
    level::Level,
    party::*,
    party_net::{Connection, MAX_FRAME},
    simulation::Game,
};
use std::{
    io,
    net::TcpStream,
    time::{Duration, Instant},
};

pub struct PartyClient {
    connection: Connection,
    pub id: u64,
    pub lobby: Option<Lobby>,
    pub members: Vec<Member>,
    pub game: Option<Game>,
    pub notice: String,
    pub slot: u8,
    pub ping_ms: u128,
    pub snapshot_at: Instant,
    pub tick: u64,
    last_received: Instant,
    last_ping: Instant,
    epoch: Instant,
}

impl PartyClient {
    pub fn new(stream: TcpStream, name: String) -> io::Result<Self> {
        let mut connection = Connection::new(stream, MAX_FRAME)?;
        connection.send(&ClientMessage::Join {
            protocol: PROTOCOL,
            name,
        })?;
        let now = Instant::now();
        Ok(Self {
            connection,
            id: 0,
            lobby: None,
            members: vec![],
            game: None,
            notice: String::new(),
            slot: 0,
            ping_ms: 0,
            snapshot_at: now,
            tick: 0,
            last_received: now,
            last_ping: now,
            epoch: now,
        })
    }

    pub fn send(&mut self, message: ClientMessage) -> Result<(), String> {
        self.connection.send(&message).map_err(|e| e.to_string())
    }

    pub fn poll(&mut self) -> Result<(), String> {
        let messages = self
            .connection
            .receive::<ServerMessage>()
            .map_err(|e| e.to_string())?;
        if !messages.is_empty() {
            self.last_received = Instant::now();
        }
        for message in messages {
            match message {
                ServerMessage::Welcome { id, protocol } => {
                    if protocol != PROTOCOL {
                        return Err("Game/server versions do not match".into());
                    }
                    self.id = id;
                }
                ServerMessage::Lobby(lobby) => {
                    if !lobby.in_game {
                        self.game = None;
                        self.members = lobby.members.clone();
                    }
                    self.lobby = Some(lobby);
                }
                ServerMessage::Started {
                    map,
                    name,
                    endless,
                    members,
                    difficulty,
                } => {
                    let level = Level::parse(&name, &map)?;
                    if level.width > 128 || level.height > 128 {
                        return Err("Server arena is too large".into());
                    }
                    if !difficulty.valid() {
                        return Err("Invalid server difficulty".into());
                    }
                    let mut game = Game::new_party(level, endless, members.len())?;
                    game.difficulty = difficulty;
                    self.slot = members
                        .iter()
                        .find(|m| m.id == self.id)
                        .ok_or("Missing player in server roster")?
                        .slot;
                    game.player.slot = members[0].slot;
                    game.player.name = members[0].name.clone();
                    for (player, member) in game.allies.iter_mut().zip(members.iter().skip(1)) {
                        player.slot = member.slot;
                        player.name = member.name.clone();
                    }
                    if let Some(index) = game.allies.iter().position(|p| p.slot == self.slot) {
                        std::mem::swap(&mut game.player, &mut game.allies[index]);
                    }
                    self.game = Some(game);
                    self.members = members;
                    self.tick = 0;
                    self.notice.clear();
                    self.snapshot_at = Instant::now();
                }
                ServerMessage::Snapshot(snapshot) => {
                    if snapshot.tick > self.tick {
                        self.tick = snapshot.tick;
                        if let Some(game) = &mut self.game {
                            snapshot.apply(game, self.slot)?;
                        }
                        self.snapshot_at = Instant::now();
                    }
                }
                ServerMessage::Error(error) => self.notice = error,
                ServerMessage::Pong(stamp) => {
                    self.ping_ms = self
                        .epoch
                        .elapsed()
                        .as_millis()
                        .saturating_sub(stamp as u128)
                }
            }
        }
        if self.last_received.elapsed() > Duration::from_secs(10) {
            return Err(
                "Connection timed out. Rejoin the waiting room when the server is available."
                    .into(),
            );
        }
        if self.last_ping.elapsed() >= Duration::from_secs(1) {
            self.send(ClientMessage::Ping(self.epoch.elapsed().as_millis() as u64))?;
            self.last_ping = Instant::now();
        }
        self.connection.flush().map_err(|e| e.to_string())
    }
}
