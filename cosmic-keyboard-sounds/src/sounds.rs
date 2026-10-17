/// Procedurally synthesised mechanical keyboard sounds.
///
/// Each theme models the physical characteristics of the real switch:
///   - transient layer  : the initial click/impact burst
///   - body layer       : switch housing resonance
///   - bottom-out layer : key hitting the PCB/plate
///   - tail layer       : spring / housing ring-out
///
/// The `pitch` parameter (1.0 = normal) lets per-key-group tuning shift
/// the fundamental frequencies so spacebar sounds deeper than a letter key.

use std::f32::consts::PI;

pub struct Sound {
    pub data: Vec<u8>,
}

// ─── WAV writer ──────────────────────────────────────────────────────────────

fn write_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let data_size = (samples.len() * 2) as u32;
    let mut w = Vec::with_capacity(44 + data_size as usize);
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + data_size).to_le_bytes());
    w.extend_from_slice(b"WAVE");
    w.extend_from_slice(b"fmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes()); // PCM
    w.extend_from_slice(&1u16.to_le_bytes()); // mono
    w.extend_from_slice(&sample_rate.to_le_bytes());
    w.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&data_size.to_le_bytes());
    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        w.extend_from_slice(&v.to_le_bytes());
    }
    w
}

// ─── Noise generator (deterministic, fast) ───────────────────────────────────
// LCG-based noise — no allocations, good distribution

struct Lcg(u64);
impl Lcg {
    fn new(seed: u64) -> Self { Self(seed ^ 0xdeadbeef_cafebabe) }
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        // map to [-1, 1]
        ((self.0 >> 33) as f32 / (u32::MAX as f32 / 2.0)) - 1.0
    }
}

// ─── Building-block primitives ───────────────────────────────────────────────

/// Exponential envelope
#[inline]
fn exp_env(t: f32, rate: f32) -> f32 {
    (-t * rate).exp()
}

/// Sine oscillator
#[inline]
fn sine(t: f32, freq: f32) -> f32 {
    (2.0 * PI * freq * t).sin()
}

/// Noise burst: noise shaped by an exp envelope + resonant sine
fn noise_burst(t: f32, freq: f32, decay: f32, rng: &mut Lcg) -> f32 {
    let n = rng.next();
    let env = exp_env(t, decay);
    let tone = sine(t, freq) * 0.3;
    (n * 0.7 + tone) * env
}

/// Soft saturation
#[inline]
fn soft_clip(x: f32) -> f32 {
    x.tanh()
}

// ─── Cherry MX Blue (Clicky) ─────────────────────────────────────────────────

pub fn cherry_mx_blue_press(seed: u64, pitch: f32) -> Sound {
    let sr = 44100u32;
    let dur = 0.20_f32;
    let n = (sr as f32 * dur) as usize;
    let mut rng = Lcg::new(seed);
    // Small per-keystroke random offset for organic feel
    let r = (seed % 31) as f32 * 0.008;

    let samples: Vec<f32> = (0..n).map(|i| {
        let t = i as f32 / sr as f32;

        // 1. Click transient (0–18ms): sharp resonant pop
        let click = if t < 0.018 {
            let f = (3600.0 + r * 500.0) * pitch;
            noise_burst(t, f, 160.0, &mut rng) * exp_env(t, 140.0)
                + sine(t, f * 1.4) * exp_env(t, 220.0) * 0.4
        } else { 0.0 };

        // 2. Bottom-out thud (12–70ms): plate/PCB impact
        let thud = if t >= 0.012 && t < 0.07 {
            let tt = t - 0.012;
            let f = (260.0 + r * 40.0) * pitch;
            noise_burst(tt, f, 48.0, &mut rng) * 0.55
        } else { 0.0 };

        // 3. Housing ring (20–200ms): switch body resonance
        let ring = if t >= 0.020 {
            let tr = t - 0.020;
            let f = (1100.0 + r * 120.0) * pitch;
            sine(tr, f) * exp_env(tr, 24.0) * 0.14
        } else { 0.0 };

        // 4. Spring ping (35–200ms): return spring vibration
        let spring = if t >= 0.035 {
            let ts = t - 0.035;
            let f = (420.0 + r * 50.0) * pitch;
            sine(ts, f) * exp_env(ts, 18.0) * 0.08
                + sine(ts, f * 2.1) * exp_env(ts, 30.0) * 0.04
        } else { 0.0 };

        soft_clip((click + thud + ring + spring) * 1.2)
    }).collect();

    Sound { data: write_wav(&samples, sr) }
}

