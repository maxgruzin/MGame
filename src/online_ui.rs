use crate::{
    assets::Assets,
    button,
    render::{self, text, MINT, MUTED, PANEL},
    settings::Settings,
    sound::Audio,
    ui_begin,
};
use macroquad::prelude::*;
use mgame::{
    party::{ClientMessage, COLORS, DEFAULT_PORT},
    party_client::PartyClient,
    simulation::{Input, WEAPONS},
};
use std::{
    net::{IpAddr, SocketAddr, TcpStream},
    sync::mpsc::{self, Receiver},
    time::Duration,
};

pub struct OnlineUi {
    fields: [String; 3],
    active: usize,
    cursor: usize,
    select_all: bool,
    connecting: Option<Receiver<Result<TcpStream, String>>>,
    pub client: Option<PartyClient>,
    notice: String,
    menu: bool,
    settings: bool,
    camera: Vec2,
    was_playing: bool,
    send_timer: f32,
    dash: bool,
    reload: bool,
    spectate: usize,
}

impl OnlineUi {
    pub fn new() -> Self {
        Self {
            fields: [
                "127.0.0.1".into(),
                DEFAULT_PORT.to_string(),
                "Player".into(),
            ],
            active: 0,
            cursor: 9,
            select_all: true,
            connecting: None,
            client: None,
            notice: String::new(),
            menu: false,
            settings: false,
            camera: Vec2::ZERO,
            was_playing: false,
            send_timer: 0.0,
            dash: false,
            reload: false,
            spectate: 0,
        }
    }

    pub fn connect(&mut self, address: Option<SocketAddr>) {
        let parsed = (|| {
            if let Some(address) = address {
                return Ok(address);
            }
            let ip: IpAddr = self.fields[0]
                .trim()
                .parse()
                .map_err(|_| "Enter a valid IPv4 or IPv6 address")?;
            let port: u16 = self.fields[1]
                .parse()
                .map_err(|_| "Port must be between 1 and 65535")?;
            if port == 0 {
                return Err("Port must be between 1 and 65535");
            }
            Ok(SocketAddr::new(ip, port))
        })();
        match parsed {
            Ok(address) => {
                if self.fields[2].trim().is_empty() {
                    self.notice = "Enter your player name".into();
                    return;
                }
                let (tx, rx) = mpsc::channel();
                self.connecting = Some(rx);
                self.notice = format!("Connecting to {address}...");
                std::thread::spawn(move || {
                    let result = TcpStream::connect_timeout(&address, Duration::from_secs(4))
                        .map_err(|e| format!("Cannot connect: {e}"));
                    let _ = tx.send(result);
                });
            }
            Err(error) => self.notice = error.into(),
        }
    }

    fn send(&mut self, message: ClientMessage) {
        if let Some(client) = &mut self.client {
            client.notice.clear();
            if let Err(error) = client.send(message) {
                self.notice = error;
                self.client = None;
            }
        }
    }

    fn network(&mut self) {
        if let Some(receiver) = &self.connecting {
            if let Ok(result) = receiver.try_recv() {
                self.connecting = None;
                match result.and_then(|stream| {
                    PartyClient::new(stream, self.fields[2].clone()).map_err(|e| e.to_string())
                }) {
                    Ok(client) => {
                        self.client = Some(client);
                        self.notice.clear();
                    }
                    Err(error) => self.notice = error,
                }
            }
        }
        if let Some(client) = &mut self.client {
            if let Err(error) = client.poll() {
                self.notice = if client.notice.is_empty() {
                    error
                } else {
                    client.notice.clone()
                };
                self.client = None;
                self.menu = false;
                self.settings = false;
            }
        }
    }

