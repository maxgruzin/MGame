use crate::simulation::Kind;

pub const CAMPAIGN_WAVES: u32 = 20;

pub fn boss_for_wave(wave: u32) -> Option<Kind> {
    match wave {
        7 => Some(Kind::ThornTyrant),
        15 => Some(Kind::BroodMatriarch),
        20 => Some(Kind::Overlord),
        _ => None,
    }
}

pub fn health_scale(wave: u32) -> f32 {
    1.0 + wave.saturating_sub(1) as f32 * 0.04
}

pub fn speed_scale(wave: u32) -> f32 {
    1.1 * (1.0 + wave.saturating_sub(1) as f32 * 0.012).min(1.5)
}

pub fn spawn_interval(wave: u32) -> f32 {
    (0.9 - wave.saturating_sub(1) as f32 * 0.02).max(0.3)
}

// Costs build the base roster before applying the enemy count multiplier.
// Bosses spend part of the same growing budget as their escorts.
pub fn threat_cost(kind: Kind) -> u32 {
    match kind {
        Kind::Runner => 1,
        Kind::Gunner | Kind::Stalker => 2,
        Kind::Hunter | Kind::Lancer => 3,
        Kind::Brute | Kind::Bomber | Kind::Sentinel => 4,
        Kind::Siege => 6,
        Kind::ThornTyrant => 12,
        Kind::BroodMatriarch => 18,
        Kind::Overlord => 25,
    }
}

pub fn roster(wave: u32) -> Vec<Kind> {
    let mut enemies = base_roster(wave);
    let base_count = enemies.len();
    let count = (base_count * 5).div_ceil(2);
    // Keep one scheduled boss and repeat its escorts to reach 2.5x the total
    // headcount. Cycling the base roster preserves species unlocks and variety.
    let first_regular = usize::from(boss_for_wave(wave).is_some());
    for i in base_count..count {
        let source = first_regular + (i - base_count) % (base_count - first_regular);
        enemies.push(enemies[source]);
    }
    enemies
}

/// Extra players add a configurable fraction of the roster; bosses remain unique.
pub fn party_roster(wave: u32, players: usize, additional_percent: u16) -> Vec<Kind> {
    let mut enemies = roster(wave);
    let base = enemies.len();
    let regular: Vec<_> = enemies.iter().copied().filter(|k| !k.is_boss()).collect();
    if !regular.is_empty() {
        let percent = 100 + players.clamp(1, 8).saturating_sub(1) * additional_percent as usize;
        for i in base..(base * percent).div_ceil(100) {
            enemies.push(regular[(i - base) % regular.len()]);
        }
    }
    enemies
}

fn base_roster(wave: u32) -> Vec<Kind> {
    if wave == 0 {
        return vec![];
    }
    let mut budget = 12 + wave.saturating_sub(1) * 4;
    let mut enemies = vec![];
    if let Some(boss) = boss_for_wave(wave) {
        enemies.push(boss);
        budget -= threat_cost(boss);
    }
    let available: Vec<_> = [
        (1, Kind::Runner),
        (2, Kind::Gunner),
        (3, Kind::Brute),
        (3, Kind::Hunter),
        (5, Kind::Bomber),
        (6, Kind::Siege),
        (8, Kind::Stalker),
        (11, Kind::Lancer),
        (16, Kind::Sentinel),
    ]
    .into_iter()
    .filter(|(unlock, _)| wave >= *unlock)
    .map(|(_, kind)| kind)
    .collect();
    // Lead with the newest species so every unlock is guaranteed to appear.
    let mut index = available.len() - 1;
    while budget > 0 {
        let candidate = available[index % available.len()];
        let kind = if threat_cost(candidate) <= budget {
            candidate
        } else {
            Kind::Runner
        };
        enemies.push(kind);
        budget -= threat_cost(kind);
        index += 1;
    }
    enemies
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_budget_grows_through_every_boss_transition() {
        let mut previous = 0;
        for wave in 1..=CAMPAIGN_WAVES {
            let enemies = base_roster(wave);
            let threat: u32 = enemies.iter().map(|&kind| threat_cost(kind)).sum();
            assert_eq!(threat, 12 + (wave - 1) * 4);
            assert!(threat > previous);
            assert!(health_scale(wave + 1) > health_scale(wave));
            assert!(speed_scale(wave + 1) > speed_scale(wave));
            assert!(spawn_interval(wave + 1) < spawn_interval(wave));
            let bosses: Vec<_> = enemies.into_iter().filter(|kind| kind.is_boss()).collect();
            assert_eq!(bosses, boss_for_wave(wave).into_iter().collect::<Vec<_>>());
            previous = threat;
        }
    }

    #[test]
    fn waves_have_two_and_a_half_times_the_enemies_with_one_scheduled_boss() {
        assert!(roster(0).is_empty());
        // Original counts include the boss on waves 7, 15, and 20.
        let original_counts = [
            12, 11, 8, 10, 10, 12, 10, 15, 14, 17, 18, 19, 20, 22, 17, 23, 24, 27, 26, 21, 29, 32,
            33, 35, 35,
        ];
        for (index, count) in original_counts.into_iter().enumerate() {
            let wave = index as u32 + 1;
            let enemies = roster(wave);
            assert_eq!(enemies.len(), (count * 5_usize).div_ceil(2), "wave {wave}");
            let bosses: Vec<_> = enemies
                .iter()
                .copied()
                .filter(|kind| kind.is_boss())
                .collect();
            assert_eq!(bosses, boss_for_wave(wave).into_iter().collect::<Vec<_>>());
            assert!(enemies.starts_with(&base_roster(wave)));
        }
    }

    #[test]
    fn enemy_speed_bonus_applies_before_and_after_the_endless_cap() {
        for (wave, previous) in [(1, 1.0), (7, 1.072), (20, 1.228), (100, 1.5)] {
            assert!((speed_scale(wave) - previous * 1.1).abs() < 0.0001);
        }
    }

    #[test]
    fn new_species_are_guaranteed_from_their_unlock_wave() {
        for (unlock, kind) in [(8, Kind::Stalker), (11, Kind::Lancer), (16, Kind::Sentinel)] {
            assert!((1..unlock).all(|wave| !roster(wave).contains(&kind)));
            assert!((unlock..=CAMPAIGN_WAVES).all(|wave| roster(wave).contains(&kind)));
        }
    }
}
