use crate::level::{Level, TILE};
use crate::waves;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

pub const STEP: f32 = 1.0 / 120.0;
pub const PLAYER_RADIUS: f32 = 13.0;

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct Input {
    pub movement: Vec2,
    pub aim: Vec2,
    pub fire: bool,
    pub dash: bool,
    pub reload: bool,
    pub weapon: Option<usize>,
}
#[derive(Clone, Copy)]
pub struct Weapon {
    pub name: &'static str,
    pub magazine: u32,
    pub interval: f32,
    pub reload: f32,
    pub damage: f32,
    pub pellets: usize,
    pub spread: f32,
    pub speed: f32,
}
pub const WEAPONS: [Weapon; 5] = [
    Weapon {
        name: "PULSE RIFLE",
        magazine: 30,
        interval: 0.105,
        reload: 1.15,
        damage: 24.0,
        pellets: 1,
        spread: 0.025,
        speed: 1100.0,
    },
    Weapon {
        name: "BREACH SHOTGUN",
        magazine: 7,
        interval: 0.58,
        reload: 1.55,
        damage: 21.0,
        pellets: 8,
        spread: 0.25,
        speed: 920.0,
    },
    Weapon {
        name: "RAIL CARBINE",
        magazine: 8,
        interval: 0.36,
        reload: 1.05,
        damage: 230.0,
        pellets: 1,
        spread: 0.0,
        speed: 1900.0,
    },
    Weapon {
        name: "ROCKET LAUNCHER",
        magazine: 4,
        interval: 0.8,
        reload: 1.8,
        damage: 150.0,
        pellets: 1,
        spread: 0.0,
        speed: 620.0,
    },
    Weapon {
        name: "GRENADE LAUNCHER",
        magazine: 5,
        interval: 0.65,
        reload: 1.65,
        damage: 175.0,
        pellets: 1,
        spread: 0.0,
        speed: 0.0,
    },
];

/// Whole percentages; the server accepts only the same 5% steps as the lobby UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Difficulty {
    pub enemy_count: u16,
    pub enemy_damage: u16,
    pub enemy_health: u16,
    pub weapon_damage: [i16; 5],
}

impl Default for Difficulty {
    fn default() -> Self {
        Self {
            enemy_count: 75,
            enemy_damage: 15,
            enemy_health: 15,
            weapon_damage: [0; 5],
        }
    }
}

impl Difficulty {
    pub fn valid(&self) -> bool {
        [self.enemy_count, self.enemy_damage, self.enemy_health]
            .iter()
            .all(|&p| p <= 300 && p % 5 == 0)
            && self
                .weapon_damage
                .iter()
                .all(|&p| (-50..=100).contains(&p) && p % 5 == 0)
    }

