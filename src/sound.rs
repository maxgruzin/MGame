use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use mgame::simulation::SoundEvent;

pub struct Audio {
    sounds: Vec<Option<Sound>>,
}
impl Audio {
    pub async fn new() -> Self {
        let mut sounds = vec![];
        for (frequency, duration, noise) in [
            (160.0, 0.09, 0.65),
            (80.0, 0.20, 0.85),
            (900.0, 0.22, 0.22),
            (500.0, 0.045, 0.6),
            (100.0, 0.18, 0.5),
            (400.0, 0.16, 0.3),
            (620.0, 0.32, 0.0),
            (55.0, 0.55, 0.85),
            (1100.0, 0.3, 0.0),
        ] {
            let bytes = wav(frequency, duration, noise);
            sounds.push(load_sound_from_bytes(&bytes).await.ok());
        }
        Self { sounds }
    }
    pub fn play(&self, events: &[SoundEvent], enabled: bool) {
        if !enabled {
            return;
        }
        let mut played = [false; 9];
        for event in events {
            let i = match event {
                SoundEvent::Rifle => 0,
                SoundEvent::Shotgun => 1,
                SoundEvent::Rail => 2,
                SoundEvent::Hit => 3,
                SoundEvent::Hurt => 4,
                SoundEvent::Dash => 5,
                SoundEvent::Wave => 6,
                SoundEvent::Explosion => 7,
                SoundEvent::Pickup => 8,
            };
            if played[i] {
                continue;
            }
            played[i] = true;
            if let Some(sound) = &self.sounds[i] {
                play_sound(
                    sound,
                    PlaySoundParams {
                        looped: false,
                        volume: if i == 3 { 0.14 } else { 0.25 },
                    },
                );
            }
        }
    }
}
fn wav(frequency: f32, duration: f32, noise: f32) -> Vec<u8> {
    let rate = 22050u32;
    let count = (duration * rate as f32) as u32;
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend((36 + count * 2).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16u32.to_le_bytes());
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(rate.to_le_bytes());
    bytes.extend((rate * 2).to_le_bytes());
    bytes.extend(2u16.to_le_bytes());
    bytes.extend(16u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend((count * 2).to_le_bytes());
    let mut seed = 12345u32;
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let envelope = (1.0 - t / duration).max(0.0).powi(2);
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let n = (seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let tone = (std::f32::consts::TAU * frequency * (t - t * t / (duration * 2.5))).sin();
        let sample = ((tone * (1.0 - noise) + n * noise) * envelope * 24000.0) as i16;
        bytes.extend(sample.to_le_bytes());
    }
    bytes
}
