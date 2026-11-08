use macroquad::prelude::*;
use mgame::{
    level::Level,
    party::*,
    party_client::PartyClient,
    party_net::{Connection, MAX_COMMAND},
    party_server::PartyServer,
    simulation::*,
    waves,
};
use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    time::{Duration, Instant},
};

fn arena() -> Level {
    Level::parse("party test", "################\n#..............#\n#..P...........#\n#..............#\n#..............#\n#............E.#\n#..............#\n################").unwrap()
}
fn game(count: usize) -> Game {
    let mut game = Game::new_party(arena(), false, count).unwrap();
    game.intermission = 100.0;
    game
}
fn enemy(pos: Vec2, kind: Kind) -> Enemy {
    Enemy {
        pos,
        previous: pos,
        kind,
        health: kind.base_health(),
        max_health: kind.base_health(),
        attack: 0.0,
        tell: 0.0,
        aim: Vec2::X,
        phase: 0.0,
    }
}
fn steps(game: &mut Game, count: usize) {
    for _ in 0..count {
        game.update_party(&[], STEP);
    }
}

#[test]
fn eight_players_spawn_safely_near_start_on_every_builtin() {
    for index in 0..mgame::arenas::NAMES.len() {
        let game = Game::new_party(Level::builtin(index), false, 8).unwrap();
        assert_eq!(game.players().count(), 8);
        for player in game.players() {
            assert!(game.level.circle_free(player.pos, PLAYER_RADIUS));
            assert!(player.pos.distance(game.level.spawn) <= 240.0);
            for other in game.players().filter(|p| p.slot != player.slot) {
                assert!(player.pos.distance(other.pos) >= 30.0);
            }
        }
    }
}

#[test]
fn party_scales_counts_health_and_all_attack_damage() {
    for count in 1..=8 {
        for wave in 1..=21 {
            let roster = waves::party_roster(wave, count, 75);
            assert_eq!(
                roster.len(),
                (waves::roster(wave).len() * (100 + (count - 1) * 75)).div_ceil(100)
            );
            assert_eq!(
                roster.iter().filter(|k| k.is_boss()).count(),
                usize::from(waves::boss_for_wave(wave).is_some())
            );
        }
        let mut g = game(count);
        let scale = 1.0 + (count - 1) as f32 * 15.0 / 100.0;
        g.wave = 1;
        g.spawns.push(Spawn {
            pos: vec2(600.0, 240.0),
            timer: 0.0,
            kind: Kind::Runner,
        });
        steps(&mut g, 1);
        assert_eq!(g.enemies[0].max_health, 48.0 * scale);
        g.enemies = vec![enemy(g.player.pos, Kind::Runner)];
        steps(&mut g, 1);
        assert!((g.player.health - (100.0 - 9.0 * scale)).abs() < 0.001);
        g.enemies = vec![enemy(g.player.pos + vec2(130.0, 0.0), Kind::Gunner)];
        g.enemies[0].tell = STEP * 0.5;
        steps(&mut g, 1);
        assert_eq!(g.bullets[0].damage, 12.0 * scale);
        g.enemies = vec![enemy(g.player.pos + vec2(130.0, 0.0), Kind::Bomber)];
        g.enemies[0].tell = STEP * 0.5;
        steps(&mut g, 1);
        assert_eq!(g.grenades[0].damage, 45.0 * scale);
    }
}

#[test]
fn every_player_moves_once_per_tick_and_can_switch_weapon() {
    let mut g = game(8);
    let positions: Vec<_> = g.players().map(|p| p.pos).collect();
    let input = Input {
        movement: Vec2::X,
        weapon: Some(4),
        ..Default::default()
    };
    g.update_party(&[input; 8], STEP);
    assert_eq!(g.time, STEP);
    for (i, player) in g.players().enumerate() {
        assert!(player.pos.x > positions[i].x);
        assert_eq!(player.weapon, 4);
        assert!(player.pos.distance(positions[i]) < 1.0);
    }
}