    pub fn frame(
        &mut self,
        assets: &Assets,
        audio: &mut Audio,
        settings: &mut Settings,
        dt: f32,
        scale: f32,
    ) -> bool {
        self.network();
        let playing = self.client.as_ref().is_some_and(|c| c.game.is_some());
        if playing && !self.was_playing {
            self.camera = self
                .client
                .as_ref()
                .unwrap()
                .game
                .as_ref()
                .unwrap()
                .player
                .pos;
            self.menu = false;
            self.settings = false;
        }
        self.was_playing = playing;
        if playing {
            self.play(assets, audio, settings, dt, scale);
        } else {
            audio.update_music(false, settings.music_volume);
            show_mouse(true);
            let (w, h) = ui_begin(scale);
            let x = w * 0.5 - 530.0;
            text("ONLINE PARTY", x, 84.0, 48.0, WHITE);
            text(
                "One squad. One weapon each. Survive together.",
                x,
                120.0,
                21.0,
                MUTED,
            );
            if self.client.as_ref().is_some_and(|c| c.lobby.is_some()) {
                self.lobby(x, scale);
            } else {
                self.connection_form(w * 0.5 - 260.0, scale);
            }
            if button(
                Rect::new(x, h - 65.0, 235.0, 40.0),
                "BACK TO ARENA MENU",
                scale,
                false,
            ) || is_key_pressed(KeyCode::Escape)
            {
                return true;
            }
            self.notice_line(x, h - 90.0);
            set_default_camera();
        }
        false
    }

    fn connection_form(&mut self, x: f32, scale: f32) {
        let busy = self.connecting.is_some() || self.client.is_some();
        for (i, label) in ["SERVER IP (IPv4 / IPv6)", "PORT", "YOUR NAME"]
            .iter()
            .enumerate()
        {
            let y = 180.0 + i as f32 * 94.0;
            text(label, x, y, 18.0, MUTED);
            let rect = Rect::new(x, y + 12.0, 520.0, 46.0);
            if !busy && button(rect, "", scale, self.active == i) {
                self.active = i;
                self.cursor = self.fields[i].len();
                self.select_all = true;
            }
            let display = if !busy && self.active == i {
                format!(
                    "{}|{}",
                    &self.fields[i][..self.cursor],
                    &self.fields[i][self.cursor..]
                )
            } else {
                self.fields[i].clone()
            };
            text(
                &display,
                x + 14.0,
                y + 43.0,
                22.0,
                if self.active == i && self.select_all {
                    MINT
                } else {
                    WHITE
                },
            );
        }
        if !busy {
            self.type_field();
            if button(
                Rect::new(x, 494.0, 520.0, 54.0),
                "CONNECT TO WAITING ROOM",
                scale,
                true,
            ) || (is_key_pressed(KeyCode::Enter)
                && !is_key_down(KeyCode::LeftAlt)
                && !is_key_down(KeyCode::RightAlt))
            {
                self.connect(None);
            }
        } else {
            text("Connecting / joining...", x, 526.0, 23.0, MINT);
            if button(Rect::new(x, 555.0, 520.0, 40.0), "CANCEL", scale, false) {
                self.connecting = None;
                self.client = None;
                self.notice.clear();
            }
        }
        text(
            "Tab: next field   Ctrl+A: select all   Ctrl+V: paste",
            x,
            586.0,
            17.0,
            MUTED,
        );
    }

