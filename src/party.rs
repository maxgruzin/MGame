//! Window-independent lobby rules and authoritative state sent to party clients.
use crate::{level::Level, simulation::*};
use macroquad::prelude::Vec2;
use serde::{Deserialize, Serialize};

pub const PROTOCOL: u32 = 2;
pub const DEFAULT_PORT: u16 = 7777;
pub const MAX_PLAYERS: usize = 8;
pub const COLORS: [u32; MAX_PLAYERS] = [
    0x61e8cb, 0xffac67, 0x8ebaff, 0xe99aff, 0xffe477, 0xff778e, 0xb7ec78, 0xe1e8ee,
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Member {
    pub id: u64,
    pub name: String,
    pub ready: bool,
    pub slot: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Lobby {
    pub members: Vec<Member>,
    pub host: u64,
    pub arena: String,
    pub endless: bool,
    pub in_game: bool,
    pub difficulty: Difficulty,
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    Join { protocol: u32, name: String },
    Difficulty(Difficulty),
    Ready(bool),
    Start,
    ReturnToLobby,
    Input(Input),
    Ping(u64),
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    Welcome {
        id: u64,
        protocol: u32,
    },
    Lobby(Lobby),
    Started {
        map: String,
        name: String,
        endless: bool,
        members: Vec<Member>,
        difficulty: Difficulty,
    },
    Snapshot(Box<Snapshot>),
    Error(String),
    Pong(u64),
}

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub tick: u64,
    pub players: Vec<Player>,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub grenades: Vec<Grenade>,
    pub blasts: Vec<Blast>,
    pub pickups: Vec<Pickup>,
    pub spawns: Vec<Spawn>,
    pub beacons: Vec<ReviveBeacon>,
    pub crates: Vec<Vec2>,
    pub wave: u32,
    pub remaining: u32,
    pub intermission: f32,
    pub wave_announcement: f32,
    pub score: u32,
    pub kills: u32,
    pub combo: u32,
    pub time: f32,
    pub victory: bool,
    pub sounds: Vec<SoundEvent>,
}

impl Snapshot {
    pub fn capture(game: &Game, tick: u64) -> Self {
        Self {
            tick,
            players: game.players().cloned().collect(),
            enemies: game.enemies.clone(),
            bullets: game.bullets.clone(),
            grenades: game.grenades.clone(),
            blasts: game.blasts.clone(),
            pickups: game.pickups.clone(),
            spawns: game.spawns.clone(),
            beacons: game.beacons.clone(),
            crates: game.level.crates.clone(),
            wave: game.wave,
            remaining: game.remaining,
            intermission: game.intermission,
            wave_announcement: game.wave_announcement,
            score: game.score,
            kills: game.kills,
            combo: game.combo,
            time: game.time,
            victory: game.victory,
            sounds: game.sounds.clone(),
        }
    }

    /// Render authoritative state with the local player in the existing HUD slot.
    pub fn apply(mut self, game: &mut Game, local_slot: u8) -> Result<(), String> {
        let index = self
            .players
            .iter()
            .position(|p| p.slot == local_slot)
            .ok_or("Server snapshot is missing this player")?;
        if self.players.len() > MAX_PLAYERS
            || self.players.iter().any(|p| p.weapon >= WEAPONS.len())
        {
            return Err("Invalid server player state".into());
        }
        for p in &mut self.players {
            if let Some(old) = game.players().find(|old| old.slot == p.slot) {
                p.previous = old.pos;
            }
        }
        game.player = self.players.remove(index);
        game.allies = self.players;
        for (index, enemy) in self.enemies.iter_mut().enumerate() {
            if let Some(old) = game
                .enemies
                .get(index)
                .filter(|e| e.kind == enemy.kind && e.pos.distance(enemy.pos) < 64.0)
            {
                enemy.previous = old.pos;
            }
        }
        game.enemies = self.enemies;
        game.bullets = self.bullets;
        game.grenades = self.grenades;
        game.blasts = self.blasts;
        game.pickups = self.pickups;
        game.spawns = self.spawns;
        game.beacons = self.beacons;
        game.level.crates = self.crates;
        game.wave = self.wave;
        game.remaining = self.remaining;
        game.intermission = self.intermission;
        game.wave_announcement = self.wave_announcement;
        game.score = self.score;
        game.kills = self.kills;
        game.combo = self.combo;
        game.time = self.time;
        game.victory = self.victory;
        game.sounds.extend(self.sounds);
        Ok(())
    }
}

pub struct Room {
    pub lobby: Lobby,
    pub game: Option<Game>,
    pub level: Level,
}

impl Room {
    pub fn new(level: Level, endless: bool) -> Result<Self, String> {
        // Bound per-player navigation work and network map size on a shared server.
        if level.width > 128 || level.height > 128 {
            return Err("Online arenas are limited to 128 x 128 tiles".into());
        }
        Ok(Self {
            lobby: Lobby {
                members: vec![],
                host: 0,
                arena: level.name.clone(),
                endless,
                in_game: false,
                difficulty: Difficulty::default(),
            },
            game: None,
            level,
        })
    }

    pub fn join(&mut self, id: u64, name: &str) -> Result<(), String> {
        if self.game.is_some() {
            return Err(
                "A match is already running. Join after the team returns to the lobby.".into(),
            );
        }
        if self.lobby.members.len() >= MAX_PLAYERS {
            return Err("The waiting room is full (8 players).".into());
        }
        if self.lobby.members.iter().any(|m| m.id == id) {
            return Err("Already joined".into());
        }
        let name: String = name
            .trim()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '-'))
            .take(20)
            .collect();
        if name.is_empty() {
            return Err("Enter a player name using letters or numbers.".into());
        }
        let slot = (0..MAX_PLAYERS as u8)
            .find(|s| self.lobby.members.iter().all(|m| m.slot != *s))
            .unwrap();
        self.lobby.members.push(Member {
            id,
            name,
            ready: false,
            slot,
        });
        if self.lobby.members.len() == 1 {
            self.lobby.host = id;
        }
        Ok(())
    }

    pub fn leave(&mut self, id: u64) {
        if let Some(member) = self.lobby.members.iter().find(|m| m.id == id) {
            if let Some(game) = &mut self.game {
                game.disconnect_player(member.slot);
            }
        }
        self.lobby.members.retain(|m| m.id != id);
        self.lobby.host = self.lobby.members.first().map_or(0, |m| m.id);
        if self.lobby.members.is_empty() {
            self.game = None;
            self.lobby.in_game = false;
            self.lobby.difficulty = Difficulty::default();
        }
    }

    pub fn command(&mut self, id: u64, command: &ClientMessage) -> Result<(), String> {
        let member = self
            .lobby
            .members
            .iter_mut()
            .find(|m| m.id == id)
            .ok_or("Join first")?;
        match command {
            ClientMessage::Difficulty(difficulty) if self.game.is_none() => {
                if id != self.lobby.host {
                    return Err("Only the host can change difficulty".into());
                }
                if !difficulty.valid() {
                    return Err("Difficulty must use 5% steps within the allowed ranges".into());
                }
                if self.lobby.difficulty != *difficulty {
                    self.lobby.difficulty = *difficulty;
                    for member in &mut self.lobby.members {
                        member.ready = false;
                    }
                }
            }
            ClientMessage::Ready(ready) if self.game.is_none() => {
                member.ready = *ready;
            }
            ClientMessage::Start if self.game.is_none() => {
                if id != self.lobby.host {
                    return Err("Only the host can start the match".into());
                }
                if !self.lobby.members.iter().all(|m| m.ready) {
                    return Err("Every player must be ready".into());
                }
                let mut game = Game::new_party(
                    self.level.clone(),
                    self.lobby.endless,
                    self.lobby.members.len(),
                )?;
                game.difficulty = self.lobby.difficulty;
                // Stable colors survive lobby departures; inputs use member order.
                game.player.slot = self.lobby.members[0].slot;
                game.player.name = self.lobby.members[0].name.clone();
                for (player, member) in game
                    .allies
                    .iter_mut()
                    .zip(self.lobby.members.iter().skip(1))
                {
                    player.slot = member.slot;
                    player.name = member.name.clone();
                }
                self.game = Some(game);
                self.lobby.in_game = true;
            }
            ClientMessage::ReturnToLobby => {
                if id != self.lobby.host {
                    return Err("Only the host can return the team to the lobby".into());
                }
                if self
                    .game
                    .as_ref()
                    .is_none_or(|g| g.team_alive() && !g.victory)
                {
                    return Err("The match is still running".into());
                }
                self.game = None;
                self.lobby.in_game = false;
                for member in &mut self.lobby.members {
                    member.ready = false;
                }
            }
            _ => return Err("That action is unavailable right now".into()),
        }
        Ok(())
    }

    pub fn map_text(&self) -> String {
        let mut tiles = self.level.tiles.clone();
        for &pos in &self.level.crates {
            tiles[self.level.index(pos)] = 'C';
        }
        tiles
            .chunks(self.level.width)
            .map(|r| r.iter().collect::<String>() + "\n")
            .collect()
    }
}

pub fn sanitize_input(mut input: Input) -> Result<Input, String> {
    if !input.movement.is_finite()
        || !input.aim.is_finite()
        || input.movement.abs().max_element() > 2.0
        || input.aim.abs().max_element() > 1_000_000.0
    {
        return Err("Invalid input coordinates".into());
    }
    input.movement = input.movement.clamp_length_max(1.0);
    if input.weapon.is_some_and(|w| w >= WEAPONS.len()) {
        return Err("Unknown weapon".into());
    }
    Ok(input)
}