pub fn cherry_mx_blue_release(seed: u64, pitch: f32) -> Sound {
    let sr = 44100u32;
    let dur = 0.12_f32;
    let n = (sr as f32 * dur) as usize;
    let mut rng = Lcg::new(seed.wrapping_add(111));
    let r = (seed % 17) as f32 * 0.01;

    let samples: Vec<f32> = (0..n).map(|i| {
        let t = i as f32 / sr as f32;

        let f = (2800.0 + r * 400.0) * pitch;
        let click = noise_burst(t, f, 190.0, &mut rng) * exp_env(t, 170.0) * 0.55;
        let ring  = sine(t, 800.0 * pitch) * exp_env(t, 35.0) * 0.08;
        soft_clip(click + ring)
    }).collect();

    Sound { data: write_wav(&samples, sr) }
}

// ─── Cherry MX Brown (Soft Tactile) ──────────────────────────────────────────

pub fn cherry_mx_brown_press(seed: u64, pitch: f32) -> Sound {
    let sr = 44100u32;
    let dur = 0.22_f32;
    let n = (sr as f32 * dur) as usize;
    let mut rng = Lcg::new(seed);
    let r = (seed % 29) as f32 * 0.007;

    let samples: Vec<f32> = (0..n).map(|i| {
        let t = i as f32 / sr as f32;

        // Softer bump — lower freq, no sharp click
        let bump = {
            let f = (1600.0 + r * 200.0) * pitch;
            noise_burst(t, f, 80.0, &mut rng) * exp_env(t, 65.0) * 0.65
        };

        // Heavier bottom-out
        let thud = if t >= 0.018 && t < 0.09 {
            let tt = t - 0.018;
            let f = (200.0 + r * 30.0) * pitch;
            noise_burst(tt, f, 40.0, &mut rng) * 0.60
        } else { 0.0 };

        let body = if t >= 0.012 {
            let tb = t - 0.012;
            let f = (580.0 + r * 80.0) * pitch;
            sine(tb, f) * exp_env(tb, 20.0) * 0.11
        } else { 0.0 };

        soft_clip((bump + thud + body) * 1.1)
    }).collect();

    Sound { data: write_wav(&samples, sr) }
}

pub fn cherry_mx_brown_release(seed: u64, pitch: f32) -> Sound {
    let sr = 44100u32;
    let dur = 0.10_f32;
    let n = (sr as f32 * dur) as usize;
    let mut rng = Lcg::new(seed.wrapping_add(222));
    let r = (seed % 13) as f32 * 0.009;

    let samples: Vec<f32> = (0..n).map(|i| {
        let t = i as f32 / sr as f32;
        let f = (1200.0 + r * 150.0) * pitch;
        let v = noise_burst(t, f, 110.0, &mut rng) * exp_env(t, 90.0) * 0.38;
        soft_clip(v)
    }).collect();

    Sound { data: write_wav(&samples, sr) }
}

// ─── Cherry MX Red (Silent Linear) ───────────────────────────────────────────

