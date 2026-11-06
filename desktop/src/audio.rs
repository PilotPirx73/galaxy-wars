//! Alle Klänge werden beim Start synthetisch erzeugt (keine Audiodateien).

use macroquad::audio::{
    load_sound_from_bytes, play_sound, play_sound_once, set_sound_volume, stop_sound,
    PlaySoundParams, Sound,
};
use std::f32::consts::TAU;

const SR: usize = 44100;
const MASTER: f32 = 0.55;

#[derive(Clone, Copy)]
pub enum Sfx {
    Fire,
    Planet,
    Sun,
    Hole,
    Reveal,
    Lost,
    Boom,
    Win,
}

#[derive(Clone, Copy)]
enum Wave {
    Sine,
    Square,
    Tri,
    Saw,
}

#[derive(Clone, Copy)]
enum Filt {
    Low,
    High,
    Band,
}

struct Buf {
    d: Vec<f32>,
    seed: u32,
}

impl Buf {
    fn new(len: f32) -> Self {
        Buf { d: vec![0.0; (len * SR as f32) as usize + 1], seed: 0x9E37_79B9 }
    }

    fn rnd(&mut self) -> f32 {
        let mut x = self.seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.seed = x;
        x as f32 / u32::MAX as f32 * 2.0 - 1.0
    }

    fn env(t: f32, dur: f32, vol: f32) -> f32 {
        let att = 0.012;
        if t < att {
            vol * t / att
        } else {
            vol * (0.0001f32 / vol).powf(((t - att) / (dur - att)).min(1.0))
        }
    }

    fn tone(&mut self, w: Wave, f0: f32, f1: f32, dur: f32, vol: f32, delay: f32) -> &mut Self {
        let st = (delay * SR as f32) as usize;
        let n = (dur * SR as f32) as usize;
        let mut ph = 0.0f32;
        for i in 0..n {
            let t = i as f32 / SR as f32;
            let f = f0 * (f1 / f0).powf(t / dur);
            ph = (ph + f / SR as f32).fract();
            let s = wave(w, ph);
            if let Some(x) = self.d.get_mut(st + i) {
                *x += s * Self::env(t, dur, vol);
            }
        }
        self
    }

    fn noise(&mut self, k: Filt, f0: f32, f1: f32, dur: f32, vol: f32, delay: f32) -> &mut Self {
        let st = (delay * SR as f32) as usize;
        let n = (dur * SR as f32) as usize;
        let coef = |fc: f32| 1.0 - (-TAU * fc.min(20000.0) / SR as f32).exp();
        let (mut l1, mut l2) = (0.0f32, 0.0f32);
        for i in 0..n {
            let t = i as f32 / SR as f32;
            let fc = f0 * (f1 / f0).powf(t / dur);
            let x = self.rnd();
            let y = match k {
                Filt::Low => {
                    l1 += coef(fc) * (x - l1);
                    l1
                }
                Filt::High => {
                    l1 += coef(fc) * (x - l1);
                    x - l1
                }
                Filt::Band => {
                    l1 += coef(fc * 1.6) * (x - l1);
                    l2 += coef(fc * 0.6) * (x - l2);
                    (l1 - l2) * 1.8
                }
            };
            if let Some(v) = self.d.get_mut(st + i) {
                *v += y * Self::env(t, dur, vol);
            }
        }
        self
    }
}