#[test]
fn allies_are_targeted_and_dead_leader_does_not_stop_the_match() {
    let mut g = game(2);
    g.player.health = 0.0;
    g.enemies.push(enemy(g.allies[0].pos, Kind::Runner));
    steps(&mut g, 1);
    assert!(g.allies[0].health < 100.0);
    assert_eq!(g.player.health, 0.0);
    assert_eq!(g.time, STEP);
    g.allies[0].invulnerable = 0.0;
    g.enemies.clear();
    let pos = g.allies[0].pos - Vec2::X * 10.0;
    g.bullets.push(Bullet {
        pos,
        previous: pos,
        origin: pos,
        velocity: Vec2::X * 1100.0,
        damage: 12.0,
        life: 1.0,
        hostile: true,
        rail: false,
        blast_radius: 0.0,
    });
    let before = g.allies[0].health;
    steps(&mut g, 1);
    assert_eq!(g.allies[0].health, before - 12.0);
}

fn down(g: &mut Game, index: usize) -> Vec2 {
    let pos = g.players().nth(index).unwrap().pos;
    g.bullets.push(Bullet {
        pos,
        previous: pos,
        origin: pos,
        velocity: Vec2::ZERO,
        damage: 1000.0,
        life: 1.0,
        hostile: true,
        rail: false,
        blast_radius: 0.0,
    });
    steps(g, 1);
    assert_eq!(g.players().nth(index).unwrap().health, 0.0);
    pos
}

#[test]
fn death_immediately_creates_a_beacon_at_the_exact_death_position() {
    let mut g = game(3);
    g.allies[0].pos += vec2(3.25, 7.5);
    let pos = down(&mut g, 1);
    assert_eq!(g.beacons.len(), 1);
    assert_eq!(g.beacons[0].pos, pos);
    let second = down(&mut g, 2);
    assert_eq!(g.beacons.len(), 2);
    assert_eq!(g.beacons[1].pos, second);
    steps(&mut g, 5);
    assert_eq!(g.beacons.len(), 2);
    assert_eq!(g.kills, 0);
}

#[test]
fn revive_drains_twice_as_fast_when_empty_or_contested_and_recovers() {
    let mut g = game(2);
    down(&mut g, 1);
    g.beacons[0].progress = 5.0;
    let origin = g.player.pos;
    g.player.pos = vec2(600.0, 240.0);
    steps(&mut g, 120);
    assert!((g.beacons[0].progress - 3.0).abs() < 0.001);
    g.player.pos = origin;
    let mut blocking = enemy(g.beacons[0].pos, Kind::Runner);
    blocking.attack = 100.0;
    g.enemies.push(blocking);
    steps(&mut g, 120);
    assert!((g.beacons[0].progress - 1.0).abs() < 0.001);
    steps(&mut g, 120);
    assert_eq!(g.beacons[0].progress, 0.0);
    g.enemies.clear();
    steps(&mut g, 600);
    assert!((g.beacons[0].progress - 5.0).abs() < 0.001);
    g.allies[0].ammo = [0; 5];
    steps(&mut g, 601);
    assert_eq!(g.allies[0].health, 100.0);
    assert_eq!(g.allies[0].ammo, WEAPONS.map(|w| w.magazine));
    assert!(g.allies[0].invulnerable > 2.9);
    assert!(g.beacons.is_empty());
}

#[test]
fn more_helpers_revive_faster_and_random_selection_is_not_join_order() {
    let mut selected = [false; 2];
    for offset in 0..24 {
        let mut g = game(4);
        // Vary the simulation's deterministic random sequence before the revive.
        g.burst(g.player.pos, WHITE, offset, 1.0);
        down(&mut g, 1);
        down(&mut g, 2);
        g.beacons.truncate(1);
        let pos = g.beacons[0].pos;
        g.player.pos = pos;
        g.allies[2].pos = pos + Vec2::X * 30.0;
        g.beacons[0].progress = 0.0;
        steps(&mut g, 300);
        assert!((g.beacons[0].progress - 5.0).abs() < 0.001);
        steps(&mut g, 301);
        assert_eq!(g.players().filter(|p| p.health > 0.0).count(), 3);
        let revived = g
            .allies
            .iter()
            .take(2)
            .position(|p| p.health > 0.0)
            .unwrap();
        selected[revived] = true;
        assert_eq!(g.allies[revived].health, 100.0);
        assert!(g.beacons.is_empty());
    }
    assert_eq!(selected, [true, true]);
}

