//! Desktop smoke scenario using a real local server and seven socket clients.
use crate::online_ui::OnlineUi;
use macroquad::prelude::*;
use mgame::{
    level::Level,
    party::{ClientMessage, Room},
    party_client::PartyClient,
    party_server::PartyServer,
    simulation::{Enemy, Kind},
};
use std::{
    net::TcpStream,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

pub struct Smoke {
    address: std::net::SocketAddr,
    connected: bool,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
    start: Instant,
    stage: u8,
}

impl Smoke {
    pub fn new() -> (Self, OnlineUi) {
        let mut server = PartyServer::bind(
            "127.0.0.1:0".parse().unwrap(),
            Room::new(Level::builtin(0), false).unwrap(),
        )
        .unwrap();
        let address = server.local_addr().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();
        let thread = std::thread::spawn(move || {
            let mut bots = vec![];
            let mut chosen = false;
            let mut downed = false;
            while !thread_stop.load(Ordering::Relaxed) {
                server.poll().unwrap();
                if server.room.lobby.members.len() == 1 && bots.is_empty() {
                    for name in [
                        "Ember", "Cobalt", "Violet", "Gold", "Rose", "Lime", "Silver",
                    ] {
                        bots.push(
                            PartyClient::new(TcpStream::connect(address).unwrap(), name.into())
                                .unwrap(),
                        );
                    }
                }
                for bot in &mut bots {
                    bot.poll().unwrap();
                }
                if !chosen && bots.len() == 7 && bots.iter().all(|b| b.lobby.is_some()) {
                    for (i, bot) in bots.iter_mut().enumerate() {
                        bot.send(ClientMessage::Weapon((i + 1) % 5)).unwrap();
                        bot.send(ClientMessage::Ready(true)).unwrap();
                    }
                    chosen = true;
                }
                if let Some(game) = &mut server.room.game {
                    game.intermission = 100.0;
                    if game.time > 2.0 && !downed {
                        game.allies[0].health = 0.0;
                        let pos = game.player.pos;
                        game.enemies.push(Enemy {
                            pos,
                            previous: pos,
                            kind: Kind::Runner,
                            health: 0.0,
                            max_health: 48.0,
                            attack: 100.0,
                            tell: 0.0,
                            aim: Vec2::X,
                            phase: 0.0,
                        });
                        downed = true;
                    }
                    if game.time > 13.0 {
                        assert!(game.allies[0].health > 0.0, "Smoke revive did not finish");
                        game.victory = true;
                    }
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        });
        let ui = OnlineUi::new();
        (
            Self {
                address,
                connected: false,
                stop,
                thread: Some(thread),
                start: Instant::now(),
                stage: 0,
            },
            ui,
        )
    }

    /// Called after rendering; captures the actual client view, not a fabricated scene.
    pub fn frame(&mut self, ui: &mut OnlineUi) -> bool {
        assert!(
            self.start.elapsed() < Duration::from_secs(45),
            "Party smoke timed out"
        );
        if !self.connected {
            get_screen_data().export_png("target/smoke-party-connect.png");
            ui.connect(Some(self.address));
            self.connected = true;
            return false;
        }
        let Some(client) = &mut ui.client else {
            return false;
        };
        match self.stage {
            0 if client.lobby.as_ref().is_some_and(|l| l.members.len() == 8) => {
                client.send(ClientMessage::Weapon(0)).unwrap();
                client.send(ClientMessage::Ready(true)).unwrap();
                self.stage = 1;
            }
            1 if client
                .lobby
                .as_ref()
                .is_some_and(|l| l.members.iter().all(|m| m.ready)) =>
            {
                get_screen_data().export_png("target/smoke-party-lobby.png");
                client.send(ClientMessage::Start).unwrap();
                self.stage = 2;
            }
            2 if client.game.as_ref().is_some_and(|g| g.time > 1.0) => {
                assert_eq!(client.game.as_ref().unwrap().players().count(), 8);
                get_screen_data().export_png("target/smoke-party-game.png");
                self.stage = 3;
            }
            3 if client
                .game
                .as_ref()
                .is_some_and(|g| g.beacons.iter().any(|b| b.progress > 4.0)) =>
            {
                get_screen_data().export_png("target/smoke-party-revive.png");
                self.stage = 4;
            }
            4 if client.game.as_ref().is_some_and(|g| g.victory) => {
                get_screen_data().export_png("target/smoke-party-results.png");
                client.send(ClientMessage::ReturnToLobby).unwrap();
                self.stage = 5;
            }
            5 if client.game.is_none() => return true,
            _ => {}
        }
        false
    }
}

impl Drop for Smoke {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}