fn wave(w: Wave, ph: f32) -> f32 {
    match w {
        Wave::Sine => (ph * TAU).sin(),
        Wave::Square => {
            if ph < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        Wave::Tri => 1.0 - 4.0 * (ph - 0.5).abs(),
        Wave::Saw => 2.0 * ph - 1.0,
    }
}

fn to_wav(d: &[f32], gain: f32) -> Vec<u8> {
    let n = d.len();
    let mut v = Vec::with_capacity(44 + n * 2);
    v.extend(b"RIFF");
    v.extend(((36 + n * 2) as u32).to_le_bytes());
    v.extend(b"WAVEfmt ");
    v.extend(16u32.to_le_bytes());
    v.extend(1u16.to_le_bytes()); // PCM
    v.extend(1u16.to_le_bytes()); // mono
    v.extend((SR as u32).to_le_bytes());
    v.extend((SR as u32 * 2).to_le_bytes());
    v.extend(2u16.to_le_bytes());
    v.extend(16u16.to_le_bytes());
    v.extend(b"data");
    v.extend(((n * 2) as u32).to_le_bytes());
    for s in d {
        let x = (s * gain).clamp(-1.0, 1.0);
        v.extend(((x * 32767.0) as i16).to_le_bytes());
    }
    v
}

async fn load(b: &Buf) -> Option<Sound> {
    load_sound_from_bytes(&to_wav(&b.d, MASTER)).await.ok()
}

pub struct Audio {
    sfx: Vec<Option<Sound>>,
    hum: Vec<Option<Sound>>,
    hum_f: Vec<f32>,
    pub muted: bool,
    pub fly_enabled: bool,
    flying: bool,
    fly_t: f32,
    cur_f: f32,
}

impl Audio {
    pub async fn new() -> Self {
        use Filt::*;
        use Wave::*;
        let mut defs: Vec<Buf> = Vec::new();

        let mut b = Buf::new(0.3); // Fire
        b.tone(Square, 900.0, 160.0, 0.2, 0.1, 0.0).noise(Band, 5000.0, 700.0, 0.14, 0.18, 0.0);
        defs.push(b);
        let mut b = Buf::new(0.55); // Planet
        b.noise(Low, 1200.0, 90.0, 0.45, 0.4, 0.0).tone(Sine, 130.0, 45.0, 0.35, 0.3, 0.0);
        defs.push(b);
        let mut b = Buf::new(1.2); // Sonne
        b.noise(High, 7000.0, 1800.0, 1.1, 0.28, 0.0).tone(Saw, 320.0, 60.0, 0.6, 0.05, 0.0);
        defs.push(b);
        let mut b = Buf::new(1.5); // Schwarzes Loch
        b.tone(Sine, 420.0, 28.0, 1.4, 0.28, 0.0)
            .tone(Tri, 640.0, 35.0, 1.2, 0.12, 0.06)
            .noise(Low, 2500.0, 60.0, 1.1, 0.14, 0.0);
        defs.push(b);
        let mut b = Buf::new(1.7); // Aufgedeckt
        b.tone(Sine, 55.0, 42.0, 1.6, 0.35, 0.0)
            .tone(Tri, 110.0, 84.0, 1.4, 0.1, 0.0)
            .tone(Sine, 1760.0, 1500.0, 0.9, 0.03, 0.0);
        defs.push(b);
        let mut b = Buf::new(0.8); // Verloren
        b.tone(Tri, 520.0, 140.0, 0.7, 0.07, 0.0);
        defs.push(b);
        let mut b = Buf::new(1.7); // Explosion
        b.noise(Low, 3500.0, 50.0, 1.6, 0.7, 0.0)
            .tone(Sine, 95.0, 24.0, 1.4, 0.55, 0.0)
            .noise(High, 8000.0, 2000.0, 0.5, 0.3, 0.05);
        defs.push(b);
        let mut b = Buf::new(1.9); // Siegesmelodie
        for (i, f) in [523.0, 659.0, 784.0, 1047.0].iter().enumerate() {
            let d = if i == 3 { 0.4 } else { 0.16 };
            b.tone(Square, *f, *f, d, 0.07, 0.95 + i as f32 * 0.13);
        }
        defs.push(b);

        let mut sfx = Vec::new();
        for d in &defs {
            sfx.push(load(d).await);
        }

        // Fluggeräusch: mehrere Schleifen in verschiedenen Tonhöhen, die je nach Tempo überblendet werden
        let mut hum = Vec::new();
        let mut hum_f = Vec::new();
        for i in 0..8 {
            let f = (140.0 * 10f32.powf(i as f32 / 7.0)).round();
            let f2 = (f * 1.012).round().max(f + 1.0);
            let mut b = Buf::new(1.0);
            let n = SR;
            b.d.truncate(n);
            for (k, s) in b.d.iter_mut().enumerate() {
                let t = k as f32 / SR as f32;
                *s = 0.5 * wave(Tri, (f * t).fract()) + 0.5 * wave(Sine, (f2 * t).fract());
            }
            hum.push(load_sound_from_bytes(&to_wav(&b.d, 0.5)).await.ok());
            hum_f.push(f);
        }

        Audio { sfx, hum, hum_f, muted: false, fly_enabled: true, flying: false, fly_t: 0.0, cur_f: 200.0 }
    }

    pub fn play(&self, s: Sfx) {
        if self.muted {
            return;
        }
        if let Some(Some(snd)) = self.sfx.get(s as usize) {
            play_sound_once(snd);
        }
    }

    pub fn fly_start(&mut self) {
        self.fly_stop();
        if self.muted || !self.fly_enabled {
            return;
        }
        for h in self.hum.iter().flatten() {
            play_sound(h, PlaySoundParams { looped: true, volume: 0.0 });
        }
        self.flying = true;
        self.fly_t = 0.0;
        self.cur_f = 200.0;
    }

    pub fn fly_update(&mut self, speed: f64, dt: f32) {
        if !self.flying {
            return;
        }
        self.fly_t += dt;
        let target = (140.0 + speed as f32 * 0.55).min(1400.0);
        self.cur_f += (target - self.cur_f) * (dt * 25.0).min(1.0);
        let n = self.hum_f.len();
        let pos = ((self.cur_f / self.hum_f[0]).ln() / (self.hum_f[n - 1] / self.hum_f[0]).ln()
            * (n - 1) as f32)
            .clamp(0.0, (n - 1) as f32 - 0.0001);
        let i = pos.floor() as usize;
        let fr = pos - i as f32;
        let v = 0.12 * (self.fly_t / 0.1).min(1.0);
        for (k, h) in self.hum.iter().enumerate() {
            if let Some(h) = h {
                let vol = if k == i {
                    (1.0 - fr) * v
                } else if k == i + 1 {
                    fr * v
                } else {
                    0.0
                };
                set_sound_volume(h, vol);
            }
        }
    }

    pub fn fly_stop(&mut self) {
        if !self.flying {
            return;
        }
        for h in self.hum.iter().flatten() {
            stop_sound(h);
        }
        self.flying = false;
    }
}