    pub fn scale(players: usize, percent: u16) -> f32 {
        1.0 + players.saturating_sub(1) as f32 * percent as f32 / 100.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub slot: u8,
    pub name: String,
    pub connected: bool,
    pub pos: Vec2,
    pub previous: Vec2,
    pub velocity: Vec2,
    pub aim: Vec2,
    pub walk_distance: f32,
    pub health: f32,
    pub weapon: usize,
    pub ammo: [u32; 5],
    pub cooldown: f32,
    pub reload: f32,
    pub dash_cooldown: f32,
    pub dash_time: f32,
    pub dash_direction: Vec2,
    pub invulnerable: f32,
    pub quad: f32,
    pub immortal: f32,
    pub haste: f32,
}
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Kind {
    Runner,
    Gunner,
    Brute,
    Hunter,
    Bomber,
    Siege,
    Stalker,
    Lancer,
    Sentinel,
    ThornTyrant,
    BroodMatriarch,
    Overlord,
}
impl Kind {
    pub fn is_boss(self) -> bool {
        matches!(
            self,
            Self::ThornTyrant | Self::BroodMatriarch | Self::Overlord
        )
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Runner => "RAVAGER",
            Self::Gunner => "SPITTER",
            Self::Brute => "THORNBACK",
            Self::Hunter => "HUNTER",
            Self::Bomber => "BROOD BOMBER",
            Self::Siege => "SIEGE BEAST",
            Self::Stalker => "STALKER",
            Self::Lancer => "LANCER",
            Self::Sentinel => "SENTINEL",
            Self::ThornTyrant => "THORN TYRANT",
            Self::BroodMatriarch => "BROOD MATRIARCH",
            Self::Overlord => "OVERLORD",
        }
    }
    pub fn base_health(self) -> f32 {
        match self {
            Self::Runner => 48.0,
            Self::Gunner => 65.0,
            Self::Brute => 190.0,
            Self::Hunter => 110.0,
            Self::Bomber => 150.0,
            Self::Siege => 310.0,
            Self::Stalker => 90.0,
            Self::Lancer => 135.0,
            Self::Sentinel => 210.0,
            Self::ThornTyrant => 1980.0,
            Self::BroodMatriarch => 3150.0,
            Self::Overlord => 4800.0,
        }
    }
    pub fn radius(self) -> f32 {
        // Boss artwork is larger, but fits the same narrow forest paths as brutes.
        if self.is_boss() || matches!(self, Self::Brute | Self::Siege | Self::Sentinel) {
            19.0
        } else {
            13.0
        }
    }
    pub fn color(self) -> Color {
        match self {
            Self::Runner => Color::from_hex(0xff5c77),
            Self::Gunner => Color::from_hex(0xffc66d),
            Self::Brute => Color::from_hex(0xb499ff),
            Self::Hunter => Color::from_hex(0xa7e3ff),
            Self::Bomber => Color::from_hex(0xc7f272),
            Self::Siege => Color::from_hex(0xff814f),
            Self::Stalker => Color::from_hex(0x63f5d0),
            Self::Lancer => Color::from_hex(0xe78bff),
            Self::Sentinel => Color::from_hex(0x78b8ff),
            Self::ThornTyrant => Color::from_hex(0xffc15c),
            Self::BroodMatriarch => Color::from_hex(0xdd88ff),
            Self::Overlord => Color::from_hex(0xff6355),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub pos: Vec2,
    pub previous: Vec2,
    pub kind: Kind,
    pub health: f32,
    pub max_health: f32,
    pub attack: f32,
    pub tell: f32,
    pub aim: Vec2,
    pub phase: f32,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Bullet {
    pub pos: Vec2,
    pub previous: Vec2,
    pub origin: Vec2,
    pub velocity: Vec2,
    pub damage: f32,
    pub life: f32,
    pub hostile: bool,
    pub rail: bool,
    pub blast_radius: f32,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Grenade {
    pub pos: Vec2,
    pub origin: Vec2,
    pub target: Vec2,
    pub age: f32,
    pub fuse: f32,
    pub damage: f32,
    pub hostile: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Blast {
    pub pos: Vec2,
    pub radius: f32,
    pub life: f32,
    pub hostile: bool,
}
pub struct Particle {
    pub pos: Vec2,
    pub velocity: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
    pub size: f32,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Pickup {
    pub pos: Vec2,
    pub life: f32,
    pub kind: PickupKind,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PickupKind {
    Health,
    Quad,
    Immortal,
    Haste,
}
impl PickupKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Health => "HEALTH",
            Self::Quad => "QUAD DAMAGE",
            Self::Immortal => "IMMORTAL",
            Self::Haste => "2X SPEED",
        }
    }
    pub fn color(self) -> Color {
        match self {
            Self::Health => Color::from_hex(0x85ebad),
            Self::Quad => Color::from_hex(0xe191ff),
            Self::Immortal => Color::from_hex(0xffe080),
            Self::Haste => Color::from_hex(0x7bdaff),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Spawn {
    pub pos: Vec2,
    pub timer: f32,
    pub kind: Kind,
}
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum SoundEvent {
    Rifle,
    Shotgun,
    Rail,
    Hit,
    Hurt,
    Dash,
    Wave,
    Boss,
    Explosion,
    Pickup,
}
pub struct Game {
    pub level: Level,
    pub player: Player,
    pub allies: Vec<Player>,
    /// Zero for solo; fixed starting headcount for online difficulty.
    pub party_size: usize,
    pub difficulty: Difficulty,
    pub beacons: Vec<ReviveBeacon>,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub grenades: Vec<Grenade>,
    pub blasts: Vec<Blast>,
    pub particles: Vec<Particle>,
    pub pickups: Vec<Pickup>,
    pub spawns: Vec<Spawn>,
    pub wave: u32,
    pub remaining: u32,
    pub intermission: f32,
    pub spawn_timer: f32,
    pub wave_announcement: f32,
    pub score: u32,
    pub kills: u32,
    pub combo: u32,
    pub combo_timer: f32,
    pub time: f32,
    pub shake: f32,
    pub hit_flash: f32,
    pub sounds: Vec<SoundEvent>,
    pub victory: bool,
    pub endless: bool,
    field: Vec<u32>,
    ally_fields: Vec<Vec<u32>>,
    field_timer: f32,
    rng: u64,
}

pub const REVIVE_RADIUS: f32 = 64.0;
pub const REVIVE_SECONDS: f32 = 10.0;

#[derive(Clone, Serialize, Deserialize)]
pub struct ReviveBeacon {
    pub pos: Vec2,
    pub progress: f32,
}

impl Game {
    pub fn new(level: Level, endless: bool) -> Self {
        let pos = level.spawn;
        let field = level.flow_field(pos);
        Self {
            level,
            player: Player {
                slot: 0,
                name: String::new(),
                connected: true,
                pos,
                previous: pos,
                velocity: Vec2::ZERO,
                aim: Vec2::X,
                walk_distance: 0.0,
                health: 100.0,
                weapon: 0,
                ammo: WEAPONS.map(|weapon| weapon.magazine),
                cooldown: 0.0,
                reload: 0.0,
                dash_cooldown: 0.0,
                dash_time: 0.0,
                dash_direction: Vec2::X,
                invulnerable: 0.0,
                quad: 0.0,
                immortal: 0.0,
                haste: 0.0,
            },
            allies: vec![],
            party_size: 0,
            difficulty: Difficulty::default(),
            beacons: vec![],
            enemies: vec![],
            bullets: vec![],
            grenades: vec![],
            blasts: vec![],
            particles: vec![],
            pickups: vec![],
            spawns: vec![],
            wave: 0,
            remaining: 0,
            intermission: 2.0,
            spawn_timer: 0.0,
            wave_announcement: 0.0,
            score: 0,
            kills: 0,
            combo: 0,
            combo_timer: 0.0,
            time: 0.0,
            shake: 0.0,
            hit_flash: 0.0,
            sounds: vec![],
            victory: false,
            endless,
            field,
            ally_fields: vec![],
            field_timer: 0.0,
            rng: 0xabc123def456,
        }
    }
    pub fn players(&self) -> impl Iterator<Item = &Player> {
        std::iter::once(&self.player).chain(self.allies.iter())
    }

    pub fn party_count_scale(&self) -> f32 {
        Difficulty::scale(self.party_size, self.difficulty.enemy_count)
    }

    pub fn team_alive(&self) -> bool {
        self.players().any(|p| p.connected && p.health > 0.0)
    }

    fn spawn_reachable(&self, index: usize) -> bool {
        (self.player.connected && self.player.health > 0.0 && self.field[index] != u32::MAX)
            || self
                .allies
                .iter()
                .zip(&self.ally_fields)
                .any(|(p, field)| p.connected && p.health > 0.0 && field[index] != u32::MAX)
    }

    fn distance_to_team(&self, pos: Vec2) -> f32 {
        self.players()
            .filter(|p| p.connected && p.health > 0.0)
            .map(|p| p.pos.distance(pos))
            .fold(f32::INFINITY, f32::min)
    }

    pub fn new_party(level: Level, endless: bool, count: usize) -> Result<Self, String> {
        if !(1..=8).contains(&count) {
            return Err("A party requires 1-8 players".into());
        }
        let mut game = Self::new(level, endless);
        game.party_size = count;
        for index in 1..count {
            let Some(pos) = game.free_party_spawn(game.level.spawn) else {
                return Err("The arena has no safe room for this party".into());
            };
            let mut player = game.player.clone();
            player.slot = index as u8;
            player.pos = pos;
            player.previous = pos;
            game.ally_fields.push(game.level.flow_field(pos));
            game.allies.push(player);
        }
        Ok(game)
    }

    fn free_party_spawn(&self, origin: Vec2) -> Option<Vec2> {
        let field = self.level.flow_field(origin);
        (0..self.level.tiles.len())
            .filter(|&i| field[i] != u32::MAX)
            .map(|i| self.level.center(i))
            .filter(|&p| {
                self.level.circle_free(p, PLAYER_RADIUS)
                    && p.distance(origin) <= TILE * 5.0
                    && self
                        .players()
                        .filter(|a| a.connected && a.health > 0.0)
                        .all(|a| a.pos.distance(p) >= PLAYER_RADIUS * 2.0 + 4.0)
                    && self
                        .enemies
                        .iter()
                        .all(|e| e.pos.distance(p) > e.kind.radius() + PLAYER_RADIUS + 10.0)
            })
            .min_by(|a, b| {
                a.distance_squared(origin)
                    .total_cmp(&b.distance_squared(origin))
            })
    }

    /// Inputs are indexed by the immutable starting roster. The world advances once.
    pub fn update_party(&mut self, inputs: &[Input], dt: f32) {
        if !self.team_alive() || self.victory {
            return;
        }
        self.begin_frame(dt);
        self.update_player(inputs.first().copied().unwrap_or_default(), dt);
        for i in 0..self.allies.len() {
            std::mem::swap(&mut self.player, &mut self.allies[i]);
            self.update_player(inputs.get(i + 1).copied().unwrap_or_default(), dt);
            std::mem::swap(&mut self.player, &mut self.allies[i]);
        }
        self.update_world(dt);
    }

    pub fn disconnect_player(&mut self, slot: u8) {
        for player in std::iter::once(&mut self.player).chain(self.allies.iter_mut()) {
            if player.slot == slot {
                player.connected = false;
                player.health = 0.0;
                player.velocity = Vec2::ZERO;
            }
        }
    }

    fn hurt_player(&mut self, index: usize, damage: f32) {
        if index == 0 {
            self.hurt(damage);
        } else {
            std::mem::swap(&mut self.player, &mut self.allies[index - 1]);
            self.hurt(damage);
            std::mem::swap(&mut self.player, &mut self.allies[index - 1]);
        }
    }

    fn update_revives(&mut self, dt: f32) {
        if self.party_size == 0 {
            return;
        }
        let mut beacons = std::mem::take(&mut self.beacons);
        for beacon in &mut beacons {
            let fallen: Vec<_> = self
                .players()
                .enumerate()
                .filter(|(_, p)| p.connected && p.health <= 0.0)
                .map(|(index, _)| index)
                .collect();
            if fallen.is_empty() {
                beacon.progress = -1.0;
                continue;
            }
            let helpers = self
                .players()
                .filter(|p| {
                    p.connected
                        && p.health > 0.0
                        && p.pos.distance(beacon.pos) <= REVIVE_RADIUS
                        && self.level.clear_shot(p.pos, beacon.pos)
                })
                .count();
            let contested = self.enemies.iter().any(|e| {
                e.health > 0.0 && e.pos.distance(beacon.pos) < REVIVE_RADIUS + e.kind.radius()
            });
            if helpers > 0 && !contested {
                beacon.progress = (beacon.progress + dt * helpers as f32).min(REVIVE_SECONDS);
            } else {
                beacon.progress = (beacon.progress - dt * 2.0).max(0.0);
            }
            if beacon.progress + 0.0001 >= REVIVE_SECONDS {
                if let Some(pos) = self.free_party_spawn(beacon.pos) {
                    let index =
                        fallen[(self.random() * fallen.len() as f32) as usize % fallen.len()];
                    let player = if index == 0 {
                        &mut self.player
                    } else {
                        &mut self.allies[index - 1]
                    };
                    player.health = 100.0;
                    player.pos = pos;
                    player.previous = pos;
                    player.velocity = Vec2::ZERO;
                    player.invulnerable = 3.0;
                    player.dash_time = 0.0;
                    player.reload = 0.0;
                    player.ammo = WEAPONS.map(|w| w.magazine);
                    self.sounds.push(SoundEvent::Pickup);
                    self.field_timer = 0.0;
                    beacon.progress = -1.0;
                }
            }
        }
        beacons.retain(|b| b.progress >= 0.0);
        self.beacons = beacons;
    }

    fn random(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng as u32) as f32 / u32::MAX as f32
    }
    pub fn burst(&mut self, pos: Vec2, color: Color, count: usize, speed: f32) {
        for _ in 0..count {
            if self.particles.len() >= 1800 {
                break;
            }
            let angle = self.random() * std::f32::consts::TAU;
            let velocity = Vec2::from_angle(angle) * speed * (0.2 + self.random() * 0.8);
            let life = 0.15 + self.random() * 0.4;
            let size = 1.5 + self.random() * 2.5;
            self.particles.push(Particle {
                pos,
                velocity,
                life,
                max_life: life,
                color,
                size,
            });
        }
    }
    pub fn update(&mut self, input: Input, dt: f32) {
        if self.player.health <= 0.0 || self.victory {
            return;
        }
        self.begin_frame(dt);
        self.update_player(input, dt);
        self.update_world(dt);
    }
    fn begin_frame(&mut self, dt: f32) {
        self.time += dt;
        self.wave_announcement = (self.wave_announcement - dt).max(0.0);
        self.shake = (self.shake - dt * 18.0).max(0.0);
        self.hit_flash = (self.hit_flash - dt).max(0.0);
        self.combo_timer -= dt;
        if self.combo_timer <= 0.0 {
            self.combo = 0;
        }
    }
    fn update_player(&mut self, input: Input, dt: f32) {
        if self.player.health <= 0.0 || !self.player.connected {
            return;
        }
        self.player.previous = self.player.pos;
        self.player.cooldown = (self.player.cooldown - dt).max(0.0);
        self.player.dash_cooldown = (self.player.dash_cooldown - dt).max(0.0);
        self.player.invulnerable = (self.player.invulnerable - dt).max(0.0);
        self.player.quad = (self.player.quad - dt).max(0.0);
        self.player.immortal = (self.player.immortal - dt).max(0.0);
        self.player.haste = (self.player.haste - dt).max(0.0);
        if let Some(w) = input.weapon.filter(|&w| w < WEAPONS.len()) {
            if w != self.player.weapon {
                self.player.weapon = w;
                self.player.reload = 0.0;
            }
        }
        if input.aim.distance_squared(self.player.pos) > 1.0 {
            self.player.aim = (input.aim - self.player.pos).normalize();
        }
        if self.player.reload > 0.0 {
            self.player.reload -= dt;
            if self.player.reload <= 0.0 {
                self.player.ammo[self.player.weapon] = WEAPONS[self.player.weapon].magazine;
            }
        }
        let movement = input.movement.normalize_or_zero();
        if input.dash && self.player.dash_cooldown <= 0.0 {
            self.player.dash_direction = if movement.length_squared() > 0.0 {
                movement
            } else {
                self.player.aim
            };
            self.player.dash_time = 0.16;
            self.player.dash_cooldown = 1.4;
            self.player.invulnerable = 0.22;
            self.sounds.push(SoundEvent::Dash);
        }
        let speed_bonus = if self.player.haste > 0.0 { 2.0 } else { 1.0 };
        let terrain = self.level.terrain_speed(self.player.pos);
        let target = movement * 245.0 * speed_bonus * terrain;
        self.player.velocity = self.player.velocity.lerp(target, 1.0 - (-24.0 * dt).exp());
        let velocity = if self.player.dash_time > 0.0 {
            self.player.dash_time -= dt;
            if ((self.time * 120.0) as u32).is_multiple_of(3) {
                self.burst(self.player.pos, Color::from_hex(0x62ead5), 2, 35.0);
            }
            self.player.dash_direction * 800.0 * speed_bonus * terrain
        } else {
            self.player.velocity
        };
        let actors: Vec<_> = self
            .enemies
            .iter()
            .map(|e| (e.pos, e.kind.radius()))
            .chain(
                self.allies
                    .iter()
                    .filter(|p| p.connected && p.health > 0.0)
                    .map(|p| (p.pos, PLAYER_RADIUS)),
            )
            .collect();
        self.player.pos =
            self.level
                .push_slide(self.player.pos, velocity * dt, PLAYER_RADIUS, &actors);
        self.player.walk_distance =
            (self.player.walk_distance + self.player.previous.distance(self.player.pos)) % 160.0;
        let weapon = WEAPONS[self.player.weapon];
        if input.reload
            && self.player.ammo[self.player.weapon] < weapon.magazine
            && self.player.reload <= 0.0
        {
            self.player.reload = weapon.reload;
        }
        if input.fire && self.player.reload <= 0.0 && self.player.cooldown <= 0.0 {
            if self.player.ammo[self.player.weapon] == 0 {
                self.player.reload = weapon.reload;
            } else {
                self.shoot();
            }
        }
    }
    fn update_world(&mut self, dt: f32) {
        self.field_timer -= dt;
        if self.field_timer <= 0.0 {
            self.field = self.level.flow_field(self.player.pos);
            self.ally_fields = self
                .allies
                .iter()
                .map(|p| self.level.flow_field(p.pos))
                .collect();
            self.field_timer = 0.16;
        }
        self.update_waves(dt);
        self.update_enemies(dt);
        self.update_bullets(dt);
        self.update_grenades(dt);
        for blast in &mut self.blasts {
            blast.life -= dt;
        }
        self.blasts.retain(|b| b.life > 0.0);
        let mut dead = vec![];
        self.enemies.retain(|e| {
            if e.health <= 0.0 {
                dead.push((e.pos, e.kind));
                false
            } else {
                true
            }
        });
        for (pos, kind) in dead {
            self.kills += 1;
            self.combo += 1;
            self.combo_timer = 3.0;
            self.score += (if kind.is_boss() {
                1500
            } else if kind == Kind::Brute {
                150
            } else {
                75
            }) * (1 + self.combo.min(20) / 5);
            self.burst(pos, kind.color(), 22, 180.0);
            let roll = self.random();
            if roll < 0.24 {
                let kind = if roll < 0.12 {
                    PickupKind::Health
                } else if roll < 0.16 {
                    PickupKind::Quad
                } else if roll < 0.20 {
                    PickupKind::Immortal
                } else {
                    PickupKind::Haste
                };
                self.pickups.push(Pickup {
                    pos,
                    life: 24.0,
                    kind,
                });
            }
        }
        for pickup in &mut self.pickups {
            pickup.life -= dt;
            for player in std::iter::once(&mut self.player).chain(self.allies.iter_mut()) {
                if player.health > 0.0
                    && player.connected
                    && pickup.life > 0.0
                    && pickup.pos.distance(player.pos) < 36.0
                    && (pickup.kind != PickupKind::Health || player.health < 100.0)
                    && self.level.clear_shot(pickup.pos, player.pos)
                {
                    match pickup.kind {
                        PickupKind::Health => player.health = (player.health + 30.0).min(100.0),
                        PickupKind::Quad => player.quad = 12.0,
                        PickupKind::Immortal => player.immortal = 8.0,
                        PickupKind::Haste => player.haste = 10.0,
                    }
                    pickup.life = 0.0;
                    self.sounds.push(SoundEvent::Pickup);
                }
            }
        }
        self.pickups.retain(|p| p.life > 0.0);
        for player in std::iter::once(&mut self.player).chain(self.allies.iter_mut()) {
            if player.health > 0.0
                && player.connected
                && self
                    .level
                    .medical
                    .iter()
                    .any(|p| p.distance(player.pos) < 34.0 && self.level.clear_shot(*p, player.pos))
            {
                player.health = (player.health + 4.0 * dt).min(100.0);
            }
        }
        self.update_revives(dt);
        for p in &mut self.particles {
            p.life -= dt;
            p.pos += p.velocity * dt;
            p.velocity *= (-4.0 * dt).exp();
        }
        self.particles.retain(|p| p.life > 0.0);
    }
    fn shoot(&mut self) {
        let w = WEAPONS[self.player.weapon];
        self.player.ammo[self.player.weapon] -= 1;
        self.player.cooldown = w.interval;
        self.shake = self
            .shake
            .max(if self.player.weapon == 0 { 1.4 } else { 4.0 });
        self.sounds.push(match self.player.weapon {
            1 => SoundEvent::Shotgun,
            2 => SoundEvent::Rail,
            3 | 4 => SoundEvent::Shotgun,
            _ => SoundEvent::Rifle,
        });
        let weapon_scale = if self.party_size > 0 {
            1.0 + self.difficulty.weapon_damage[self.player.weapon] as f32 / 100.0
        } else {
            1.0
        };
        let damage = w.damage * weapon_scale * if self.player.quad > 0.0 { 4.0 } else { 1.0 };
        if self.player.weapon == 4 {
            let target = self.player.pos + self.player.aim * 175.0;
            self.throw_grenade(self.player.pos, target, damage, false);
            return;
        }
        for _ in 0..w.pellets {
            let spread = (self.random() * 2.0 - 1.0) * w.spread;
            let direction = Vec2::from_angle(self.player.aim.to_angle() + spread);
            // Start at the actor center so a muzzle pressed against a wall cannot shoot through it.
            let pos = self.player.pos;
            self.bullets.push(Bullet {
                pos,
                previous: pos,
                origin: pos,
                velocity: direction * w.speed,
                damage,
                life: if self.player.weapon == 3 { 2.2 } else { 1.2 },
                hostile: false,
                rail: self.player.weapon == 2,
                blast_radius: if self.player.weapon == 3 { 135.0 } else { 0.0 },
            });
        }
        self.burst(
            self.player.pos + self.player.aim * 22.0,
            Color::from_hex(0xffe9aa),
            5,
            85.0,
        );
    }
    pub fn between_waves(&self) -> bool {
        self.remaining == 0 && self.enemies.is_empty() && self.spawns.is_empty()
    }
    pub fn boss_warning(&self) -> Option<(u32, Kind)> {
        if self.victory || !self.team_alive() {
            return None;
        }
        let wave = if self.between_waves() {
            self.wave + 1
        } else if self.wave_announcement > 0.0 {
            self.wave
        } else {
            return None;
        };
        waves::boss_for_wave(wave).map(|boss| (wave, boss))
    }
    fn update_waves(&mut self, dt: f32) {
        if self.remaining == 0 && self.enemies.is_empty() && self.spawns.is_empty() {
            if self.wave >= waves::CAMPAIGN_WAVES && !self.endless {
                self.victory = true;
                return;
            }
            self.intermission -= dt;
            if self.intermission <= 0.0 {
                self.wave += 1;
                self.remaining =
                    waves::party_roster(self.wave, self.party_size, self.difficulty.enemy_count)
                        .len() as u32;
                self.spawn_timer = 0.0;
                self.intermission = 4.0;
                self.wave_announcement = 3.5;
                self.sounds
                    .push(if waves::boss_for_wave(self.wave).is_some() {
                        SoundEvent::Boss
                    } else {
                        SoundEvent::Wave
                    });
                for player in std::iter::once(&mut self.player).chain(self.allies.iter_mut()) {
                    if player.health > 0.0 && player.connected {
                        player.health = (player.health + 10.0).min(100.0);
                    }
                }
                let kind = match self.wave % 3 {
                    1 => PickupKind::Quad,
                    2 => PickupKind::Haste,
                    _ => PickupKind::Immortal,
                };
                let supply = (0..self.level.tiles.len())
                    .filter(|&i| self.spawn_reachable(i))
                    .map(|i| self.level.center(i))
                    .filter(|&p| self.level.circle_free(p, 18.0))
                    .min_by(|a, b| {
                        (a.distance(self.player.pos) - 150.0)
                            .abs()
                            .total_cmp(&(b.distance(self.player.pos) - 150.0).abs())
                    });
                if let Some(pos) = supply {
                    self.pickups.push(Pickup {
                        pos,
                        life: 30.0,
                        kind,
                    });
                }
            }
        }
        self.spawn_timer -= dt;
        if self.remaining > 0
            && self.spawn_timer <= 0.0
            && self.enemies.len() + self.spawns.len()
                < (48.0 * self.party_count_scale()).ceil() as usize
        {
            let mut candidates: Vec<Vec2> = self
                .level
                .portals
                .iter()
                .copied()
                .filter(|p| {
                    self.spawn_reachable(self.level.index(*p))
                        && self.level.circle_free(*p, 19.0)
                        && self
                            .players()
                            .filter(|p| p.health > 0.0 && p.connected)
                            .all(|a| p.distance(a.pos) > TILE * 4.0)
                        && self.enemies.iter().all(|e| e.pos.distance(*p) > 38.0)
                        && self.spawns.iter().all(|s| s.pos.distance(*p) > 38.0)
                })
                .collect();
            if candidates.is_empty() {
                let farthest = (0..self.level.tiles.len())
                    .filter(|&i| self.spawn_reachable(i))
                    .map(|i| self.distance_to_team(self.level.center(i)))
                    .fold(0.0_f32, f32::max);
                let safe_distance = (farthest * 0.7).min(TILE * 4.0);
                candidates = (0..self.level.tiles.len())
                    .filter(|&i| self.spawn_reachable(i))
                    .map(|i| self.level.center(i))
                    .filter(|p| {
                        self.players()
                            .filter(|p| p.health > 0.0 && p.connected)
                            .all(|a| p.distance(a.pos) > safe_distance)
                            && self.level.circle_free(*p, 19.0)
                            && self.enemies.iter().all(|e| e.pos.distance(*p) > 42.0)
                            && self.spawns.iter().all(|s| s.pos.distance(*p) > 42.0)
                    })
                    .collect();
            }
            if !candidates.is_empty() {
                let idx = (self.random() * candidates.len() as f32) as usize % candidates.len();
                let roster =
                    waves::party_roster(self.wave, self.party_size, self.difficulty.enemy_count);
                let kind = roster[roster.len() - self.remaining as usize];
                self.spawns.push(Spawn {
                    pos: candidates[idx],
                    timer: if kind.is_boss() { 2.4 } else { 0.85 },
                    kind,
                });
                self.remaining -= 1;
            }
            self.spawn_timer = waves::spawn_interval(self.wave) / self.party_count_scale();
        }
        for s in &mut self.spawns {
            s.timer -= dt;
        }
        let mut ready = vec![];
        self.spawns.retain(|s| {
            if s.timer <= 0.0 {
                ready.push((s.pos, s.kind));
                false
            } else {
                true
            }
        });
        for (pos, kind) in ready {
            let health = kind.base_health()
                * waves::health_scale(self.wave)
                * Difficulty::scale(self.party_size, self.difficulty.enemy_health);
            let phase = self.random() * std::f32::consts::TAU;
            self.enemies.push(Enemy {
                pos,
                previous: pos,
                kind,
                health,
                max_health: health,
                attack: 0.6,
                tell: 0.0,
                aim: Vec2::X,
                phase,
            });
            self.burst(pos, kind.color(), 14, 100.0);
        }
    }
    fn update_enemies(&mut self, dt: f32) {
        let positions: Vec<Vec2> = self.enemies.iter().map(|e| e.pos).collect();
        let targets: Vec<_> = self
            .players()
            .map(|p| (p.pos, p.velocity, p.health > 0.0 && p.connected))
            .collect();
        let mut damage = vec![0.0; targets.len()];
        let damage_scale = Difficulty::scale(self.party_size, self.difficulty.enemy_damage);
        let mut mortars = vec![];
        for (i, e) in self.enemies.iter_mut().enumerate() {
            e.previous = e.pos;
            e.attack -= dt;
            let Some((target_index, &(target_pos, target_velocity, _))) = targets
                .iter()
                .enumerate()
                .filter(|(index, (_, _, alive))| {
                    *alive
                        && self.level.route_reachable(
                            e.pos,
                            targets[*index].0,
                            if *index == 0 {
                                &self.field
                            } else {
                                &self.ally_fields[*index - 1]
                            },
                            e.kind.radius(),
                        )
                })
                .min_by(|(_, a), (_, b)| {
                    a.0.distance_squared(e.pos)
                        .total_cmp(&b.0.distance_squared(e.pos))
                })
            else {
                continue;
            };
            let target_field = if target_index == 0 {
                &self.field
            } else {
                &self.ally_fields[target_index - 1]
            };
            let to_player = target_pos - e.pos;
            let distance = to_player.length();
            let visible = distance < 700.0 && self.level.clear_shot(e.pos, target_pos);
            let waypoint = self
                .level
                .waypoint(e.pos, target_pos, target_field, e.kind.radius());
            let path_direction = (waypoint - e.pos).normalize_or_zero();
            let mut direction = path_direction;
            let mut speed = match e.kind {
                Kind::Runner => 145.0,
                Kind::Gunner => 110.0,
                Kind::Brute => 85.0,
                Kind::Hunter => 155.0,
                Kind::Bomber => 95.0,
                Kind::Siege => 75.0,
                Kind::Stalker => 190.0,
                Kind::Lancer => 100.0,
                Kind::Sentinel => 80.0,
                Kind::ThornTyrant => 105.0,
                Kind::BroodMatriarch => 85.0,
                Kind::Overlord => 95.0,
            };
            speed *= waves::speed_scale(self.wave);
            if e.kind == Kind::Stalker && visible && distance > 70.0 {
                let forward = to_player.normalize_or_zero();
                direction = forward
                    + vec2(-forward.y, forward.x) * (self.time * 7.0 + e.phase).sin() * 0.65;
            }
            let range = match e.kind {
                Kind::Hunter => 530.0,
                Kind::Bomber => 470.0,
                Kind::Siege | Kind::ThornTyrant => 500.0,
                Kind::Lancer => 610.0,
                Kind::Sentinel => 440.0,
                Kind::BroodMatriarch => 600.0,
                Kind::Overlord => 650.0,
                _ => 360.0,
            };
            if !matches!(e.kind, Kind::Runner | Kind::Brute | Kind::Stalker)
                && (visible || matches!(e.kind, Kind::Bomber | Kind::BroodMatriarch))
                && distance < range
            {
                let forward = to_player.normalize_or_zero();
                direction = vec2(-forward.y, forward.x) * (self.time * 0.7 + e.phase).sin() * 0.75;
                if distance < range * 0.5 {
                    direction -= forward;
                }
                if e.attack <= 0.0 && e.tell <= 0.0 {
                    e.tell = match e.kind {
                        kind if kind.is_boss() => 1.1,
                        Kind::Siege | Kind::Bomber | Kind::Lancer => 0.9,
                        _ => 0.6,
                    };
                    e.aim = (target_pos
                        + target_velocity * if e.kind == Kind::Hunter { 0.26 } else { 0.2 }
                        - e.pos)
                        .normalize_or_zero();
                }
            }
            if e.tell > 0.0 {
                e.tell -= dt;
                direction *= 0.25;
                if e.tell <= 0.0 {
                    if matches!(e.kind, Kind::Bomber | Kind::BroodMatriarch | Kind::Overlord) {
                        let target = target_pos + target_velocity * 0.4;
                        let side = vec2(-e.aim.y, e.aim.x);
                        if e.kind == Kind::BroodMatriarch {
                            for offset in [-95.0, 0.0, 95.0] {
                                mortars.push((e.pos, target + side * offset));
                            }
                        } else {
                            mortars.push((e.pos, target));
                        }
                    }
                    if !matches!(e.kind, Kind::Bomber | Kind::BroodMatriarch) {
                        let count = match e.kind {
                            Kind::Gunner | Kind::Hunter | Kind::Overlord => 3,
                            Kind::Sentinel => 5,
                            Kind::ThornTyrant => 12,
                            _ => 1,
                        };
                        for k in 0..count {
                            let spread = (k as f32 - (count - 1) as f32 * 0.5)
                                * match e.kind {
                                    Kind::Hunter => 0.055,
                                    Kind::ThornTyrant => std::f32::consts::TAU / count as f32,
                                    _ => 0.22,
                                };
                            self.bullets.push(Bullet {
                                pos: e.pos,
                                previous: e.pos,
                                origin: e.pos,
                                velocity: Vec2::from_angle(e.aim.to_angle() + spread)
                                    * match e.kind {
                                        Kind::Hunter => 650.0,
                                        Kind::Lancer => 850.0,
                                        Kind::Siege | Kind::Overlord => 300.0,
                                        Kind::ThornTyrant => 270.0,
                                        _ => 320.0,
                                    },
                                damage: match e.kind {
                                    Kind::Siege => 38.0,
                                    Kind::Overlord => 32.0,
                                    Kind::Lancer => 24.0,
                                    Kind::ThornTyrant => 18.0,
                                    _ => 12.0,
                                } * damage_scale,
                                life: 3.0,
                                hostile: true,
                                rail: matches!(e.kind, Kind::Hunter | Kind::Lancer),
                                blast_radius: if matches!(e.kind, Kind::Siege | Kind::Overlord) {
                                    110.0
                                } else {
                                    0.0
                                },
                            });
                        }
                    }
                    e.attack = match e.kind {
                        Kind::Bomber => 3.3,
                        Kind::Siege => 2.5,
                        Kind::Hunter => 2.0,
                        Kind::Lancer => 2.4,
                        Kind::Sentinel => 2.3,
                        Kind::ThornTyrant => 2.1,
                        Kind::BroodMatriarch => 3.1,
                        Kind::Overlord => 2.8,
                        _ => 1.7,
                    };
                    if e.kind.is_boss() && e.health < e.max_health * 0.5 {
                        e.attack *= 0.75;
                    }
                }
            }
            e.aim = if e.tell > 0.0 {
                e.aim
            } else {
                to_player.normalize_or_zero()
            };
            let mut separation = Vec2::ZERO;
            for (j, &p) in positions.iter().enumerate() {
                if i == j {
                    continue;
                }
                let delta = e.pos - p;
                let d = delta.length();
                let reach = e.kind.radius() + 22.0;
                if d < reach && d > 0.01 {
                    separation += delta / d * (1.0 - d / reach);
                } else if d <= 0.01 {
                    separation += Vec2::from_angle(e.phase);
                }
            }
            // Path direction retains priority; separation never fully cancels forward progress.
            let mut steering =
                (direction + separation.clamp_length_max(0.65)).clamp_length_max(1.0);
            speed *= self.level.terrain_speed(e.pos);
            // Anticipate obstacles before separation, weaving, or strafing pushes
            // the body into them. Follow the clear route at the intended pace,
            // including the slower movement during an attack windup.
            let lookahead = (speed * 0.25).min(e.pos.distance(waypoint));
            if !self
                .level
                .clear_line(e.pos, e.pos + steering * lookahead, e.kind.radius())
            {
                steering = path_direction * direction.length().min(1.0);
            }
            let movement = steering * speed * dt;
            let next = self.level.slide(e.pos, movement, e.kind.radius());
            e.pos = if next.distance_squared(e.pos) < movement.length_squared() * 0.5
                && direction.length_squared() > 0.1
            {
                self.level.slide(
                    e.pos,
                    path_direction * direction.length().min(1.0) * speed * dt,
                    e.kind.radius(),
                )
            } else {
                next
            };
            if distance < e.kind.radius() + PLAYER_RADIUS + 5.0 && e.attack <= 0.0 {
                damage[target_index] += if e.kind.is_boss() {
                    28.0
                } else if matches!(e.kind, Kind::Brute | Kind::Sentinel) {
                    22.0
                } else if e.kind == Kind::Stalker {
                    12.0
                } else {
                    9.0
                };
                e.attack = 0.8;
            }
        }
        for (origin, target) in mortars {
            self.throw_grenade(origin, target, 45.0 * damage_scale, true);
        }
        for (index, amount) in damage.into_iter().enumerate() {
            if amount > 0.0 {
                self.hurt_player(index, amount * damage_scale);
            }
        }
    }
    fn hurt(&mut self, damage: f32) {
        if self.player.health <= 0.0
            || !self.player.connected
            || self.player.invulnerable > 0.0
            || self.player.immortal > 0.0
        {
            return;
        }
        self.player.health = (self.player.health - damage).max(0.0);
        if self.player.health == 0.0 && self.party_size > 0 {
            self.beacons.push(ReviveBeacon {
                pos: self.player.pos,
                progress: 0.0,
            });
        }
        self.player.invulnerable = 0.45;
        self.hit_flash = 0.2;
        self.shake = 6.0;
        self.sounds.push(SoundEvent::Hurt);
        self.burst(self.player.pos, Color::from_hex(0xff5c77), 12, 130.0);
    }
    fn update_bullets(&mut self, dt: f32) {
        let mut impacts = vec![];
        let targets: Vec<_> = self
            .players()
            .map(|p| (p.pos, p.health > 0.0 && p.connected))
            .collect();
        let mut hurt = vec![0.0; targets.len()];
        let mut explosions = vec![];
        for b in &mut self.bullets {
            b.previous = b.pos;
            b.life -= dt;
            if b.life <= 0.0 {
                if b.blast_radius > 0.0 {
                    explosions.push((b.pos, b.damage, b.blast_radius, b.hostile));
                }
                continue;
            }
            let delta = b.velocity * dt;
            let steps = (delta.length() / 3.0).ceil().max(1.0) as usize;
            for _ in 0..steps {
                let safe = b.pos;
                b.pos += delta / steps as f32;
                if !self.level.projectile_free(b.pos, 2.0) {
                    b.life = 0.0;
                    if b.blast_radius > 0.0 {
                        explosions.push((safe, b.damage, b.blast_radius, b.hostile));
                    }
                    impacts.push((b.pos, Color::from_hex(0x91a9a8), false));
                    break;
                }
                if b.hostile {
                    if let Some(index) = targets.iter().position(|(pos, alive)| {
                        *alive && b.pos.distance(*pos) < PLAYER_RADIUS + 3.0
                    }) {
                        if b.blast_radius > 0.0 {
                            explosions.push((b.pos, b.damage, b.blast_radius, b.hostile));
                        } else {
                            hurt[index] += b.damage;
                        }
                        b.life = 0.0;
                        break;
                    }
                } else if let Some(e) = self
                    .enemies
                    .iter_mut()
                    .find(|e| e.health > 0.0 && e.pos.distance(b.pos) < e.kind.radius() + 2.0)
                {
                    if b.blast_radius > 0.0 {
                        explosions.push((b.pos, b.damage, b.blast_radius, b.hostile));
                    } else {
                        e.health -= b.damage;
                    }
                    e.pos = self.level.slide(
                        e.pos,
                        b.velocity.normalize_or_zero() * if b.rail { 12.0 } else { 4.0 },
                        e.kind.radius(),
                    );
                    b.life = 0.0;
                    impacts.push((b.pos, e.kind.color(), true));
                    break;
                }
            }
        }
        for (pos, damage, radius, hostile) in explosions {
            self.explode(pos, damage, radius, hostile);
        }
        self.bullets.retain(|b| b.life > 0.0);
        for (p, c, hit) in impacts {
            self.burst(p, c, if hit { 7 } else { 3 }, 100.0);
            if hit {
                self.sounds.push(SoundEvent::Hit);
            }
        }
        for (index, amount) in hurt.into_iter().enumerate() {
            if amount > 0.0 {
                self.hurt_player(index, amount);
            }
        }
    }

    fn throw_grenade(&mut self, origin: Vec2, target: Vec2, damage: f32, hostile: bool) {
        let mut landing = target;
        // A lob can clear cover, but must land in a free spot, never inside a tree or crate.
        if !self.level.projectile_free(landing, 5.0) {
            for i in 1..=100 {
                landing = target.lerp(origin, i as f32 / 100.0);
                if self.level.projectile_free(landing, 5.0) {
                    break;
                }
            }
        }
        self.grenades.push(Grenade {
            pos: origin,
            origin,
            target: landing,
            age: 0.0,
            fuse: 2.0,
            damage,
            hostile,
        });
    }
    fn update_grenades(&mut self, dt: f32) {
        let mut blasts = vec![];
        for g in &mut self.grenades {
            g.age += dt;
            g.pos = g.origin.lerp(g.target, (g.age / 0.6).min(1.0));
            if g.age >= g.fuse {
                blasts.push((g.pos, g.damage, g.hostile));
            }
        }
        self.grenades.retain(|g| g.age < g.fuse);
        for (p, d, hostile) in blasts {
            self.explode(p, d, 145.0, hostile);
        }
    }
    fn explode(&mut self, pos: Vec2, damage: f32, radius: f32, hostile: bool) {
        self.blasts.push(Blast {
            pos,
            radius,
            life: 0.45,
            hostile,
        });
        self.burst(pos, Color::from_hex(0xffb449), 48, 350.0);
        self.shake = self.shake.max(8.0);
        self.sounds.push(SoundEvent::Explosion);
        if !hostile {
            for e in &mut self.enemies {
                let distance = e.pos.distance(pos);
                if distance < radius + e.kind.radius() && self.level.clear_shot(pos, e.pos) {
                    e.health -= damage * (1.0 - 0.65 * (distance / radius).min(1.0));
                }
            }
        }
        // Party explosives do not damage allies (or their owner). Solo keeps self damage.
        if hostile || self.party_size == 0 {
            let targets: Vec<_> = self.players().map(|p| p.pos).collect();
            for (index, target) in targets.into_iter().enumerate() {
                let distance = target.distance(pos);
                if distance < radius + PLAYER_RADIUS && self.level.clear_shot(pos, target) {
                    self.hurt_player(
                        index,
                        damage
                            * (1.0 - 0.65 * (distance / radius).min(1.0))
                            * if hostile { 1.0 } else { 0.25 },
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn game() -> Game {
        Game::new(Level::parse("test", "############\n#P........E#\n#..........#\n#..........#\n#..........#\n############").unwrap(),false)
    }
    #[test]
    fn diagonal_speed_matches_cardinal() {
        let mut a = game();
        let mut b = game();
        for _ in 0..60 {
            a.update(
                Input {
                    movement: Vec2::X,
                    ..Default::default()
                },
                STEP,
            );
            b.update(
                Input {
                    movement: vec2(1.0, 1.0),
                    ..Default::default()
                },
                STEP,
            );
        }
        assert!(
            (a.player.pos.distance(a.level.spawn) - b.player.pos.distance(b.level.spawn)).abs()
                < 0.1
        );
    }
    #[test]
    fn walking_phase_uses_actual_distance_including_wall_collisions() {
        let mut g = game();
        let mut traveled = 0.0;
        for _ in 0..100 {
            let previous = g.player.pos;
            g.update(
                Input {
                    movement: -Vec2::X,
                    ..Default::default()
                },
                STEP,
            );
            traveled += previous.distance(g.player.pos);
            assert!((g.player.walk_distance - traveled % 160.0).abs() < 0.001);
        }
        assert!(traveled > 0.0);
        assert_eq!(g.player.previous, g.player.pos);
        assert!(g.player.velocity.length() > 200.0);
    }
    #[test]
    fn shots_cannot_cross_walls() {
        let mut g = game();
        g.player.pos = vec2(70.0, 70.0);
        for _ in 0..20 {
            g.update(
                Input {
                    aim: vec2(-100.0, 70.0),
                    fire: true,
                    ..Default::default()
                },
                STEP,
            );
        }
        assert!(g.bullets.iter().all(|b| b.pos.x >= 48.0));
    }
    #[test]
    fn reload_and_weapon_switch_are_consistent() {
        let mut g = game();
        g.player.ammo[0] = 0;
        g.update(
            Input {
                fire: true,
                ..Default::default()
            },
            STEP,
        );
        assert!(g.player.reload > 0.0);
        for _ in 0..150 {
            g.update(Input::default(), STEP);
        }
        assert_eq!(g.player.ammo[0], 30);
        g.update(
            Input {
                weapon: Some(2),
                ..Default::default()
            },
            STEP,
        );
        assert_eq!(g.player.weapon, 2);
    }
    #[test]
    fn dash_cannot_tunnel_through_boundary() {
        let mut g = game();
        for _ in 0..90 {
            g.update(
                Input {
                    movement: -Vec2::X,
                    dash: true,
                    ..Default::default()
                },
                STEP,
            );
        }
        assert!(g.player.pos.x >= 48.0 + PLAYER_RADIUS - 0.01);
    }
    #[test]
    fn sustained_simulation_stays_finite_and_inside_map() {
        let mut g = game();
        g.player.health = 100000.0;
        for frame in 0..7200 {
            g.player.health = 100000.0;
            let a = frame as f32 * 0.013;
            g.update(
                Input {
                    movement: Vec2::from_angle(a),
                    aim: g.player.pos + Vec2::from_angle(a) * 200.0,
                    fire: true,
                    dash: frame % 180 == 0,
                    ..Default::default()
                },
                STEP,
            );
            assert!(g.level.circle_free(g.player.pos, PLAYER_RADIUS));
            assert!(g
                .enemies
                .iter()
                .all(|e| e.pos.is_finite() && g.level.circle_free(e.pos, e.kind.radius())));
        }
        assert!(g.wave > 0);
        assert!(g.kills > 0);
    }

    #[test]
    fn small_arena_can_spawn_even_when_player_covers_center() {
        let level = Level::parse("small", "#####\n#E..#\n#.P.#\n#...#\n#####").unwrap();
        let mut g = Game::new(level, false);
        for _ in 0..400 {
            g.update(Input::default(), STEP);
        }
        assert!(!g.enemies.is_empty());
    }

    #[test]
    fn fast_projectile_hits_actor_between_endpoints() {
        let mut g = game();
        let p = vec2(150.0, 100.0);
        g.enemies.push(Enemy {
            pos: p,
            previous: p,
            kind: Kind::Runner,
            health: 48.0,
            max_health: 48.0,
            attack: 1.0,
            tell: 0.0,
            aim: Vec2::X,
            phase: 0.0,
        });
        g.bullets.push(Bullet {
            pos: vec2(80.0, 100.0),
            previous: vec2(80.0, 100.0),
            origin: vec2(80.0, 100.0),
            velocity: vec2(1900.0, 0.0),
            damage: 110.0,
            life: 1.0,
            hostile: false,
            rail: true,
            blast_radius: 0.0,
        });
        g.update_bullets(0.1);
        assert!(g.enemies[0].health <= 0.0);
        assert!(g.bullets.is_empty());
    }

    #[test]
    fn twenty_wave_objective_and_endless_mode_diverge() {
        let mut finite = game();
        finite.wave = waves::CAMPAIGN_WAVES;
        finite.update(Input::default(), STEP);
        assert!(finite.victory);
        let mut endless = game();
        endless.wave = waves::CAMPAIGN_WAVES;
        endless.endless = true;
        endless.intermission = 0.0;
        endless.update(Input::default(), STEP);
        assert!(!endless.victory);
        assert_eq!(endless.wave, waves::CAMPAIGN_WAVES + 1);
    }

    #[test]
    fn full_campaign_spawns_each_boss_once_and_only_finishes_after_twenty() {
        let mut g = game();
        let mut bosses = vec![];
        for wave in 1..=waves::CAMPAIGN_WAVES {
            g.intermission = 0.0;
            g.update_waves(STEP);
            assert_eq!(g.wave, wave);
            assert!(!g.victory);
            let mut spawned = 0;
            for _ in 0..12000 {
                g.update_waves(STEP);
                for enemy in g.enemies.drain(..) {
                    assert!(g.level.circle_free(enemy.pos, enemy.kind.radius()));
                    spawned += 1;
                    if enemy.kind.is_boss() {
                        bosses.push((wave, enemy.kind));
                    }
                }
                if g.between_waves() {
                    break;
                }
            }
            assert!(g.between_waves(), "wave {wave} stalled");
            assert_eq!(spawned, waves::roster(wave).len());
        }
        g.update_waves(STEP);
        assert!(g.victory);
        assert_eq!(
            bosses,
            vec![
                (7, Kind::ThornTyrant),
                (15, Kind::BroodMatriarch),
                (20, Kind::Overlord)
            ]
        );
    }

    #[test]
    fn spawned_bosses_have_triple_health_with_wave_scaling() {
        for (wave, kind, previous_health) in [
            (7, Kind::ThornTyrant, 660.0),
            (15, Kind::BroodMatriarch, 1050.0),
            (20, Kind::Overlord, 1600.0),
        ] {
            let mut g = game();
            g.wave = wave;
            for kind in [kind, Kind::Runner] {
                g.spawns.push(Spawn {
                    pos: vec2(350.0, 120.0),
                    timer: 0.0,
                    kind,
                });
            }
            g.update_waves(STEP);
            let expected = previous_health * 3.0 * waves::health_scale(wave);
            assert_eq!(g.enemies[0].health, expected);
            assert_eq!(g.enemies[0].max_health, expected);
            assert_eq!(g.enemies[1].health, 48.0 * waves::health_scale(wave));
        }
    }

    #[test]
    fn every_enemy_moves_ten_percent_faster_on_open_ground() {
        let border = "#".repeat(32);
        let floor = format!("#{}#", ".".repeat(30));
        let markers = format!("#P{}E#", ".".repeat(28));
        let level = Level::parse(
            "open",
            &[border.clone(), markers, floor.clone(), floor, border].join("\n"),
        )
        .unwrap();
        for (kind, previous_speed) in [
            (Kind::Runner, 145.0),
            (Kind::Gunner, 110.0),
            (Kind::Brute, 85.0),
            (Kind::Hunter, 155.0),
            (Kind::Bomber, 95.0),
            (Kind::Siege, 75.0),
            (Kind::Stalker, 190.0),
            (Kind::Lancer, 100.0),
            (Kind::Sentinel, 80.0),
            (Kind::ThornTyrant, 105.0),
            (Kind::BroodMatriarch, 85.0),
            (Kind::Overlord, 95.0),
        ] {
            let mut g = Game::new(level.clone(), false);
            g.wave = 1;
            let start = g.level.portals[0];
            g.enemies.push(monster(start, kind));
            for _ in 0..60 {
                g.update_enemies(STEP);
            }
            let travelled = start.distance(g.enemies[0].pos);
            assert!(
                (travelled - previous_speed * 1.1 * 0.5).abs() < 0.01,
                "{kind:?}: {travelled}"
            );
        }
    }

    #[test]
    fn stalker_takes_the_clear_route_instead_of_weaving_into_a_tree() {
        let level = Level::parse(
            "tree",
            "############\n#P........E#\n#....T.....#\n#..........#\n#..........#\n############",
        )
        .unwrap();
        let mut g = Game::new(level, false);
        g.player.pos = vec2(72.0, 84.0);
        g.field = g.level.flow_field(g.player.pos);
        let start = vec2(312.0, 84.0);
        assert!(g.level.clear_shot(start, g.player.pos));
        assert!(!g
            .level
            .clear_line(start, g.player.pos, Kind::Stalker.radius()));
        let mut enemy = monster(start, Kind::Stalker);
        enemy.phase = -std::f32::consts::FRAC_PI_2;
        g.enemies.push(enemy);
        for frame in 0..24 {
            g.time = frame as f32 * STEP;
            g.update_enemies(STEP);
            assert!(g
                .level
                .circle_free(g.enemies[0].pos, Kind::Stalker.radius()));
        }
        assert!(
            g.enemies[0].pos.x < 285.0,
            "stalker stalled at {:?}",
            g.enemies[0].pos
        );
        assert!(
            g.enemies[0].pos.y < 84.0,
            "stalker should go above the tree"
        );
    }

    #[test]
    fn crowded_enemies_detour_around_trees_without_slow_wall_sliding() {
        let level = Level::parse(
            "trees",
            "############\n#P........E#\n#....T.....#\n#....T.....#\n#..........#\n############",
        )
        .unwrap();
        for (kind, speed) in [(Kind::Runner, 145.0), (Kind::Brute, 85.0)] {
            let mut g = Game::new(level.clone(), false);
            let start = vec2(288.0 + kind.radius(), 168.0);
            g.enemies.push(monster(start, kind));
            g.enemies.push(monster(start + vec2(15.0, 15.0), kind));
            let mut slow_steps = 0;
            for frame in 0..240 {
                g.time = frame as f32 * STEP;
                let before = g.enemies[0].pos;
                g.update_enemies(STEP);
                let travelled = before.distance(g.enemies[0].pos);
                if travelled < speed * 1.1 * STEP * 0.7 {
                    slow_steps += 1;
                }
                for enemy in &g.enemies {
                    assert!(g.level.circle_free(enemy.pos, kind.radius()));
                }
            }
            assert!(slow_steps < 12, "{kind:?}: {slow_steps} slow steps");
            assert!(
                g.enemies[0].pos.x < 240.0 - kind.radius(),
                "{kind:?}: failed to round trees at {:?}",
                g.enemies[0].pos
            );
        }
    }

    #[test]
    fn enemies_escape_tiles_blocked_by_moved_crates() {
        let level = Level::parse(
            "moved cover",
            "############\n#.........E#\n#........P.#\n#..........#\n#..........#\n############",
        )
        .unwrap();
        for kind in [Kind::Runner, Kind::Brute] {
            for party in [false, true] {
                let mut g = Game::new(level.clone(), false);
                if party {
                    g = Game::new_party(level.clone(), false, 2).unwrap();
                    g.player.health = 0.0;
                }
                g.level.crates.push(vec2(300.0, 120.0));
                g.field = g.level.flow_field(g.player.pos);
                g.ally_fields = g.allies.iter().map(|p| g.level.flow_field(p.pos)).collect();
                let start = vec2(250.0, 120.0);
                assert!(g.level.circle_free(start, kind.radius()));
                assert_eq!(g.field[g.level.index(start)], u32::MAX);
                g.enemies.push(monster(start, kind));
                for frame in 0..720 {
                    g.time = frame as f32 * STEP;
                    g.update_enemies(STEP);
                    assert!(g.level.circle_free(g.enemies[0].pos, kind.radius()));
                }
                assert!(
                    g.enemies[0].pos.x > 350.0,
                    "{kind:?}, party {party}: stuck at {:?}",
                    g.enemies[0].pos
                );
            }
        }
    }

    #[test]
    fn clear_ground_preserves_slow_strafing_and_attack_windups() {
        for winding_up in [false, true] {
            let mut g = game();
            let start = vec2(350.0, 120.0);
            let mut enemy = monster(start, Kind::Gunner);
            enemy.attack = 10.0;
            enemy.phase = (0.4_f32 / 0.75).asin();
            enemy.tell = if winding_up { 1.0 } else { 0.0 };
            g.enemies.push(enemy);
            g.update_enemies(STEP);
            let movement = g.enemies[0].pos - start;
            let forward = (g.player.pos - start).normalize_or_zero();
            let expected = 110.0 * 1.1 * 0.4 * STEP * if winding_up { 0.25 } else { 1.0 };
            assert!(movement.dot(forward).abs() < 0.0001);
            assert!((movement.length() - expected).abs() < 0.0001);
        }
    }

    #[test]
    fn boss_warnings_cover_countdown_and_arrival() {
        for wave in [7, 15, 20] {
            let mut g = game();
            g.wave = wave - 1;
            assert_eq!(
                g.boss_warning(),
                waves::boss_for_wave(wave).map(|boss| (wave, boss))
            );
            g.intermission = 0.0;
            g.update_waves(STEP);
            assert_eq!(g.boss_warning().unwrap().0, wave);
            assert!(g.spawns.iter().any(|s| s.kind.is_boss() && s.timer > 2.0));
            assert!(g
                .sounds
                .iter()
                .any(|event| matches!(event, SoundEvent::Boss)));
            g.wave_announcement = 0.0;
            assert!(g.boss_warning().is_none());
        }
        let mut ordinary = game();
        ordinary.wave = 8;
        assert!(ordinary.boss_warning().is_none());
    }

    #[test]
    fn final_boss_and_pending_spawns_block_victory() {
        let mut g = game();
        g.wave = waves::CAMPAIGN_WAVES;
        g.spawns.push(Spawn {
            pos: vec2(350.0, 120.0),
            timer: 0.1,
            kind: Kind::Overlord,
        });
        g.update_waves(0.2);
        assert!(!g.victory);
        assert!(g.enemies.iter().any(|e| e.kind == Kind::Overlord));
        g.update_waves(STEP);
        assert!(!g.victory);
        g.enemies[0].health = 0.0;
        g.update(Input::default(), STEP);
        g.update(Input::default(), STEP);
        assert!(g.victory);
    }

    #[test]
    fn buffed_weapons_hit_their_new_damage_breakpoints() {
        for (weapon, kind, wave) in [(2, Kind::Brute, 6), (1, Kind::Bomber, 3)] {
            let mut g = game();
            g.player.pos = vec2(100.0, 120.0);
            g.player.aim = Vec2::X;
            g.player.weapon = weapon;
            let mut target = monster(vec2(140.0, 120.0), kind);
            target.health = kind.base_health() * waves::health_scale(wave);
            target.max_health = target.health;
            g.enemies.push(target);
            g.shoot();
            g.update_bullets(0.08);
            assert!(
                g.enemies[0].health <= 0.0,
                "weapon {weapon} failed its damage breakpoint"
            );
        }
    }

    #[test]
    fn railgun_starts_full_and_refills_its_larger_magazine() {
        let mut g = game();
        assert_eq!(g.player.ammo[2], 8);
        g.player.weapon = 2;
        g.player.ammo[2] = 0;
        g.update(
            Input {
                fire: true,
                ..Default::default()
            },
            STEP,
        );
        for _ in 0..130 {
            g.update(Input::default(), STEP);
        }
        assert_eq!(g.player.ammo[2], 8);
        assert!(g.player.reload <= 0.0);
    }

    fn monster(pos: Vec2, kind: Kind) -> Enemy {
        Enemy {
            pos,
            previous: pos,
            kind,
            health: 300.0,
            max_health: 300.0,
            attack: 0.0,
            tell: 0.0,
            aim: Vec2::X,
            phase: 0.0,
        }
    }
    #[test]
    fn grenade_fuse_is_two_seconds_and_hits_multiple_targets() {
        let mut g = game();
        let landing = vec2(240.0, 140.0);
        g.enemies.push(monster(landing, Kind::Brute));
        g.enemies
            .push(monster(landing + vec2(40.0, 0.0), Kind::Runner));
        g.throw_grenade(g.player.pos, landing, 175.0, false);
        g.update_grenades(1.9);
        assert_eq!(g.enemies[0].health, 300.0);
        assert_eq!(g.grenades.len(), 1);
        g.update_grenades(0.11);
        assert!(g.grenades.is_empty());
        assert_eq!(g.blasts.len(), 1);
        assert!(g.enemies.iter().all(|e| e.health < 300.0));
    }
    #[test]
    fn wooden_cover_stops_splash_and_cannot_be_destroyed() {
        let mut g = game();
        g.level.crates.push(vec2(264.0, 120.0));
        g.enemies.push(monster(vec2(312.0, 120.0), Kind::Brute));
        g.enemies.push(monster(vec2(200.0, 165.0), Kind::Runner));
        g.explode(vec2(200.0, 120.0), 1000.0, 160.0, false);
        assert_eq!(g.enemies[0].health, 300.0);
        assert!(g.enemies[1].health < 0.0);
        assert_eq!(g.level.crates, vec![vec2(264.0, 120.0)]);
    }
    #[test]
    fn immortality_blocks_all_damage_until_it_expires() {
        let mut g = game();
        g.player.immortal = 0.1;
        g.hurt(999.0);
        g.explode(g.player.pos, 999.0, 120.0, true);
        assert_eq!(g.player.health, 100.0);
        for _ in 0..13 {
            g.update(Input::default(), STEP);
        }
        g.hurt(20.0);
        assert_eq!(g.player.health, 80.0);
    }
    #[test]
    fn rocket_impact_damages_a_cluster() {
        let mut g = game();
        g.player.weapon = 3;
        g.player.pos = vec2(72.0, 120.0);
        g.player.aim = Vec2::X;
        g.enemies.push(monster(vec2(220.0, 120.0), Kind::Runner));
        g.enemies.push(monster(vec2(245.0, 165.0), Kind::Runner));
        g.shoot();
        g.update_bullets(0.4);
        assert!(g.bullets.is_empty());
        assert_eq!(g.blasts.len(), 1);
        assert!(g.enemies.iter().all(|e| e.health < 300.0));
    }
    #[test]
    fn advanced_monsters_fire_their_distinct_weapons() {
        for kind in [Kind::Gunner, Kind::Hunter, Kind::Bomber, Kind::Siege] {
            let mut g = game();
            g.player.velocity = vec2(80.0, 0.0);
            g.enemies.push(monster(vec2(350.0, 120.0), kind));
            for _ in 0..150 {
                g.update_enemies(STEP);
            }
            match kind {
                Kind::Bomber => assert!(!g.grenades.is_empty()),
                Kind::Siege => assert!(g.bullets.iter().any(|b| b.blast_radius > 0.0)),
                _ => assert!(g.bullets.len() >= 3),
            }
        }
    }

    #[test]
    fn new_monsters_and_bosses_use_their_distinct_attacks() {
        for kind in [
            Kind::Lancer,
            Kind::Sentinel,
            Kind::ThornTyrant,
            Kind::BroodMatriarch,
            Kind::Overlord,
        ] {
            let mut g = game();
            g.enemies.push(monster(vec2(350.0, 120.0), kind));
            g.update_enemies(STEP);
            assert!(g.enemies[0].tell > 0.0);
            assert!(
                g.bullets.is_empty() && g.grenades.is_empty(),
                "attack must be telegraphed"
            );
            for _ in 0..150 {
                g.update_enemies(STEP);
            }
            match kind {
                Kind::Lancer => {
                    assert_eq!(g.bullets.len(), 1);
                    assert!(g.bullets[0].velocity.length() > 800.0);
                }
                Kind::Sentinel => assert_eq!(g.bullets.len(), 5),
                Kind::ThornTyrant => {
                    assert_eq!(g.bullets.len(), 12);
                    assert!(g.bullets.iter().any(|b| b.velocity.x > 0.0));
                    assert!(g.bullets.iter().any(|b| b.velocity.x < 0.0));
                }
                Kind::BroodMatriarch => {
                    assert_eq!(g.grenades.len(), 3);
                    assert!(g.grenades[0].target.distance(g.grenades[1].target) > 50.0);
                }
                Kind::Overlord => {
                    assert_eq!(g.bullets.len(), 3);
                    assert!(g.bullets.iter().all(|b| b.blast_radius > 0.0));
                    assert_eq!(g.grenades.len(), 1);
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn stalker_weaves_toward_player_faster_than_a_ravager() {
        let mut runner = game();
        let mut stalker = game();
        runner
            .enemies
            .push(monster(vec2(350.0, 120.0), Kind::Runner));
        stalker
            .enemies
            .push(monster(vec2(350.0, 120.0), Kind::Stalker));
        for i in 0..60 {
            runner.time = i as f32 * STEP;
            stalker.time = runner.time;
            runner.update_enemies(STEP);
            stalker.update_enemies(STEP);
        }
        assert!(
            stalker.enemies[0].pos.distance(stalker.player.pos)
                < runner.enemies[0].pos.distance(runner.player.pos)
        );
        assert!(stalker.bullets.is_empty());
        assert!(stalker
            .level
            .circle_free(stalker.enemies[0].pos, Kind::Stalker.radius()));
    }
}
