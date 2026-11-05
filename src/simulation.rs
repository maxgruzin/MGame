use crate::level::{Level, TILE};
use macroquad::prelude::*;

pub const STEP: f32 = 1.0 / 120.0;
pub const PLAYER_RADIUS: f32 = 13.0;

#[derive(Clone, Copy, Default)]
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
        damage: 19.0,
        pellets: 8,
        spread: 0.25,
        speed: 920.0,
    },
    Weapon {
        name: "RAIL CARBINE",
        magazine: 6,
        interval: 0.48,
        reload: 1.4,
        damage: 110.0,
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
pub struct Player {
    pub pos: Vec2,
    pub previous: Vec2,
    pub velocity: Vec2,
    pub aim: Vec2,
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
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Runner,
    Gunner,
    Brute,
    Hunter,
    Bomber,
    Siege,
}
impl Kind {
    pub fn radius(self) -> f32 {
        if matches!(self, Self::Brute | Self::Siege) {
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
        }
    }
}
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
pub struct Bullet {
    pub pos: Vec2,
    pub previous: Vec2,
    pub velocity: Vec2,
    pub damage: f32,
    pub life: f32,
    pub hostile: bool,
    pub rail: bool,
    pub blast_radius: f32,
}
pub struct Grenade {
    pub pos: Vec2,
    pub origin: Vec2,
    pub target: Vec2,
    pub age: f32,
    pub fuse: f32,
    pub damage: f32,
    pub hostile: bool,
}
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
pub struct Pickup {
    pub pos: Vec2,
    pub life: f32,
    pub kind: PickupKind,
}
#[derive(Clone, Copy, Debug, PartialEq)]
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
pub struct Spawn {
    pub pos: Vec2,
    pub timer: f32,
    pub kind: Kind,
}
#[derive(Clone, Copy)]
pub enum SoundEvent {
    Rifle,
    Shotgun,
    Rail,
    Hit,
    Hurt,
    Dash,
    Wave,
    Explosion,
    Pickup,
}
pub struct Game {
    pub level: Level,
    pub player: Player,
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
    field_timer: f32,
    rng: u64,
}

impl Game {
    pub fn new(level: Level, endless: bool) -> Self {
        let pos = level.spawn;
        let field = level.flow_field(pos);
        Self {
            level,
            player: Player {
                pos,
                previous: pos,
                velocity: Vec2::ZERO,
                aim: Vec2::X,
                health: 100.0,
                weapon: 0,
                ammo: [30, 7, 6, 4, 5],
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
            field_timer: 0.0,
            rng: 0xabc123def456,
        }
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
        self.time += dt;
        self.shake = (self.shake - dt * 18.0).max(0.0);
        self.hit_flash = (self.hit_flash - dt).max(0.0);
        self.combo_timer -= dt;
        if self.combo_timer <= 0.0 {
            self.combo = 0;
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
            .collect();
        self.player.pos =
            self.level
                .push_slide(self.player.pos, velocity * dt, PLAYER_RADIUS, &actors);
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
        self.field_timer -= dt;
        if self.field_timer <= 0.0 {
            self.field = self.level.flow_field(self.player.pos);
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
            self.score +=
                (if kind == Kind::Brute { 150 } else { 75 }) * (1 + self.combo.min(20) / 5);
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
            if pickup.pos.distance(self.player.pos) < 36.0
                && (pickup.kind != PickupKind::Health || self.player.health < 100.0)
                && self.level.clear_shot(pickup.pos, self.player.pos)
            {
                match pickup.kind {
                    PickupKind::Health => {
                        self.player.health = (self.player.health + 30.0).min(100.0)
                    }
                    PickupKind::Quad => self.player.quad = 12.0,
                    PickupKind::Immortal => self.player.immortal = 8.0,
                    PickupKind::Haste => self.player.haste = 10.0,
                }
                pickup.life = 0.0;
                self.sounds.push(SoundEvent::Pickup);
            }
        }
        self.pickups.retain(|p| p.life > 0.0);
        if self.level.medical.iter().any(|p| {
            p.distance(self.player.pos) < 34.0 && self.level.clear_shot(*p, self.player.pos)
        }) {
            self.player.health = (self.player.health + 4.0 * dt).min(100.0);
        }
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
        let damage = w.damage * if self.player.quad > 0.0 { 4.0 } else { 1.0 };
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
    fn update_waves(&mut self, dt: f32) {
        if self.remaining == 0 && self.enemies.is_empty() && self.spawns.is_empty() {
            if self.wave >= 6 && !self.endless {
                self.victory = true;
                return;
            }
            self.intermission -= dt;
            if self.intermission <= 0.0 {
                self.wave += 1;
                self.remaining = 7 + self.wave * 4;
                self.spawn_timer = 0.0;
                self.intermission = 4.0;
                self.sounds.push(SoundEvent::Wave);
                self.player.health = (self.player.health + 10.0).min(100.0);
                let kind = match self.wave % 3 {
                    1 => PickupKind::Quad,
                    2 => PickupKind::Haste,
                    _ => PickupKind::Immortal,
                };
                let supply = (0..self.level.tiles.len())
                    .filter(|&i| self.field[i] != u32::MAX)
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
            && self.enemies.len() + self.spawns.len() < 48
        {
            let mut candidates: Vec<Vec2> = self
                .level
                .portals
                .iter()
                .copied()
                .filter(|p| {
                    self.field[self.level.index(*p)] != u32::MAX
                        && self.level.circle_free(*p, 19.0)
                        && p.distance(self.player.pos) > TILE * 4.0
                        && self.enemies.iter().all(|e| e.pos.distance(*p) > 38.0)
                        && self.spawns.iter().all(|s| s.pos.distance(*p) > 38.0)
                })
                .collect();
            if candidates.is_empty() {
                let farthest = (0..self.level.tiles.len())
                    .filter(|&i| self.field[i] != u32::MAX)
                    .map(|i| self.level.center(i).distance(self.player.pos))
                    .fold(0.0_f32, f32::max);
                let safe_distance = (farthest * 0.7).min(TILE * 4.0);
                candidates = (0..self.level.tiles.len())
                    .filter(|&i| self.field[i] != u32::MAX)
                    .map(|i| self.level.center(i))
                    .filter(|p| {
                        p.distance(self.player.pos) > safe_distance
                            && self.enemies.iter().all(|e| e.pos.distance(*p) > 42.0)
                            && self.spawns.iter().all(|s| s.pos.distance(*p) > 42.0)
                    })
                    .collect();
            }
            if !candidates.is_empty() {
                let idx = (self.random() * candidates.len() as f32) as usize % candidates.len();
                let roll = self.random();
                let kind = if self.wave >= 6 && roll < 0.18 {
                    Kind::Siege
                } else if self.wave >= 5 && roll < 0.36 {
                    Kind::Bomber
                } else if self.wave >= 3 && roll < 0.52 {
                    Kind::Hunter
                } else if self.wave >= 3 && roll < 0.70 {
                    Kind::Brute
                } else if self.wave >= 2 && self.random() < 0.4 {
                    Kind::Gunner
                } else {
                    Kind::Runner
                };
                self.spawns.push(Spawn {
                    pos: candidates[idx],
                    timer: 0.85,
                    kind,
                });
                self.remaining -= 1;
            }
            self.spawn_timer = (0.95 - self.wave as f32 * 0.07).max(0.28);
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
            let health = match kind {
                Kind::Runner => 48.0,
                Kind::Gunner => 65.0,
                Kind::Brute => 190.0,
                Kind::Hunter => 110.0,
                Kind::Bomber => 150.0,
                Kind::Siege => 310.0,
            } * (1.0 + self.wave.saturating_sub(1) as f32 * 0.06);
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
        let mut damage = 0.0;
        let mut mortars = vec![];
        for (i, e) in self.enemies.iter_mut().enumerate() {
            e.previous = e.pos;
            e.attack -= dt;
            let to_player = self.player.pos - e.pos;
            let distance = to_player.length();
            let visible = distance < 700.0 && self.level.clear_shot(e.pos, self.player.pos);
            let waypoint =
                self.level
                    .waypoint(e.pos, self.player.pos, &self.field, e.kind.radius());
            let mut direction = (waypoint - e.pos).normalize_or_zero();
            let mut speed = match e.kind {
                Kind::Runner => 145.0,
                Kind::Gunner => 110.0,
                Kind::Brute => 85.0,
                Kind::Hunter => 155.0,
                Kind::Bomber => 95.0,
                Kind::Siege => 75.0,
            };
            speed *= (1.0 + self.wave as f32 * 0.025).min(1.45);
            let range = match e.kind {
                Kind::Hunter => 530.0,
                Kind::Bomber => 470.0,
                Kind::Siege => 500.0,
                _ => 360.0,
            };
            if matches!(
                e.kind,
                Kind::Gunner | Kind::Hunter | Kind::Bomber | Kind::Siege
            ) && (visible || e.kind == Kind::Bomber)
                && distance < range
            {
                let forward = to_player.normalize_or_zero();
                direction = vec2(-forward.y, forward.x) * (self.time * 0.7 + e.phase).sin() * 0.75;
                if distance < range * 0.5 {
                    direction -= forward;
                }
                if e.attack <= 0.0 && e.tell <= 0.0 {
                    e.tell = if e.kind == Kind::Siege || e.kind == Kind::Bomber {
                        0.9
                    } else {
                        0.6
                    };
                    e.aim = (self.player.pos
                        + self.player.velocity * if e.kind == Kind::Hunter { 0.26 } else { 0.2 }
                        - e.pos)
                        .normalize_or_zero();
                }
            }
            if e.tell > 0.0 {
                e.tell -= dt;
                direction *= 0.25;
                if e.tell <= 0.0 {
                    if e.kind == Kind::Bomber {
                        mortars.push((e.pos, self.player.pos + self.player.velocity * 0.4));
                    } else {
                        let count = if e.kind == Kind::Gunner || e.kind == Kind::Hunter {
                            3
                        } else {
                            1
                        };
                        for k in 0..count {
                            let spread = (k as f32 - (count - 1) as f32 * 0.5)
                                * if e.kind == Kind::Hunter { 0.055 } else { 0.22 };
                            self.bullets.push(Bullet {
                                pos: e.pos,
                                previous: e.pos,
                                velocity: Vec2::from_angle(e.aim.to_angle() + spread)
                                    * match e.kind {
                                        Kind::Hunter => 650.0,
                                        Kind::Siege => 300.0,
                                        _ => 320.0,
                                    },
                                damage: if e.kind == Kind::Siege { 38.0 } else { 12.0 },
                                life: 3.0,
                                hostile: true,
                                rail: e.kind == Kind::Hunter,
                                blast_radius: if e.kind == Kind::Siege { 110.0 } else { 0.0 },
                            });
                        }
                    }
                    e.attack = match e.kind {
                        Kind::Bomber => 3.3,
                        Kind::Siege => 2.5,
                        Kind::Hunter => 2.0,
                        _ => 1.7,
                    };
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
            let steering = (direction + separation.clamp_length_max(0.65)).clamp_length_max(1.0);
            speed *= self.level.terrain_speed(e.pos);
            let next = self
                .level
                .slide(e.pos, steering * speed * dt, e.kind.radius());
            e.pos = if next.distance_squared(e.pos) < 0.01 && direction.length_squared() > 0.1 {
                self.level
                    .slide(e.pos, direction * speed * dt, e.kind.radius())
            } else {
                next
            };
            if distance < e.kind.radius() + PLAYER_RADIUS + 5.0 && e.attack <= 0.0 {
                damage += if e.kind == Kind::Brute { 22.0 } else { 9.0 };
                e.attack = 0.8;
            }
        }
        for (origin, target) in mortars {
            self.throw_grenade(origin, target, 45.0, true);
        }
        if damage > 0.0 {
            self.hurt(damage);
        }
    }
    fn hurt(&mut self, damage: f32) {
        if self.player.invulnerable > 0.0 || self.player.immortal > 0.0 {
            return;
        }
        self.player.health = (self.player.health - damage).max(0.0);
        self.player.invulnerable = 0.45;
        self.hit_flash = 0.2;
        self.shake = 6.0;
        self.sounds.push(SoundEvent::Hurt);
        self.burst(self.player.pos, Color::from_hex(0xff5c77), 12, 130.0);
    }
    fn update_bullets(&mut self, dt: f32) {
        let mut impacts = vec![];
        let mut hurt = 0.0;
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
                    if b.pos.distance(self.player.pos) < PLAYER_RADIUS + 3.0 {
                        if b.blast_radius > 0.0 {
                            explosions.push((b.pos, b.damage, b.blast_radius, b.hostile));
                        } else {
                            hurt += b.damage;
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
        if hurt > 0.0 {
            self.hurt(hurt);
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
        let distance = self.player.pos.distance(pos);
        if distance < radius + PLAYER_RADIUS && self.level.clear_shot(pos, self.player.pos) {
            self.hurt(
                damage
                    * (1.0 - 0.65 * (distance / radius).min(1.0))
                    * if hostile { 1.0 } else { 0.25 },
            );
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
    fn six_wave_objective_and_endless_mode_diverge() {
        let mut finite = game();
        finite.wave = 6;
        finite.update(Input::default(), STEP);
        assert!(finite.victory);
        let mut endless = game();
        endless.wave = 6;
        endless.endless = true;
        endless.intermission = 0.0;
        endless.update(Input::default(), STEP);
        assert!(!endless.victory);
        assert_eq!(endless.wave, 7);
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
}
