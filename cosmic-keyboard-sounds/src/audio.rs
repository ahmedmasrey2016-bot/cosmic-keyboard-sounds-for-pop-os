use crate::config::{Config, KeyGroup, Theme};
use crate::sounds;
use anyhow::{Context, Result};
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

enum AudioMsg {
    Play(Vec<u8>, f32 /* volume */),
}

/// Audio engine — all rodio objects live on a single dedicated thread.
pub struct AudioEngine {
    config: Config,
    tx: mpsc::SyncSender<AudioMsg>,
}

impl AudioEngine {
    pub fn new(config: Config) -> Result<Self> {
        let (tx, rx) = mpsc::sync_channel::<AudioMsg>(64);

        std::thread::Builder::new()
            .name("audio".into())
            .spawn(move || audio_thread(rx))
            .context("Failed to spawn audio thread")?;

        Ok(Self { config, tx })
    }

    /// Play a key-press sound for the given key code
    pub fn play_press(&self, key_code: u16) {
        let (seed, pitch, vol) = self.params(key_code);
        let wav = self.gen_press(seed, pitch);
        self.send(wav, vol);
    }

    /// Play a key-release sound for the given key code
    pub fn play_release(&self, key_code: u16) {
        let (seed, pitch, vol) = self.params(key_code);
        let wav = self.gen_release(seed, pitch);
        self.send(wav, vol);
    }

    // ── Internal ─────────────────────────────────────────────────────────────

    fn params(&self, key_code: u16) -> (u64 /* seed */, f32 /* pitch */, f32 /* vol */) {
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64;
        let seed = (key_code as u64).wrapping_mul(6364136223846793005)
            ^ t.wrapping_mul(2862933555777941757);

        let (pitch, vol_factor) = if self.config.per_key_group_sounds {
            let group = KeyGroup::from_key_code(key_code);
            (group.pitch_factor(), group.volume_factor())
        } else {
            (1.0, 1.0)
        };

        // ±4% random pitch variation for organic feel
        let pitch = if self.config.randomize_pitch {
            let jitter = (seed % 1000) as f32 / 12500.0; // 0.0–0.08
            pitch * (0.96 + jitter)
        } else {
            pitch
        };

        let vol = self.config.volume * vol_factor;
        (seed, pitch, vol)
    }

    fn gen_press(&self, seed: u64, pitch: f32) -> Vec<u8> {
        match self.config.theme {
            Theme::CherryMxBlue  => sounds::cherry_mx_blue_press(seed, pitch).data,
            Theme::CherryMxBrown => sounds::cherry_mx_brown_press(seed, pitch).data,
            Theme::CherryMxRed   => sounds::cherry_mx_red_press(seed, pitch).data,
            Theme::Typewriter    => sounds::typewriter_press(seed, pitch).data,
        }
    }

    fn gen_release(&self, seed: u64, pitch: f32) -> Vec<u8> {
        match self.config.theme {
            Theme::CherryMxBlue  => sounds::cherry_mx_blue_release(seed, pitch).data,
            Theme::CherryMxBrown => sounds::cherry_mx_brown_release(seed, pitch).data,
            Theme::CherryMxRed   => sounds::cherry_mx_red_release(seed, pitch).data,
            Theme::Typewriter    => sounds::typewriter_release(seed, pitch).data,
        }
    }

    fn send(&self, wav: Vec<u8>, vol: f32) {
        match self.tx.try_send(AudioMsg::Play(wav, vol)) {
            Ok(_) => {}
            Err(mpsc::TrySendError::Full(_)) => {
                debug!("Audio channel full, dropping sound");
            }
            Err(mpsc::TrySendError::Disconnected(_)) => {
                warn!("Audio thread has exited");
            }
        }
    }
}

// ── Audio thread ──────────────────────────────────────────────────────────────

fn audio_thread(rx: mpsc::Receiver<AudioMsg>) {
    use rodio::{Decoder, DeviceSinkBuilder, Player};
    use std::io::Cursor;

    let sink = match DeviceSinkBuilder::open_default_sink() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[audio] Could not open audio device: {e}");
            return;
        }
    };

    // Keep players alive until they finish; prune on each new sound
    let mut players: Vec<Player> = Vec::with_capacity(16);

    while let Ok(AudioMsg::Play(wav_bytes, volume)) = rx.recv() {
        // Prune finished players
        players.retain(|p| !p.empty());

        let cursor = Cursor::new(wav_bytes);
        let decoder = match Decoder::new(cursor) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[audio] Decode error: {e}");
                continue;
            }
        };

        let player = Player::connect_new(sink.mixer());
        player.set_volume(volume as rodio::Float);
        player.append(decoder);
        player.play();
        players.push(player);
    }
    // rx dropped → thread exits cleanly
}