#[test]
fn party_explosives_are_safe_for_allies_but_hostile_explosives_hit_them() {
    let mut g = game(2);
    for hostile in [false, true] {
        let pos = g.allies[0].pos;
        g.grenades.push(Grenade {
            pos,
            origin: pos,
            target: pos,
            age: 1.999,
            fuse: 2.0,
            damage: 30.0,
            hostile,
        });
        steps(&mut g, 1);
        assert_eq!(g.allies[0].health, if hostile { 70.0 } else { 100.0 });
    }
}

#[test]
fn disconnected_players_do_not_revive_or_change_difficulty() {
    let mut g = game(2);
    g.disconnect_player(1);
    g.beacons.push(ReviveBeacon {
        pos: g.player.pos,
        progress: 9.9,
    });
    steps(&mut g, 20);
    assert_eq!(g.allies[0].health, 0.0);
    assert_eq!(g.party_size, 2);
    assert!(g.beacons.is_empty());
    g.disconnect_player(0);
    let time = g.time;
    steps(&mut g, 20);
    assert_eq!(g.time, time);
    assert!(!g.team_alive());
}

#[test]
fn small_arena_still_spawns_when_party_covers_the_original_spawn_margin() {
    let mut g = Game::new_party(
        Level::parse(
            "small",
            "########\n#.....E#\n#......#\n#..P...#\n#......#\n#......#\n#......#\n########",
        )
        .unwrap(),
        false,
        8,
    )
    .unwrap();
    g.intermission = 0.0;
    steps(&mut g, 1);
    assert_eq!(g.wave, 1);
    assert_eq!(g.remaining + g.spawns.len() as u32, 188);
    assert!(!g.spawns.is_empty());
}

#[test]
fn dead_players_stay_dead_through_wave_healing_and_final_wave_finishes_for_survivors() {
    let mut g = game(2);
    g.player.health = 0.0;
    g.level.medical.push(g.player.pos);
    g.intermission = 0.0;
    steps(&mut g, 1);
    assert_eq!(g.player.health, 0.0);
    g.wave = 20;
    g.remaining = 0;
    g.enemies.clear();
    g.spawns.clear();
    steps(&mut g, 1);
    assert!(g.victory);
    assert_eq!(g.player.health, 0.0);
}

#[test]
fn pushed_cover_cannot_overlap_a_living_ally() {
    let mut g = game(2);
    let cover = vec2(360.0, 168.0);
    g.level.crates = vec![cover];
    g.player.pos = cover - vec2(35.0, 0.0);
    g.allies[0].pos = cover + vec2(35.0, 0.0);
    for _ in 0..120 {
        g.update_party(
            &[Input {
                movement: Vec2::X,
                ..Default::default()
            }],
            STEP,
        );
    }
    assert!(g.level.crates[0].x < cover.x + 3.0);
    assert!(g.level.circle_free(g.allies[0].pos, PLAYER_RADIUS));
}

#[test]
fn wire_snapshot_transfers_cover_and_preserves_local_identity() {
    let mut g = game(3);
    g.level.crates = vec![vec2(500.0, 200.0)];
    g.allies[1].health = 37.0;
    let data = serde_json::to_vec(&Snapshot::capture(&g, 12)).unwrap();
    let snapshot: Snapshot = serde_json::from_slice(&data).unwrap();
    let mut view = game(3);
    snapshot.apply(&mut view, 2).unwrap();
    assert_eq!(view.player.slot, 2);
    assert_eq!(view.player.health, 37.0);
    assert_eq!(view.level.crates, g.level.crates);
    assert_eq!(view.allies.len(), 2);
}