    fn type_field(&mut self) {
        if is_key_pressed(KeyCode::Tab) {
            self.active = (self.active + 1) % 3;
            self.cursor = self.fields[self.active].len();
            self.select_all = true;
        }
        let ctrl = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
        let value = &mut self.fields[self.active];
        if ctrl && is_key_pressed(KeyCode::A) {
            self.select_all = true;
        }
        if is_key_pressed(KeyCode::Left) {
            self.cursor = self.cursor.saturating_sub(1);
            self.select_all = false;
        }
        if is_key_pressed(KeyCode::Right) {
            self.cursor = (self.cursor + 1).min(value.len());
            self.select_all = false;
        }
        if is_key_pressed(KeyCode::Home) {
            self.cursor = 0;
            self.select_all = false;
        }
        if is_key_pressed(KeyCode::End) {
            self.cursor = value.len();
            self.select_all = false;
        }
        if is_key_pressed(KeyCode::Backspace) || is_key_pressed(KeyCode::Delete) {
            if self.select_all {
                value.clear();
                self.cursor = 0;
                self.select_all = false;
            } else if is_key_pressed(KeyCode::Backspace) && self.cursor > 0 {
                self.cursor -= 1;
                value.remove(self.cursor);
            } else if is_key_pressed(KeyCode::Delete) && self.cursor < value.len() {
                value.remove(self.cursor);
            }
        }
        let mut inserted = String::new();
        if ctrl && is_key_pressed(KeyCode::V) {
            inserted = macroquad::miniquad::window::clipboard_get().unwrap_or_default();
        }
        while let Some(c) = get_char_pressed() {
            if !ctrl {
                inserted.push(c);
            }
        }
        inserted.retain(|c| match self.active {
            0 => c.is_ascii_hexdigit() || matches!(c, ':' | '.'),
            1 => c.is_ascii_digit(),
            _ => c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_'),
        });
        if !inserted.is_empty() {
            if self.select_all {
                value.clear();
                self.cursor = 0;
                self.select_all = false;
            }
            let limit: usize = [45, 5, 20][self.active];
            inserted.truncate(limit.saturating_sub(value.len()));
            value.insert_str(self.cursor, &inserted);
            self.cursor += inserted.len();
        }
    }

    fn lobby(&mut self, x: f32, scale: f32) {
        let client = self.client.as_ref().unwrap();
        let lobby = client.lobby.as_ref().unwrap().clone();
        let id = client.id;
        let local = lobby.members.iter().find(|m| m.id == id);
        let weapon = local.and_then(|m| m.weapon);
        let ready = local.is_some_and(|m| m.ready);
        text(
            &format!("WAITING ROOM  /  {} OF 8", lobby.members.len()),
            x,
            175.0,
            24.0,
            MINT,
        );
        text(
            &format!(
                "{}  /  {}",
                lobby.arena,
                if lobby.endless { "Endless" } else { "20 waves" }
            ),
            x,
            205.0,
            18.0,
            MUTED,
        );
        for (i, member) in lobby.members.iter().enumerate() {
            let y = 247.0 + i as f32 * 43.0;
            let color = Color::from_hex(COLORS[member.slot as usize % 8]);
            draw_rectangle(x, y - 24.0, 562.0, 37.0, PANEL);
            draw_circle(x + 14.0, y - 6.0, 6.0, color);
            let name = if member.name.chars().count() > 12 {
                format!("{}..", member.name.chars().take(10).collect::<String>())
            } else {
                member.name.clone()
            };
            text(
                &format!(
                    "P{} {}{}",
                    member.slot + 1,
                    name,
                    if member.id == lobby.host { " *" } else { "" }
                ),
                x + 29.0,
                y,
                16.0,
                color,
            );
            text(
                member.weapon.map_or("Choose weapon", |w| WEAPONS[w].name),
                x + 232.0,
                y,
                15.0,
                WHITE,
            );
            text(
                if member.ready { "READY" } else { "WAITING" },
                x + 475.0,
                y,
                15.0,
                if member.ready { MINT } else { MUTED },
            );
        }
        let right = x + 605.0;
        text(
            "YOUR WEAPON / LOCKED FOR THE MATCH",
            right,
            175.0,
            19.0,
            MINT,
        );
        for (i, weapon_info) in WEAPONS.iter().enumerate() {
            if button(
                Rect::new(right, 194.0 + i as f32 * 48.0, 455.0, 40.0),
                weapon_info.name,
                scale,
                weapon == Some(i),
            ) {
                self.send(ClientMessage::Weapon(i));
            }
        }
        if button(
            Rect::new(right, 450.0, 455.0, 45.0),
            if ready {
                "READY / CLICK TO UNREADY"
            } else {
                "I AM READY"
            },
            scale,
            ready,
        ) {
            self.send(ClientMessage::Ready(!ready));
        }
        let all_ready = !lobby.members.is_empty()
            && lobby.members.iter().all(|m| m.ready && m.weapon.is_some());
        if id == lobby.host {
            if button(
                Rect::new(right, 507.0, 455.0, 50.0),
                if all_ready {
                    "START MATCH"
                } else {
                    "START / WAITING FOR EVERYONE"
                },
                scale,
                all_ready,
            ) {
                self.send(ClientMessage::Start);
            }
        } else {
            text(
                "The host (*) starts when everyone is ready.",
                right,
                537.0,
                18.0,
                MUTED,
            );
        }
        let count = lobby.members.len().max(1);
        text(
            &format!(
                "{}x enemies / +{}% health & damage",
                count,
                (count - 1) * 25
            ),
            right,
            586.0,
            18.0,
            MUTED,
        );
        text(
            "Changing your weapon clears your ready status.",
            right,
            612.0,
            17.0,
            MUTED,
        );
    }