pub fn cherry_mx_red_press(seed: u64, pitch: f32) -> Sound {
    let sr = 44100u32;
    let dur = 0.18_f32;
    let n = (sr as f32 * dur) as usize;
    let mut rng = Lcg::new(seed);
    let r = (seed % 23) as f32 * 0.006;

    let samples: Vec<f32> = (0..n).map(|i| {
        let t = i as f32 / sr as f32;

        // Very quiet: just a soft bottom-out thud
        let f = (165.0 + r * 20.0) * pitch;
        let thud = noise_burst(t, f, 30.0, &mut rng) * exp_env(t, 28.0) * 0.38;

        // Subtle keycap rattle
        let rattle = if t >= 0.006 {
            let tr = t - 0.006;
            sine(tr, 380.0 * pitch) * exp_env(tr, 16.0) * 0.05
        } else { 0.0 };

        soft_clip(thud + rattle)
    }).collect();

    Sound { data: write_wav(&samples, sr) }
}

pub fn cherry_mx_red_release(seed: u64, pitch: f32) -> Sound {
    let sr = 44100u32;
    let dur = 0.08_f32;
    let n = (sr as f32 * dur) as usize;
    let mut rng = Lcg::new(seed.wrapping_add(333));
    let r = (seed % 11) as f32 * 0.005;

    let samples: Vec<f32> = (0..n).map(|i| {
        let t = i as f32 / sr as f32;
        let f = (150.0 + r * 15.0) * pitch;
        let v = noise_burst(t, f, 45.0, &mut rng) * exp_env(t, 42.0) * 0.18;
        soft_clip(v)
    }).collect();

    Sound { data: write_wav(&samples, sr) }
}

// ─── Typewriter ───────────────────────────────────────────────────────────────

pub fn typewriter_press(seed: u64, pitch: f32) -> Sound {
    let sr = 44100u32;
    let dur = 0.30_f32;
    let n = (sr as f32 * dur) as usize;
    let mut rng = Lcg::new(seed);
    let r = (seed % 37) as f32 * 0.010;

    let samples: Vec<f32> = (0..n).map(|i| {
        let t = i as f32 / sr as f32;

        // Hard metal impact
        let f_impact = (3900.0 + r * 400.0) * pitch;
        let impact = noise_burst(t, f_impact, 180.0, &mut rng) * exp_env(t, 145.0);

        // Metal bar resonance (the type arm)
        let arm = if t >= 0.004 {
            let ta = t - 0.004;
            let f1 = (360.0 + r * 50.0) * pitch;
            let f2 = f1 * 2.4;
            let env = exp_env(ta, 11.0);
            (sine(ta, f1) * 0.55 + sine(ta, f2) * 0.25) * env * 0.5
        } else { 0.0 };

        // Platen (roller) low thud
        let platen = if t >= 0.010 {
            let tp = t - 0.010;
            let f = 130.0 * pitch;
            noise_burst(tp, f, 22.0, &mut rng) * 0.35
        } else { 0.0 };

        // Long metallic ring tail
        let ring = if t >= 0.025 {
            let tr = t - 0.025;
            let f = (490.0 + r * 40.0) * pitch;
            sine(tr, f) * exp_env(tr, 7.0) * 0.15
                + sine(tr, f * 1.6) * exp_env(tr, 11.0) * 0.08
        } else { 0.0 };

        soft_clip((impact + arm + platen + ring) * 1.3)
    }).collect();

    Sound { data: write_wav(&samples, sr) }
}

pub fn typewriter_release(seed: u64, pitch: f32) -> Sound {
    let sr = 44100u32;
    let dur = 0.15_f32;
    let n = (sr as f32 * dur) as usize;
    let mut rng = Lcg::new(seed.wrapping_add(444));
    let r = (seed % 19) as f32 * 0.008;

    let samples: Vec<f32> = (0..n).map(|i| {
        let t = i as f32 / sr as f32;
        let f = (2800.0 + r * 300.0) * pitch;
        let click = noise_burst(t, f, 145.0, &mut rng) * exp_env(t, 120.0) * 0.45;
        let ring  = sine(t, 430.0 * pitch) * exp_env(t, 13.0) * 0.12;
        soft_clip(click + ring)
    }).collect();

    Sound { data: write_wav(&samples, sr) }
}