#[test]
fn lobby_rejects_invalid_starts_and_transfers_host() {
    let mut room = Room::new(arena(), false).unwrap();
    for id in 1..=8 {
        room.join(id, &format!("Player{id}")).unwrap();
    }
    assert!(room.join(9, "Overflow").is_err());
    assert!(room.command(1, &ClientMessage::Start).is_err());
    for id in 1..=8 {
        room.command(id, &ClientMessage::Ready(true)).unwrap();
    }
    room.command(8, &ClientMessage::Ready(false)).unwrap();
    assert!(room.command(1, &ClientMessage::Start).is_err());
    room.command(8, &ClientMessage::Ready(true)).unwrap();
    assert!(room.command(2, &ClientMessage::Start).is_err());
    room.leave(1);
    assert_eq!(room.lobby.host, 2);
    room.command(2, &ClientMessage::Start).unwrap();
    assert!(room
        .command(2, &ClientMessage::Difficulty(Difficulty::default()))
        .is_err());
    assert!(room.join(9, "Late").is_err());
    assert!(room.command(2, &ClientMessage::ReturnToLobby).is_err());
    let g = room.game.as_mut().unwrap();
    g.victory = true;
    room.command(2, &ClientMessage::ReturnToLobby).unwrap();
    assert!(room.game.is_none());
    assert!(room.lobby.members.iter().all(|m| !m.ready));
}