    fn play(
        &mut self,
        assets: &Assets,
        audio: &mut Audio,
        settings: &mut Settings,
        dt: f32,
        scale: f32,
    ) {
        if !self.settings && is_key_pressed(KeyCode::Escape) {
            self.menu = !self.menu;
        }
        let client = self.client.as_mut().unwrap();
        let game = client.game.as_mut().unwrap();
        let ended = game.victory || !game.team_alive();
        let fallen = game.player.health <= 0.0;
        let active = !self.menu && !self.settings && !ended && !fallen && crate::app_focused();
        let living: Vec<_> = game
            .players()
            .filter(|p| p.connected && p.health > 0.0)
            .collect();
        if is_key_pressed(KeyCode::Tab) {
            self.spectate += 1;
        }
        let followed = if fallen && !living.is_empty() {
            living[self.spectate % living.len()]
        } else {
            &game.player
        };
        let target = followed.pos;
        self.camera = self.camera.lerp(target, 1.0 - (-10.0 * dt).exp());
        let camera = render::camera(self.camera, scale);
        self.dash |=
            active && (is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::LeftShift));
        self.reload |= active && is_key_pressed(KeyCode::R);
        if !active {
            self.dash = false;
            self.reload = false;
        }
        let input = if active {
            Input {
                movement: vec2(
                    (is_key_down(KeyCode::D) as i32 - is_key_down(KeyCode::A) as i32) as f32,
                    (is_key_down(KeyCode::S) as i32 - is_key_down(KeyCode::W) as i32) as f32,
                ),
                aim: camera.screen_to_world(Vec2::from(mouse_position())),
                fire: is_mouse_button_down(MouseButton::Left),
                dash: self.dash,
                reload: self.reload,
                weapon: None,
            }
        } else {
            Input::default()
        };
        audio.play(&game.sounds, settings.sound, settings.sound_volume);
        game.sounds.clear();
        audio.update_music(settings.sound && !ended, settings.music_volume);
        let interpolation = (client.snapshot_at.elapsed().as_secs_f32() / 0.05).min(1.0);
        render::world(game, assets, &camera, interpolation, settings.effects);
        render::hud(game, scale);
        let (w, h) = ui_begin(scale);
        text(
            &format!("ONLINE PARTY / {} ms", client.ping_ms),
            28.0,
            114.0,
            16.0,
            MINT,
        );
        if client.snapshot_at.elapsed() > Duration::from_secs(1) {
            text(
                "CONNECTION STALLED...",
                w * 0.5 - 130.0,
                180.0,
                24.0,
                ORANGE,
            );
        }
        for (i, member) in client.members.iter().enumerate() {
            let player = game.players().find(|p| p.slot == member.slot);
            let status = player.map_or_else(
                || "LEFT".into(),
                |p| {
                    if !p.connected {
                        "LEFT".into()
                    } else if p.health <= 0.0 {
                        "DOWN".into()
                    } else {
                        format!("{:.0} HP", p.health)
                    }
                },
            );
            text(
                &format!("P{} {}  {}", member.slot + 1, member.name, status),
                28.0,
                268.0 + i as f32 * 24.0,
                16.0,
                Color::from_hex(COLORS[member.slot as usize % 8]),
            );
        }
        if fallen && !ended {
            draw_rectangle(w * 0.5 - 285.0, h - 174.0, 570.0, 55.0, PANEL);
            text(
                "DOWN / TAB TO SPECTATE AN ALLY",
                w * 0.5 - 245.0,
                h - 150.0,
                20.0,
                MINT,
            );
            text(
                "Your team can revive you by holding an enemy-dropped beacon.",
                w * 0.5 - 245.0,
                h - 130.0,
                16.0,
                MUTED,
            );
        }
        let is_host = client.lobby.as_ref().is_some_and(|l| l.host == client.id);
        let mut return_lobby = false;
        let mut leave = false;
        if self.menu || ended {
            draw_rectangle(w * 0.5 - 280.0, h * 0.5 - 145.0, 560.0, 290.0, PANEL);
            let x = w * 0.5 - 245.0;
            let y = h * 0.5 - 98.0;
            text(
                if game.victory {
                    "SECTOR SECURED"
                } else if ended {
                    "SQUAD LOST"
                } else {
                    "PARTY MENU"
                },
                x,
                y,
                32.0,
                MINT,
            );
            text(
                if ended {
                    "The host can return everyone to the waiting room."
                } else {
                    "The online match continues while this menu is open."
                },
                x,
                y + 29.0,
                17.0,
                MUTED,
            );
            if !ended {
                if button(Rect::new(x, y + 47.0, 490.0, 42.0), "RESUME", scale, true) {
                    self.menu = false;
                }
            } else if is_host
                && button(
                    Rect::new(x, y + 47.0, 490.0, 42.0),
                    "RETURN TEAM TO WAITING ROOM",
                    scale,
                    true,
                )
            {
                return_lobby = true;
            }
            if button(
                Rect::new(x, y + 100.0, 490.0, 42.0),
                "DISPLAY & AUDIO",
                scale,
                false,
            ) {
                self.settings = true;
            }
            leave = button(
                Rect::new(x, y + 153.0, 490.0, 42.0),
                "LEAVE PARTY",
                scale,
                false,
            );
        }
        self.notice_line(28.0, h - 140.0);
        set_default_camera();
        show_mouse(self.menu || ended || self.settings || fallen);
        if active {
            render::crosshair();
        }
        self.send_timer += dt;
        if self.send_timer >= 1.0 / 30.0 {
            self.send_timer = 0.0;
            self.send(ClientMessage::Input(input));
            self.dash = false;
            self.reload = false;
        }
        if return_lobby {
            self.send(ClientMessage::ReturnToLobby);
            self.menu = false;
        }
        if leave {
            self.client = None;
            self.notice = "Left the party".into();
            self.menu = false;
        }
        if self.settings {
            let (w, h) = ui_begin(scale);
            draw_rectangle(0.0, 0.0, w, h, Color::new(0.01, 0.02, 0.03, 0.98));
            if crate::settings_panel(settings, scale, w, h) {
                self.settings = false;
                if let Err(error) = settings.save() {
                    self.notice = error;
                }
            }
            set_default_camera();
        }
    }

    fn notice_line(&self, x: f32, y: f32) {
        let notice = self
            .client
            .as_ref()
            .filter(|c| !c.notice.is_empty())
            .map_or(self.notice.as_str(), |c| c.notice.as_str());
        for (i, line) in crate::wrap(notice, 110).iter().take(2).enumerate() {
            text(line, x, y + i as f32 * 20.0, 17.0, ORANGE);
        }
    }
}