#[test]
fn inputs_reject_nonfinite_or_extreme_coordinates_and_invalid_weapons() {
    for aim in [Vec2::NAN, Vec2::splat(f32::INFINITY), Vec2::splat(1e9)] {
        assert!(sanitize_input(Input {
            aim,
            ..Default::default()
        })
        .is_err());
    }
    let input = sanitize_input(Input {
        movement: Vec2::ONE,
        weapon: Some(4),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(input.weapon, Some(4));
    assert!(sanitize_input(Input {
        weapon: Some(5),
        ..Default::default()
    })
    .is_err());
    assert!((input.movement.length() - 1.0).abs() < 0.001);
}

fn pump(
    server: &mut PartyServer,
    clients: &mut [PartyClient],
    predicate: impl Fn(&PartyServer, &[PartyClient]) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        for client in clients.iter_mut() {
            client.poll().unwrap();
        }
        server.poll().unwrap();
        if predicate(server, clients) {
            return;
        }
        assert!(Instant::now() < deadline, "network test timed out");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn real_sockets_synchronize_eight_clients_and_handle_host_departure() {
    let mut server = PartyServer::bind(
        "127.0.0.1:0".parse().unwrap(),
        Room::new(arena(), false).unwrap(),
    )
    .unwrap();
    let address = server.local_addr().unwrap();
    let mut clients: Vec<_> = (0..8)
        .map(|i| PartyClient::new(TcpStream::connect(address).unwrap(), format!("P{i}")).unwrap())
        .collect();
    pump(&mut server, &mut clients, |_, c| {
        c.iter()
            .all(|c| c.lobby.as_ref().is_some_and(|l| l.members.len() == 8))
    });
    let mut rejected =
        PartyClient::new(TcpStream::connect(address).unwrap(), "Ninth".into()).unwrap();
    rejected.poll().unwrap();
    server.poll().unwrap();
    rejected.poll().unwrap();
    assert!(rejected.notice.contains("full"));
    let difficulty = Difficulty {
        enemy_health: 30,
        weapon_damage: [5, 10, 15, 20, 25],
        ..Default::default()
    };
    clients[0]
        .send(ClientMessage::Difficulty(difficulty))
        .unwrap();
    pump(&mut server, &mut clients, |_, c| {
        c.iter()
            .all(|c| c.lobby.as_ref().unwrap().difficulty == difficulty)
    });
    for client in &mut clients {
        client.send(ClientMessage::Ready(true)).unwrap();
    }
    pump(&mut server, &mut clients, |s, _| {
        s.room.lobby.members.iter().all(|m| m.ready)
    });
    clients[0].send(ClientMessage::Start).unwrap();
    pump(&mut server, &mut clients, |_, c| {
        c.iter().all(|c| c.tick >= 6)
    });
    assert!(clients
        .iter()
        .all(|c| c.game.as_ref().unwrap().players().count() == 8));
    for client in &clients {
        assert_eq!(client.game.as_ref().unwrap().player.weapon, 0);
        assert_eq!(client.game.as_ref().unwrap().difficulty, difficulty);
        assert_eq!(
            client.game.as_ref().unwrap().player.name,
            format!("P{}", client.slot)
        );
    }
    server.room.game.as_mut().unwrap().intermission = 100.0;
    let start = clients[3].game.as_ref().unwrap().player.pos;
    clients[3]
        .send(ClientMessage::Input(Input {
            movement: Vec2::X,
            weapon: Some(4),
            ..Default::default()
        }))
        .unwrap();
    pump(&mut server, &mut clients, |_, c| {
        c[3].game.as_ref().unwrap().player.pos.x > start.x + 10.0
    });
    let cutoff = Instant::now() + Duration::from_millis(700);
    pump(&mut server, &mut clients, |_, _| Instant::now() >= cutoff);
    assert!(
        server
            .room
            .game
            .as_ref()
            .unwrap()
            .players()
            .nth(3)
            .unwrap()
            .velocity
            .length()
            < 0.1
    );
    assert_eq!(
        server
            .room
            .game
            .as_ref()
            .unwrap()
            .players()
            .nth(3)
            .unwrap()
            .weapon,
        4
    );
    let first = clients.remove(0);
    let old_id = first.id;
    drop(first);
    pump(&mut server, &mut clients, |s, c| {
        s.room.lobby.host != old_id && c[0].lobby.as_ref().unwrap().host != old_id
    });
    assert!(!server.room.game.as_ref().unwrap().player.connected);
    server.room.game.as_mut().unwrap().victory = true;
    clients[0].send(ClientMessage::ReturnToLobby).unwrap();
    pump(&mut server, &mut clients, |_, c| {
        c.iter().all(|c| c.game.is_none())
    });
}

#[test]
fn framing_handles_fragmented_messages_and_rejects_large_frames() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut writer = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    writer.set_nodelay(true).unwrap();
    let mut reader = Connection::new(listener.accept().unwrap().0, MAX_COMMAND).unwrap();
    let data = serde_json::to_vec(&ClientMessage::Ready(true)).unwrap();
    let length = (data.len() as u32).to_be_bytes();
    writer.write_all(&length[..2]).unwrap();
    assert!(reader.receive::<ClientMessage>().unwrap().is_empty());
    writer.write_all(&length[2..]).unwrap();
    writer.write_all(&data).unwrap();
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        if !reader.receive::<ClientMessage>().unwrap().is_empty() {
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    writer
        .write_all(&((MAX_COMMAND + 1) as u32).to_be_bytes())
        .unwrap();
    loop {
        if reader.receive::<ClientMessage>().is_err() {
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn difficulty_is_host_only_validated_and_clears_readiness() {
    let mut room = Room::new(arena(), false).unwrap();
    room.join(1, "Host").unwrap();
    room.join(2, "Friend").unwrap();
    let changed = Difficulty {
        enemy_count: 300,
        enemy_damage: 0,
        enemy_health: 25,
        weapon_damage: [-50, 100, 0, 5, 10],
    };
    assert!(room
        .command(2, &ClientMessage::Difficulty(changed))
        .is_err());
    for invalid in [
        Difficulty {
            enemy_count: 301,
            ..changed
        },
        Difficulty {
            enemy_damage: 7,
            ..changed
        },
        Difficulty {
            enemy_health: 305,
            ..changed
        },
        Difficulty {
            weapon_damage: [-55; 5],
            ..changed
        },
        Difficulty {
            weapon_damage: [105; 5],
            ..changed
        },
        Difficulty {
            weapon_damage: [1; 5],
            ..changed
        },
    ] {
        assert!(room
            .command(1, &ClientMessage::Difficulty(invalid))
            .is_err());
    }
    room.command(1, &ClientMessage::Ready(true)).unwrap();
    room.command(2, &ClientMessage::Ready(true)).unwrap();
    room.command(1, &ClientMessage::Difficulty(changed))
        .unwrap();
    assert!(room.lobby.members.iter().all(|m| !m.ready));
    assert!(room.command(1, &ClientMessage::Start).is_err());
    room.command(1, &ClientMessage::Ready(true)).unwrap();
    room.command(2, &ClientMessage::Ready(true)).unwrap();
    room.command(1, &ClientMessage::Start).unwrap();
    assert_eq!(room.game.as_ref().unwrap().difficulty, changed);
    assert_eq!(room.game.as_ref().unwrap().player.name, "Host");
    assert_eq!(room.game.as_ref().unwrap().allies[0].name, "Friend");
    room.leave(2);
    assert_eq!(room.game.as_ref().unwrap().difficulty, changed);
    assert_eq!(room.game.as_ref().unwrap().party_count_scale(), 4.0);
    room.leave(1);
    assert_eq!(room.lobby.difficulty, Difficulty::default());
}

#[test]
fn configurable_counts_keep_one_boss_and_health_damage_are_independent() {
    for count in 1..=8 {
        for percent in [0, 5, 75, 300] {
            for wave in 0..=21 {
                let roster = waves::party_roster(wave, count, percent);
                assert_eq!(
                    roster.len(),
                    (waves::roster(wave).len() * (100 + (count - 1) * percent as usize))
                        .div_ceil(100)
                );
                assert_eq!(
                    roster.iter().filter(|k| k.is_boss()).count(),
                    usize::from(waves::boss_for_wave(wave).is_some())
                );
            }
        }
    }
    let mut g = game(3);
    g.difficulty = Difficulty {
        enemy_count: 0,
        enemy_health: 300,
        enemy_damage: 50,
        ..Default::default()
    };
    g.intermission = 0.0;
    steps(&mut g, 1);
    assert_eq!(g.remaining + g.spawns.len() as u32, 30);
    g.spawns = vec![Spawn {
        pos: vec2(600.0, 240.0),
        timer: 0.0,
        kind: Kind::Runner,
    }];
    steps(&mut g, 1);
    assert_eq!(g.enemies[0].max_health, 48.0 * 7.0);
    g.enemies = vec![enemy(g.player.pos, Kind::Runner)];
    steps(&mut g, 1);
    assert_eq!(g.player.health, 100.0 - 9.0 * 2.0);
    g.enemies = vec![enemy(g.player.pos + vec2(130.0, 0.0), Kind::Siege)];
    g.enemies[0].tell = STEP * 0.5;
    steps(&mut g, 1);
    assert!(g.bullets.iter().any(|b| b.hostile && b.blast_radius > 0.0));
}

#[test]
fn every_weapon_uses_its_own_damage_adjustment_including_explosives_and_quad() {
    for (weapon, info) in WEAPONS.iter().enumerate() {
        for percent in [-50, 0, 100] {
            for quad in [false, true] {
                let mut g = game(2);
                g.difficulty.weapon_damage[weapon] = percent;
                // Test the ally path as well as primary-player firing.
                g.allies[0].quad = if quad { 10.0 } else { 0.0 };
                let input = Input {
                    weapon: Some(weapon),
                    fire: true,
                    aim: g.allies[0].pos + Vec2::X * 300.0,
                    ..Default::default()
                };
                g.update_party(&[Input::default(), input], STEP);
                let expected =
                    info.damage * (1.0 + percent as f32 / 100.0) * if quad { 4.0 } else { 1.0 };
                if weapon == 4 {
                    assert_eq!(g.grenades.len(), 1);
                    assert_eq!(g.grenades[0].damage, expected);
                } else {
                    assert_eq!(g.bullets.len(), info.pellets);
                    assert!(g.bullets.iter().all(|b| b.damage == expected));
                    assert_eq!(g.bullets[0].blast_radius > 0.0, weapon == 3);
                }
            }
        }
    }
}
