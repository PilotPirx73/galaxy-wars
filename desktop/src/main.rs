#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // unter Windows kein Konsolenfenster
//! Galaxy Wars 2026 – Artillerie zwischen Planeten.

mod audio;

use audio::{Audio, Sfx};
use macroquad::prelude::*;
use std::f32::consts::TAU;

// ---------- Konstanten ----------
const W: f64 = 1600.0;
const H: f64 = 1000.0;
const G: f64 = 25.0;
const SHIP_R: f64 = 13.0;
const SHOT_R: f64 = 3.0;
const DT: f64 = 1.0 / 240.0;
const SIM_SPEED: f64 = 1.6;
const V_PER: f64 = 5.2;
const MAX_T: f64 = 30.0;

const BG: Color = Color::new(0.020, 0.024, 0.051, 1.0);
const FG: Color = Color::new(0.91, 0.925, 1.0, 1.0);
const MUTED: Color = Color::new(0.54, 0.565, 0.69, 1.0);
const PANEL: Color = Color::new(0.055, 0.067, 0.133, 0.95);
const LINE: Color = Color::new(0.149, 0.169, 0.29, 1.0);
const BTN: Color = Color::new(0.086, 0.102, 0.2, 1.0);

fn pcol(i: usize) -> Color {
    if i == 0 {
        Color::new(0.31, 0.82, 1.0, 1.0)
    } else {
        Color::new(1.0, 0.42, 0.545, 1.0)
    }
}
fn alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}
fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}
fn hsl(h: f32, s: f32, l: f32) -> Color {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = (h / 60.0).rem_euclid(6.0);
    let x = c * (1.0 - ((hp % 2.0) - 1.0).abs());
    let (r, g, b) = match hp as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    Color::new(r + m, g + m, b + m, 1.0)
}
fn rnd(a: f64, b: f64) -> f64 {
    if b <= a {
        a
    } else {
        rand::gen_range(a, b)
    }
}
fn rndf(a: f32, b: f32) -> f32 {
    rand::gen_range(a, b)
}
fn gauss() -> f64 {
    ((0..6).map(|_| rnd(0.0, 1.0)).sum::<f64>() - 3.0) / 0.707
}
fn wrap360(a: f64) -> f64 {
    a.rem_euclid(360.0)
}
fn dist(ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt()
}

// ---------- Zeichenhilfen ----------
fn tw(s: &str, size: f32) -> f32 {
    measure_text(s, None, size.max(1.0) as u16, 1.0).width
}
fn txt(s: &str, x: f32, y: f32, size: f32, c: Color) {
    draw_text(s, x, y, size, c);
}
fn txt_c(s: &str, cx: f32, cy: f32, size: f32, c: Color) {
    draw_text(s, cx - tw(s, size) / 2.0, cy + size * 0.3, size, c);
}
fn glow(x: f32, y: f32, r_in: f32, r_out: f32, c: Color, a: f32) {
    let n = 12;
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        draw_circle(x, y, r_out + (r_in - r_out) * t, alpha(c, a / n as f32 * 1.6));
    }
}
fn sphere(x: f32, y: f32, r: f32, dark: Color, mid: Color, light: Color) {
    let n = 16;
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let rr = r * (1.0 - t * 0.85);
        let off = r * 0.38 * t;
        let c = if t < 0.5 { mix(dark, mid, t * 2.0) } else { mix(mid, light, (t - 0.5) * 2.0) };
        draw_circle(x - off, y - off, rr, c);
    }
}
fn ell_pt(cx: f32, cy: f32, rx: f32, ry: f32, rot: f32, a: f32) -> Vec2 {
    let (s, c) = rot.sin_cos();
    let (x, y) = (rx * a.cos(), ry * a.sin());
    vec2(cx + x * c - y * s, cy + x * s + y * c)
}
fn fill_ellipse(cx: f32, cy: f32, rx: f32, ry: f32, rot: f32, col: Color) {
    let n = 48;
    for i in 0..n {
        let a0 = i as f32 / n as f32 * TAU;
        let a1 = (i + 1) as f32 / n as f32 * TAU;
        draw_triangle(vec2(cx, cy), ell_pt(cx, cy, rx, ry, rot, a0), ell_pt(cx, cy, rx, ry, rot, a1), col);
    }
}
fn ellipse_arc(cx: f32, cy: f32, rx: f32, ry: f32, rot: f32, a0: f32, len: f32, th: f32, col: Color) {
    let n = 14;
    let mut p = ell_pt(cx, cy, rx, ry, rot, a0);
    for i in 1..=n {
        let q = ell_pt(cx, cy, rx, ry, rot, a0 + len * i as f32 / n as f32);
        draw_line(p.x, p.y, q.x, q.y, th, col);
        p = q;
    }
}

// ---------- Spielobjekte ----------
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Planet,
    Sun,
    Hole,
}

struct Body {
    kind: Kind,
    x: f64,
    y: f64,
    r: f64,
    m: f64,
    hue: f32,
    dens: f32,
    banded: bool,
    tilt: f32,
    spots: Vec<(f32, f32, f32)>,
    revealed: bool,
    reveal: f32,
}

impl Body {
    fn new(kind: Kind, x: f64, y: f64, r: f64, m: f64) -> Self {
        Body {
            kind,
            x,
            y,
            r,
            m,
            hue: rndf(0.0, 360.0),
            dens: 1.0,
            banded: rand::gen_range(0, 2) == 0,
            tilt: rndf(-0.5, 0.5),
            spots: (0..6).map(|_| (rndf(0.0, TAU), rndf(0.0, 0.7), rndf(0.08, 0.2))).collect(),
            revealed: true,
            reveal: 1.0,
        }
    }
}

struct Ship {
    x: f64,
    y: f64,
    dead: bool,
}

struct Player {
    angle: f64,
    power: f64,
    trails: Vec<Vec<(f64, f64)>>,
}

struct Shot {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    t: f64,
    owner: usize,
    pts: Vec<(f64, f64)>,
    n: u64,
}

#[derive(Clone, Copy)]
enum Hit {
    Body(Kind, usize),
    Ship(usize),
    Out,
    Time,
    Abort,
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Menu,
    Aim,
    Ai,
    Flying,
    Over,
}

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f32,
    max: f32,
    c: Color,
    r: f32,
    ring: Option<(f32, bool)>, // (Radius, nach innen)
    drag: bool,
}

struct Star {
    x: f32,
    y: f32,
    r: f32,
    a: f32,
}

struct AiAnim {
    a0: f64,
    da: f64,
    p0: f64,
    dp: f64,
    t: f32,
}

struct Settings {
    cpu: bool,
    ai_lvl: usize,
    planets: usize,
    sun: bool,
    holes: usize,
    holes_hidden: bool,
    endless: bool,
    fly_sound: bool,
    sound: bool,
    trails: bool,
}

// ---------- Physik ----------
fn step_shot(o: &mut Shot, dt: f64, bodies: &[Body], ships: &[Ship], limits: bool, known_only: bool) -> Option<Hit> {
    let (mut ax, mut ay) = (0.0, 0.0);
    for (i, p) in bodies.iter().enumerate() {
        if known_only && p.kind == Kind::Hole && !p.revealed {
            continue;
        }
        let dx = p.x - o.x;
        let dy = p.y - o.y;
        let d2 = dx * dx + dy * dy;
        if d2.sqrt() < p.r + SHOT_R {
            return Some(Hit::Body(p.kind, i));
        }
        // Glättung beim Schwarzen Loch, damit nahe Vorbeiflüge numerisch stabil bleiben
        let q = if p.kind == Kind::Hole { d2 + 900.0 } else { d2 };
        let f = G * p.m / (q * q.sqrt());
        ax += f * dx;
        ay += f * dy;
    }
    o.vx += ax * dt;
    o.vy += ay * dt;
    o.x += o.vx * dt;
    o.y += o.vy * dt;
    o.t += dt;
    for (i, s) in ships.iter().enumerate() {
        if s.dead || (i == o.owner && o.t < 0.25) {
            continue;
        }
        if dist(s.x, s.y, o.x, o.y) < SHIP_R + SHOT_R {
            return Some(Hit::Ship(i));
        }
    }
    if limits {
        if o.x < -W * 1.2 || o.x > W * 2.2 || o.y < -H * 1.2 || o.y > H * 2.2 {
            return Some(Hit::Out);
        }
        if o.t > MAX_T {
            return Some(Hit::Time);
        }
    } else if o.x.abs() > W * 100.0 || o.y.abs() > H * 100.0 {
        return Some(Hit::Out); // so weit draußen kommt nichts mehr zurück
    }
    None
}

fn launch(ship: &Ship, owner: usize, angle: f64, power: f64) -> Shot {
    let a = angle.to_radians();
    let (dx, dy) = (a.cos(), -a.sin());
    let v = power * V_PER;
    Shot {
        x: ship.x + dx * (SHIP_R + 5.0),
        y: ship.y + dy * (SHIP_R + 5.0),
        vx: dx * v,
        vy: dy * v,
        t: 0.0,
        owner,
        pts: vec![],
        n: 0,
    }
}

// ---------- Einfache Sofort-Oberfläche ----------
struct Ui {
    m: Vec2,
    pressed: bool,
    down: bool,
    active: Option<u32>,
}

impl Ui {
    fn begin(&mut self) {
        let (x, y) = mouse_position();
        self.m = vec2(x, y);
        self.pressed = is_mouse_button_pressed(MouseButton::Left);
        self.down = is_mouse_button_down(MouseButton::Left);
        if !self.down {
            self.active = None;
        }
    }
    fn hover(&self, r: Rect) -> bool {
        r.contains(self.m)
    }
    fn button(&mut self, r: Rect, label: &str, bg: Color, fg: Color, size: f32, enabled: bool) -> bool {
        let h = enabled && self.hover(r);
        let c = if !enabled {
            alpha(bg, 0.35)
        } else if h {
            mix(bg, WHITE, 0.15)
        } else {
            bg
        };
        draw_rectangle(r.x, r.y, r.w, r.h, c);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, alpha(LINE, if enabled { 1.0 } else { 0.4 }));
        txt_c(label, r.x + r.w / 2.0, r.y + r.h / 2.0, size, alpha(fg, if enabled { 1.0 } else { 0.4 }));
        h && self.pressed
    }
    fn slider(&mut self, id: u32, r: Rect, v: &mut f64, min: f64, max: f64, col: Color, enabled: bool) -> bool {
        let a = if enabled { 1.0 } else { 0.35 };
        let cy = r.y + r.h / 2.0;
        draw_line(r.x, cy, r.x + r.w, cy, 4.0, alpha(LINE, a));
        let t = ((*v - min) / (max - min)) as f32;
        draw_line(r.x, cy, r.x + r.w * t, cy, 4.0, alpha(col, a));
        draw_circle(r.x + r.w * t, cy, r.h * 0.28, alpha(col, a));
        if enabled && self.pressed && self.hover(Rect::new(r.x - 8.0, r.y, r.w + 16.0, r.h)) {
            self.active = Some(id);
        }
        if enabled && self.active == Some(id) && self.down {
            let t = ((self.m.x - r.x) / r.w).clamp(0.0, 1.0) as f64;
            *v = min + t * (max - min);
            return true;
        }
        false
    }
    fn checkbox(&mut self, r: Rect, label: &str, v: &mut bool, size: f32) -> bool {
        txt(label, r.x, r.y + r.h / 2.0 + size * 0.3, size, FG);
        let b = r.h * 0.62;
        let br = Rect::new(r.x + r.w - b, r.y + (r.h - b) / 2.0, b, b);
        draw_rectangle(br.x, br.y, br.w, br.h, if *v { pcol(0) } else { BTN });
        draw_rectangle_lines(br.x, br.y, br.w, br.h, 1.0, LINE);
        if *v {
            let (x, y, s) = (br.x, br.y, br.w);
            draw_line(x + s * 0.22, y + s * 0.52, x + s * 0.42, y + s * 0.72, 2.5, BG);
            draw_line(x + s * 0.42, y + s * 0.72, x + s * 0.8, y + s * 0.28, 2.5, BG);
        }
        if self.pressed && self.hover(r) {
            *v = !*v;
            return true;
        }
        false
    }
    fn choice(&mut self, r: Rect, label: &str, opts: &[&str], idx: &mut usize, size: f32, enabled: bool) -> bool {
        let a = if enabled { 1.0 } else { 0.4 };
        txt(label, r.x, r.y + r.h / 2.0 + size * 0.3, size, alpha(FG, a));
        let bw = r.w * 0.5;
        let br = Rect::new(r.x + r.w - bw, r.y + r.h * 0.1, bw, r.h * 0.8);
        draw_rectangle(br.x, br.y, br.w, br.h, alpha(BTN, a));
        draw_rectangle_lines(br.x, br.y, br.w, br.h, 1.0, LINE);
        txt_c("<", br.x + size * 0.6, br.y + br.h / 2.0, size, alpha(MUTED, a));
        txt_c(">", br.x + br.w - size * 0.6, br.y + br.h / 2.0, size, alpha(MUTED, a));
        txt_c(opts[*idx], br.x + br.w / 2.0, br.y + br.h / 2.0, size, alpha(FG, a));
        if enabled && self.pressed && self.hover(br) {
            if self.m.x < br.x + br.w / 2.0 {
                *idx = (*idx + opts.len() - 1) % opts.len();
            } else {
                *idx = (*idx + 1) % opts.len();
            }
            return true;
        }
        false
    }
}

// ---------- Spiel ----------
struct Game {
    set: Settings,
    scores: [u32; 2],
    turn: usize,
    starter: usize,
    players: [Player; 2],
    ships: Vec<Ship>,
    bodies: Vec<Body>,
    stars: Vec<Star>,
    nebula: Vec<(f32, f32, f32, f32)>,
    shot: Option<Shot>,
    phase: Phase,
    prev: Phase,
    particles: Vec<Particle>,
    ai_turns: i32,
    ai: Option<AiAnim>,
    ai_delay: f32,
    ai_fire: Option<f32>,
    acc: f64,
    toast: String,
    toast_t: f32,
    over_t: f32,
    over_show: bool,
    over_title: String,
    over_color: Color,
    over_sub: String,
    audio: Audio,
    ui: Ui,
    s: f32,
    ox: f32,
    oy: f32,
    k: f32,
    panel_h: f32,
    quit: bool,
    keyrep: [f32; 4],
    has_game: bool,
    aiming: bool,
    time: f32,
}

impl Game {
    fn new(audio: Audio) -> Self {
        let mut g = Game {
            set: Settings {
                cpu: false,
                ai_lvl: 1,
                planets: 6,
                sun: true,
                holes: 1,
                holes_hidden: true,
                endless: false,
                fly_sound: true,
                sound: true,
                trails: true,
            },
            scores: [0, 0],
            turn: 0,
            starter: 0,
            players: [
                Player { angle: 0.0, power: 55.0, trails: vec![] },
                Player { angle: 180.0, power: 55.0, trails: vec![] },
            ],
            ships: vec![],
            bodies: vec![],
            stars: vec![],
            nebula: vec![],
            shot: None,
            phase: Phase::Menu,
            prev: Phase::Aim,
            particles: vec![],
            ai_turns: 0,
            ai: None,
            ai_delay: 0.0,
            ai_fire: None,
            acc: 0.0,
            toast: String::new(),
            toast_t: 0.0,
            over_t: 0.0,
            over_show: false,
            over_title: String::new(),
            over_color: FG,
            over_sub: String::new(),
            audio,
            ui: Ui { m: Vec2::ZERO, pressed: false, down: false, active: None },
            s: 1.0,
            ox: 0.0,
            oy: 0.0,
            k: 1.0,
            panel_h: 60.0,
            quit: false,
            keyrep: [0.0; 4],
            has_game: false,
            aiming: false,
            time: 0.0,
        };
        g.make_sky();
        g
    }

    fn names(&self) -> [&'static str; 2] {
        if self.set.cpu {
            ["Du", "Computer"]
        } else {
            ["Spieler 1", "Spieler 2"]
        }
    }
    fn human_aim(&self) -> bool {
        self.phase == Phase::Aim && !(self.set.cpu && self.turn == 1)
    }
    fn toast(&mut self, s: &str) {
        self.toast = s.to_string();
        self.toast_t = 1.8;
    }
    fn sync_audio(&mut self) {
        self.audio.muted = !self.set.sound;
        self.audio.fly_enabled = self.set.fly_sound;
        if self.audio.muted || !self.audio.fly_enabled {
            self.audio.fly_stop();
        } else if self.phase == Phase::Flying {
            self.audio.fly_start();
        }
    }

    fn make_sky(&mut self) {
        self.stars = (0..300)
            .map(|_| Star { x: rndf(0.0, 1.0), y: rndf(0.0, 1.0), r: rndf(0.3, 1.4), a: rndf(0.25, 1.0) })
            .collect();
        self.nebula = (0..3).map(|_| (rndf(0.0, 1.0), rndf(0.0, 1.0), rndf(0.25, 0.5), rndf(180.0, 320.0))).collect();
    }

    // ----- Level -----
    fn place(&self, r: f64, x0: f64, x1: f64, y0: f64, y1: f64, gap: f64, ship_gap: f64) -> Option<(f64, f64)> {
        for _ in 0..3000 {
            let x = rnd(x0, x1);
            let lo = y0.max(r + 25.0);
            let hi = y1.min(H - r - 25.0);
            if hi <= lo {
                return None;
            }
            let y = rnd(lo, hi);
            if self.bodies.iter().any(|p| {
                dist(p.x, p.y, x, y) < p.r + r + gap + if p.kind != Kind::Planet { 60.0 } else { 0.0 }
            }) {
                continue;
            }
            if self.ships.iter().any(|s| dist(s.x, s.y, x, y) < r + ship_gap) {
                continue;
            }
            return Some((x, y));
        }
        None
    }

    fn new_level(&mut self) {
        self.ships = vec![
            Ship { x: rnd(90.0, 320.0), y: rnd(140.0, H - 140.0), dead: false },
            Ship { x: rnd(W - 320.0, W - 90.0), y: rnd(140.0, H - 140.0), dead: false },
        ];
        self.bodies.clear();
        if self.set.sun {
            let r = rnd(70.0, 88.0);
            if let Some((x, y)) = self.place(r, W / 2.0 - 120.0, W / 2.0 + 120.0, H / 2.0 - 160.0, H / 2.0 + 160.0, 0.0, 200.0) {
                self.bodies.push(Body::new(Kind::Sun, x, y, r, r * r * r * 1.5));
            }
        }
        for _ in 0..self.set.holes {
            if let Some((x, y)) = self.place(14.0, 340.0, W - 340.0, 0.0, H, 90.0, 220.0) {
                let mut b = Body::new(Kind::Hole, x, y, 14.0, 62f64.powi(3) * 1.7);
                b.revealed = !self.set.holes_hidden;
                b.reveal = if b.revealed { 1.0 } else { 0.0 };
                self.bodies.push(b);
            }
        }
        for _ in 0..self.set.planets {
            let r = rnd(22.0, 70.0);
            if let Some((x, y)) = self.place(r, 260.0, W - 260.0, 0.0, H, 35.0, 90.0) {
                let dens = rnd(0.7, 1.35);
                let mut b = Body::new(Kind::Planet, x, y, r, r * r * r * dens);
                b.dens = dens as f32;
                self.bodies.push(b);
            }
        }
        self.make_sky();
        self.players[0].angle = 0.0;
        self.players[1].angle = 180.0;
        for p in self.players.iter_mut() {
            p.power = 55.0;
            p.trails.clear();
        }
        self.audio.fly_stop();
        self.shot = None;
        self.particles.clear();
        self.ai_turns = 0;
        self.ai = None;
        self.ai_fire = None;
        self.turn = self.starter;
        self.phase = Phase::Aim;
        self.over_show = false;
        self.has_game = true;
        self.maybe_ai();
    }

    // ----- Schießen -----
    fn fire(&mut self) {
        if !matches!(self.phase, Phase::Aim | Phase::Ai) {
            return;
        }
        let p = &self.players[self.turn];
        let mut s = launch(&self.ships[self.turn], self.turn, p.angle, p.power);
        s.pts.push((s.x, s.y));
        self.shot = Some(s);
        self.phase = Phase::Flying;
        self.acc = 0.0;
        self.audio.play(Sfx::Fire);
        self.audio.fly_start();
    }

    fn end_shot(&mut self) {
        if self.phase == Phase::Flying {
            if let Some(s) = &self.shot {
                if s.t >= 0.4 {
                    self.resolve(Hit::Abort);
                }
            }
        }
    }

    fn resolve(&mut self, hit: Hit) {
        let Some(mut o) = self.shot.take() else { return };
        self.audio.fly_stop();
        o.pts.push((o.x, o.y));
        let owner = o.owner;
        let (ox, oy) = (o.x, o.y);
        if self.set.trails {
            let tr = &mut self.players[owner].trails;
            tr.push(o.pts);
            if tr.len() > 4 {
                tr.remove(0);
            }
        }
        let n = self.names();
        match hit {
            Hit::Ship(v) => {
                self.ships[v].dead = true;
                let (sx, sy) = (self.ships[v].x, self.ships[v].y);
                self.explode(sx, sy, pcol(v), 110, true);
                self.audio.play(Sfx::Boom);
                self.audio.play(Sfx::Win);
                let winner = if v == owner { 1 - owner } else { owner };
                self.scores[winner] += 1;
                self.phase = Phase::Over;
                self.starter = 1 - self.starter;
                self.over_t = 1.1;
                self.over_title = if v == owner {
                    format!("Eigentreffer! Punkt für {}", n[winner])
                } else if self.set.cpu && owner == 0 {
                    "Du triffst!".to_string()
                } else {
                    format!("{} trifft!", n[owner])
                };
                self.over_color = pcol(winner);
                self.over_sub = format!("Stand {} : {}   ·   Leertaste: nächste Runde", self.scores[0], self.scores[1]);
                return;
            }
            Hit::Body(Kind::Planet, _) => {
                self.explode(ox, oy, Color::new(1.0, 0.82, 0.48, 1.0), 22, false);
                self.audio.play(Sfx::Planet);
                self.toast("Einschlag auf Planet");
            }
            Hit::Body(Kind::Sun, _) => {
                self.explode(ox, oy, Color::new(1.0, 0.7, 0.28, 1.0), 30, false);
                self.audio.play(Sfx::Sun);
                self.toast("In der Sonne verglüht");
            }
            Hit::Body(Kind::Hole, i) => {
                self.implode(i);
                self.bodies[i].revealed = true;
                self.audio.play(Sfx::Hole);
                self.toast("Vom Schwarzen Loch verschluckt");
            }
            Hit::Out => {
                self.audio.play(Sfx::Lost);
                self.toast("Im All verschollen");
            }
            Hit::Time => {
                self.audio.play(Sfx::Lost);
                self.toast("Schuss verglüht im Orbit");
            }
            Hit::Abort => {
                self.audio.play(Sfx::Lost);
                self.toast("Schuss beendet");
            }
        }
        self.turn = 1 - self.turn;
        self.phase = Phase::Aim;
        self.maybe_ai();
    }

    fn explode(&mut self, x: f64, y: f64, c: Color, n: usize, big: bool) {
        for _ in 0..n {
            let a = rnd(0.0, 7.0);
            let v = rnd(20.0, if big { 260.0 } else { 120.0 });
            let l = rndf(0.4, if big { 1.6 } else { 0.8 });
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * v,
                vy: a.sin() * v,
                life: l,
                max: l,
                c: if rndf(0.0, 1.0) < 0.3 { WHITE } else { c },
                r: rndf(1.0, if big { 3.0 } else { 2.0 }),
                ring: None,
                drag: true,
            });
        }
        let l = if big { 0.8 } else { 0.4 };
        self.particles.push(Particle {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            life: l,
            max: l,
            c,
            r: 0.0,
            ring: Some((if big { 90.0 } else { 30.0 }, false)),
            drag: false,
        });
    }

    fn implode(&mut self, i: usize) {
        let (px, py) = (self.bodies[i].x, self.bodies[i].y);
        for _ in 0..40 {
            let a = rnd(0.0, 7.0);
            let d = rnd(40.0, 90.0);
            let l = rnd(0.5, 0.9);
            let sp = d / l;
            self.particles.push(Particle {
                x: px + a.cos() * d,
                y: py + a.sin() * d,
                vx: -a.cos() * sp + a.sin() * sp * 0.6,
                vy: -a.sin() * sp - a.cos() * sp * 0.6,
                life: l as f32,
                max: l as f32,
                c: if rndf(0.0, 1.0) < 0.5 { Color::new(0.79, 0.64, 1.0, 1.0) } else { Color::new(1.0, 0.85, 0.63, 1.0) },
                r: rndf(1.0, 2.2),
                ring: None,
                drag: false,
            });
        }
        self.particles.push(Particle {
            x: px,
            y: py,
            vx: 0.0,
            vy: 0.0,
            life: 0.7,
            max: 0.7,
            c: Color::new(0.7, 0.55, 1.0, 1.0),
            r: 0.0,
            ring: Some((80.0, true)),
            drag: false,
        });
    }

    // ----- Computer -----
    fn ai_eval(&self, a: f64, p: f64, dt: f64) -> f64 {
        let mut o = launch(&self.ships[1], 1, a, p);
        let tg = &self.ships[0];
        let mut min = 1e9f64;
        let steps = (MAX_T * 0.7 / dt) as usize;
        for _ in 0..steps {
            // der Computer kennt versteckte Schwarze Löcher auch nicht
            let res = step_shot(&mut o, dt, &self.bodies, &self.ships, true, true);
            min = min.min(dist(o.x, o.y, tg.x, tg.y));
            if let Some(h) = res {
                if let Hit::Ship(i) = h {
                    return if i == 0 { 0.0 } else { 1e6 };
                }
                break;
            }
        }
        min
    }

    fn ai_plan(&mut self) -> (f64, f64) {
        let mut best = (180.0, 55.0, 1e9);
        for _ in 0..380 {
            let (a, p) = (rnd(0.0, 360.0), rnd(25.0, 100.0));
            let s = self.ai_eval(a, p, 1.0 / 120.0);
            if s < best.2 {
                best = (a, p, s);
            }
        }
        best.2 = self.ai_eval(best.0, best.1, DT);
        for k in 0..160 {
            let sp = 10.0 * (1.0 - k as f64 / 160.0) + 0.3;
            let a = best.0 + gauss() * sp;
            let p = (best.1 + gauss() * sp).clamp(5.0, 100.0);
            let s = self.ai_eval(a, p, DT);
            if s < best.2 {
                best = (a, p, s);
            }
            if best.2 == 0.0 {
                break;
            }
        }
        let base = [12.0, 5.0, 1.5][self.set.ai_lvl];
        let f = 0.72f64.powi(self.ai_turns);
        self.ai_turns += 1;
        (wrap360(best.0 + gauss() * base * f), (best.1 + gauss() * base * 1.2 * f).clamp(5.0, 100.0))
    }

    fn maybe_ai(&mut self) {
        if self.set.cpu && self.turn == 1 && self.phase == Phase::Aim {
            self.phase = Phase::Ai;
            self.ai = None;
            self.ai_fire = None;
            self.ai_delay = 0.35;
        }
    }

    // ----- Eingabe -----
    fn key_rep(&mut self, i: usize, k: KeyCode, dt: f32) -> bool {
        if is_key_pressed(k) {
            self.keyrep[i] = 0.0;
            return true;
        }
        if is_key_down(k) {
            self.keyrep[i] += dt;
            if self.keyrep[i] > 0.35 {
                self.keyrep[i] -= 0.03;
                return true;
            }
        }
        false
    }

    fn set_angle(&mut self, a: f64) {
        if self.human_aim() {
            self.players[self.turn].angle = (wrap360(a) * 10.0).round() / 10.0 % 360.0;
        }
    }
    fn set_power(&mut self, p: f64) {
        if self.human_aim() {
            self.players[self.turn].power = ((p.clamp(0.0, 100.0)) * 10.0).round() / 10.0;
        }
    }

    fn show_menu(&mut self) {
        if self.phase != Phase::Menu {
            self.prev = self.phase;
            self.phase = Phase::Menu;
            self.audio.fly_stop();
        }
    }
    fn resume(&mut self) {
        self.phase = self.prev;
        if self.phase == Phase::Ai && self.ai.is_none() {
            self.ai_delay = 0.2;
        }
        if self.phase == Phase::Flying {
            self.audio.fly_start();
        }
    }
    fn start_game(&mut self) {
        self.scores = [0, 0];
        self.starter = 0;
        self.sync_audio();
        self.new_level();
    }

    fn keys(&mut self, dt: f32) {
        if is_key_pressed(KeyCode::M) {
            self.set.sound = !self.set.sound;
            self.sync_audio();
            let t = if self.set.sound { "Ton an" } else { "Ton aus" };
            self.toast(t);
        }
        if is_key_pressed(KeyCode::F) {
            self.set.fly_sound = !self.set.fly_sound;
            self.sync_audio();
            let t = if self.set.fly_sound { "Fluggeräusch an" } else { "Fluggeräusch aus" };
            self.toast(t);
        }
        if self.phase == Phase::Menu {
            if is_key_pressed(KeyCode::Escape) && self.has_game {
                self.resume();
            }
            return;
        }
        if is_key_pressed(KeyCode::Escape) {
            self.show_menu();
            return;
        }
        let go = is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter);
        if self.phase == Phase::Over {
            if self.over_show && go {
                self.new_level();
            }
            return;
        }
        if self.phase == Phase::Flying {
            if go || is_key_pressed(KeyCode::Backspace) {
                self.end_shot();
            }
            return;
        }
        if is_key_pressed(KeyCode::N) && self.phase == Phase::Aim {
            self.new_level();
            self.toast("Neue Karte");
            return;
        }
        if !self.human_aim() {
            return;
        }
        let f = if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) { 0.1 } else { 1.0 };
        let (a, p) = (self.players[self.turn].angle, self.players[self.turn].power);
        if self.key_rep(0, KeyCode::Left, dt) {
            self.set_angle(a + f);
        }
        if self.key_rep(1, KeyCode::Right, dt) {
            self.set_angle(a - f);
        }
        if self.key_rep(2, KeyCode::Up, dt) {
            self.set_power(p + f);
        }
        if self.key_rep(3, KeyCode::Down, dt) {
            self.set_power(p - f);
        }
        if go {
            self.fire();
        }
    }

    // ----- Simulation -----
    fn update(&mut self, dt: f32) {
        self.time += dt;
        self.toast_t -= dt;
        if self.phase == Phase::Menu {
            return;
        }
        match self.phase {
            Phase::Flying => {
                self.acc += dt as f64 * SIM_SPEED;
                while self.acc >= DT {
                    self.acc -= DT;
                    let Some(s) = self.shot.as_mut() else { break };
                    let res = step_shot(s, DT, &self.bodies, &self.ships, !self.set.endless, false);
                    s.n += 1;
                    if s.n % 2 == 0 {
                        s.pts.push((s.x, s.y));
                        if s.pts.len() > 5000 {
                            // lange Flüge ausdünnen
                            s.pts = s.pts.iter().step_by(2).copied().collect();
                        }
                    }
                    let (sx, sy) = (s.x, s.y);
                    let mut revealed = false;
                    for b in self.bodies.iter_mut() {
                        if b.kind == Kind::Hole && !b.revealed && dist(b.x, b.y, sx, sy) < 220.0 {
                            b.revealed = true;
                            revealed = true;
                        }
                    }
                    if revealed {
                        self.audio.play(Sfx::Reveal);
                        self.toast("Achtung: ein Schwarzes Loch!");
                    }
                    if let Some(h) = res {
                        self.resolve(h);
                        break;
                    }
                }
                if let Some(s) = &self.shot {
                    let sp = (s.vx * s.vx + s.vy * s.vy).sqrt();
                    self.audio.fly_update(sp, dt);
                }
            }
            Phase::Ai => {
                if let Some(d) = self.ai_fire.as_mut() {
                    *d -= dt;
                    if *d <= 0.0 {
                        self.ai_fire = None;
                        self.fire();
                    }
                } else if let Some(an) = self.ai.as_mut() {
                    an.t = (an.t + dt / 0.9).min(1.0);
                    let t = an.t;
                    let e = if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 } as f64;
                    self.players[1].angle = wrap360(an.a0 + an.da * e);
                    self.players[1].power = an.p0 + an.dp * e;
                    if an.t >= 1.0 {
                        self.ai = None;
                        self.ai_fire = Some(0.25);
                    }
                } else {
                    self.ai_delay -= dt;
                    if self.ai_delay <= 0.0 {
                        let (a, p) = self.ai_plan();
                        let pl = &self.players[1];
                        let da = (a - pl.angle + 540.0).rem_euclid(360.0) - 180.0;
                        self.ai = Some(AiAnim { a0: pl.angle, da, p0: pl.power, dp: p - pl.power, t: 0.0 });
                    }
                }
            }
            Phase::Over => {
                self.over_t -= dt;
                if self.over_t <= 0.0 {
                    self.over_show = true;
                }
            }
            _ => {}
        }
        for b in self.bodies.iter_mut() {
            if b.kind == Kind::Hole && b.revealed && b.reveal < 1.0 {
                b.reveal = (b.reveal + dt * 1.2).min(1.0);
            }
        }
        for p in self.particles.iter_mut() {
            p.life -= dt;
            if p.ring.is_none() {
                p.x += p.vx * dt as f64;
                p.y += p.vy * dt as f64;
                if p.drag {
                    p.vx *= 0.985;
                    p.vy *= 0.985;
                }
            }
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    // ----- Darstellung -----
    fn ws(&self, x: f64, y: f64) -> Vec2 {
        vec2(self.ox + x as f32 * self.s, self.oy + y as f32 * self.s)
    }
    fn wl(&self, l: f64) -> f32 {
        l as f32 * self.s
    }

    fn panel_items(&self, sw: f32) -> (Vec<Rect>, f32) {
        // Elemente: Name, Winkel-Gruppe, Geschwindigkeit-Gruppe, Feuer, Ton, Menü
        let k = self.k;
        let fs = 15.0 * k;
        let gap = 18.0 * k;
        let fixed = 110.0 * k + tw("Winkel", fs) + tw("Geschwindigkeit", fs) + 180.0 * k + 160.0 * k + 44.0 * k + 80.0 * k + gap * 5.0;
        let slider = ((sw - 24.0 * k - fixed) / 2.0).clamp(100.0 * k, 220.0 * k);
        let widths = [
            110.0 * k,
            tw("Winkel", fs) + slider + 90.0 * k,
            tw("Geschwindigkeit", fs) + slider + 90.0 * k,
            160.0 * k,
            44.0 * k,
            80.0 * k,
        ];
        let rh = 40.0 * k;
        let pad = 9.0 * k;
        let mut rows: Vec<Vec<usize>> = vec![vec![]];
        let mut w = 0.0;
        for (i, iw) in widths.iter().enumerate() {
            let need = if rows.last().unwrap().is_empty() { *iw } else { w + gap + iw };
            if need > sw - 24.0 * k && !rows.last().unwrap().is_empty() {
                rows.push(vec![i]);
                w = *iw;
            } else {
                rows.last_mut().unwrap().push(i);
                w = need;
            }
        }
        let ph = rows.len() as f32 * rh + pad * 2.0;
        let top = screen_height() - ph;
        let mut out = vec![Rect::default(); widths.len()];
        for (r, row) in rows.iter().enumerate() {
            let total: f32 = row.iter().map(|i| widths[*i]).sum::<f32>() + gap * (row.len() as f32 - 1.0);
            let mut x = (sw - total) / 2.0;
            for i in row {
                out[*i] = Rect::new(x, top + pad + r as f32 * rh + 3.0 * k, widths[*i], rh - 6.0 * k);
                x += widths[*i] + gap;
            }
        }
        (out, ph)
    }

    fn layout(&mut self) {
        // macroquad rechnet bereits in logischen Punkten (Retina wird automatisch skaliert)
        self.k = 1.0;
        let (sw, sh) = (screen_width(), screen_height());
        let (_, ph) = self.panel_items(sw);
        self.panel_h = ph;
        let top = 34.0 * self.k;
        let aw = sw - 16.0 * self.k;
        let ah = (sh - top - ph - 8.0 * self.k).max(100.0);
        self.s = (aw / W as f32).min(ah / H as f32);
        self.ox = (sw - W as f32 * self.s) / 2.0;
        self.oy = top + (ah - H as f32 * self.s) / 2.0 + 4.0 * self.k;
    }

    fn draw_sky(&self) {
        let (sw, sh) = (screen_width(), screen_height());
        clear_background(BG);
        for (x, y, r, h) in &self.nebula {
            let c = hsl(*h, 0.7, 0.4);
            let rr = r * sw.max(sh);
            glow(x * sw, y * sh, 0.0, rr, c, 0.12);
        }
        for s in &self.stars {
            draw_circle(s.x * sw, s.y * sh, s.r * self.k, Color::new(1.0, 1.0, 1.0, s.a));
        }
        let a = self.ws(0.0, 0.0);
        draw_rectangle_lines(a.x, a.y, W as f32 * self.s, H as f32 * self.s, 1.0, Color::new(0.47, 0.51, 0.78, 0.12));
    }

    fn draw_planet(&self, p: &Body) {
        let c = self.ws(p.x, p.y);
        let r = self.wl(p.r);
        glow(c.x, c.y, r * 0.95, r * 1.6, hsl(p.hue, 0.8, 0.6), 0.12 + 0.1 * p.dens);
        sphere(c.x, c.y, r, hsl(p.hue, 0.6, 0.12), hsl(p.hue, 0.5, 0.4), hsl(p.hue, 0.7, 0.72));
        if p.banded {
            let (s, co) = p.tilt.sin_cos();
            let th = r * 0.14;
            for i in -3i32..=3 {
                let dy = i as f32 * r * 0.28;
                let hw2 = r * r - (dy.abs() + th / 2.0).powi(2);
                if hw2 <= 0.0 {
                    continue;
                }
                let hw = hw2.sqrt() * 0.97;
                let col = alpha(hsl(p.hue + 30.0, 0.6, if i % 2 != 0 { 0.7 } else { 0.25 }), 0.13);
                let p0 = vec2(c.x + (-hw) * co - dy * s, c.y + (-hw) * s + dy * co);
                let p1 = vec2(c.x + hw * co - dy * s, c.y + hw * s + dy * co);
                draw_line(p0.x, p0.y, p1.x, p1.y, th, col);
            }
        } else {
            for (a, d, sz) in &p.spots {
                let cr = r * sz;
                let dd = (r * d).min(r - cr - 1.0).max(0.0);
                draw_circle(c.x + a.cos() * dd, c.y + a.sin() * dd, cr, Color::new(0.0, 0.0, 0.0, 0.18));
            }
        }
    }

    fn draw_sun(&self, p: &Body) {
        let c = self.ws(p.x, p.y);
        let r = self.wl(p.r);
        let t = self.time;
        let pulse = 1.0 + 0.05 * (t * 2.3).sin();
        glow(c.x, c.y, r * 0.8, r * 2.8 * pulse, Color::new(1.0, 0.55, 0.2, 1.0), 0.45);
        for i in 0..14 {
            let a = t * 0.06 + i as f32 * TAU / 14.0;
            let l = r * (1.45 + 0.3 * (t * 1.7 + i as f32 * 1.9).sin());
            let (s, co) = a.sin_cos();
            let base = r * 0.85;
            let w = r * 0.13;
            let p0 = vec2(c.x + co * base - s * w, c.y + s * base + co * w);
            let p1 = vec2(c.x + co * base + s * w, c.y + s * base - co * w);
            let tip = vec2(c.x + co * l, c.y + s * l);
            draw_triangle(p0, p1, tip, Color::new(1.0, 0.78, 0.43, 0.14));
        }
        sphere(
            c.x,
            c.y,
            r,
            Color::new(0.91, 0.34, 0.11, 1.0),
            Color::new(1.0, 0.6, 0.18, 1.0),
            Color::new(1.0, 0.85, 0.43, 1.0),
        );
        draw_circle(c.x - r * 0.3, c.y - r * 0.3, r * 0.25, Color::new(1.0, 0.98, 0.9, 0.5));
        for i in 0..7 {
            let a = i as f32 * 2.4 + t * 0.08;
            let d = r * (0.25 + 0.5 * ((i * 37) % 10) as f32 / 10.0);
            draw_circle(c.x + a.cos() * d, c.y + a.sin() * d, r * 0.12, Color::new(0.78, 0.27, 0.04, 0.16));
        }
    }

    fn draw_hole(&self, p: &Body) {
        if p.reveal <= 0.0 {
            return;
        }
        let a = p.reveal;
        let c = self.ws(p.x, p.y);
        let r = self.wl(p.r);
        let t = self.time;
        glow(c.x, c.y, r, r * 7.0, Color::new(0.59, 0.39, 1.0, 1.0), 0.22 * a);
        let n = 10;
        for i in 0..n {
            let f = i as f32 / (n - 1) as f32;
            let rx = r * (4.2 - 3.0 * f);
            let col = if f < 0.5 {
                mix(Color::new(0.67, 0.16, 0.5, 0.0), Color::new(1.0, 0.55, 0.24, 0.25), f * 2.0)
            } else {
                mix(Color::new(1.0, 0.55, 0.24, 0.25), Color::new(1.0, 0.9, 0.67, 0.5), (f - 0.5) * 2.0)
            };
            fill_ellipse(c.x, c.y, rx, rx * 0.32, p.tilt, alpha(col, col.a * a));
        }
        for i in 0..6 {
            let rx = r * (1.6 + i as f32 * 0.42);
            let a0 = t * (2.2 - i as f32 * 0.25) + i as f32 * 1.7;
            ellipse_arc(c.x, c.y, rx, rx * 0.32, p.tilt, a0, 1.4, 1.3, Color::new(1.0, 0.94, 0.82, (0.35 - i as f32 * 0.04) * a));
        }
        draw_circle(c.x, c.y, r * 1.15, Color::new(0.0, 0.0, 0.0, a));
        draw_circle_lines(c.x, c.y, r * 1.2, 1.5, Color::new(1.0, 0.88, 0.67, 0.85 * a));
    }

    fn draw_world(&self) {
        let k = self.k;
        // alte Flugbahnen (gestrichelt)
        if self.set.trails {
            for i in 0..2 {
                let tr = &self.players[i].trails;
                for (j, pts) in tr.iter().enumerate() {
                    let a = 0.5 * 0.55f32.powi((tr.len() - 1 - j) as i32);
                    let col = alpha(pcol(i), a);
                    for (n, w) in pts.windows(2).enumerate() {
                        if (n / 3) % 2 == 0 {
                            let p = self.ws(w[0].0, w[0].1);
                            let q = self.ws(w[1].0, w[1].1);
                            draw_line(p.x, p.y, q.x, q.y, 1.4 * k, col);
                        }
                    }
                }
            }
        }
        for b in &self.bodies {
            match b.kind {
                Kind::Planet => self.draw_planet(b),
                Kind::Sun => self.draw_sun(b),
                Kind::Hole => self.draw_hole(b),
            }
        }
        // Schiffe
        for (i, sh) in self.ships.iter().enumerate() {
            if sh.dead {
                continue;
            }
            let pl = &self.players[i];
            let a = pl.angle.to_radians() as f32;
            let c = self.ws(sh.x, sh.y);
            let u = vec2(a.cos(), -a.sin());
            let v = vec2(a.sin(), a.cos());
            let sc = self.s;
            let active = i == self.turn && matches!(self.phase, Phase::Aim | Phase::Ai);
            if active {
                let pulse = (self.time * 4.0).sin();
                draw_circle_lines(c.x, c.y, (SHIP_R as f32 + 9.0 + 2.0 * pulse) * sc, 1.5 * k, alpha(pcol(i), 0.35 + 0.25 * pulse));
                let len = SHIP_R as f32 + 14.0 + pl.power as f32 * 1.1;
                let mut d = SHIP_R as f32 + 6.0;
                while d < len {
                    let e = (d + 6.0).min(len);
                    let p = c + u * d * sc;
                    let q = c + u * e * sc;
                    draw_line(p.x, p.y, q.x, q.y, 1.5 * k, alpha(pcol(i), 0.7));
                    d += 12.0;
                }
            }
            glow(c.x, c.y, 0.0, 22.0 * sc, pcol(i), 0.35);
            let pt = |px: f32, py: f32| c + (u * px + v * py) * sc;
            let (tip, l, notch, r) = (pt(17.0, 0.0), pt(-11.0, -10.0), pt(-5.0, 0.0), pt(-11.0, 10.0));
            draw_triangle(tip, l, notch, pcol(i));
            draw_triangle(tip, notch, r, pcol(i));
            let w = pt(3.0, 0.0);
            draw_circle(w.x, w.y, 2.6 * sc, Color::new(1.0, 1.0, 1.0, 0.85));
        }
        // aktueller Schuss
        if let Some(o) = &self.shot {
            let col = alpha(pcol(o.owner), 0.85);
            let mut prev: Option<Vec2> = None;
            for (x, y) in o.pts.iter().chain(std::iter::once(&(o.x, o.y))) {
                let p = self.ws(*x, *y);
                if let Some(q) = prev {
                    draw_line(q.x, q.y, p.x, p.y, 2.0 * k, col);
                }
                prev = Some(p);
            }
            let p = self.ws(o.x, o.y);
            glow(p.x, p.y, 0.0, 9.0 * k, WHITE, 0.6);
            draw_circle(p.x, p.y, (SHOT_R as f32 * self.s).max(2.0 * k), WHITE);
            // Pfeil am Rand, wenn der Schuss außerhalb des Bildes ist
            let (sw, sh) = (screen_width(), screen_height() - self.panel_h);
            let m = 16.0 * k;
            if p.x < 0.0 || p.x > sw || p.y < 0.0 || p.y > sh {
                let cp = vec2(p.x.clamp(m, sw - m), p.y.clamp(m, sh - m));
                let ang = (p.y - cp.y).atan2(p.x - cp.x);
                let (s, co) = ang.sin_cos();
                let f = |a: f32, b: f32| vec2(cp.x + a * co - b * s, cp.y + a * s + b * co);
                draw_triangle(f(10.0 * k, 0.0), f(-7.0 * k, -7.0 * k), f(-7.0 * k, 7.0 * k), pcol(o.owner));
                let d = dist(o.x, o.y, W / 2.0, H / 2.0) / 1000.0;
                let label = format!("{:.1}k", d);
                txt_c(&label, cp.x - co * 26.0 * k, cp.y - s * 18.0 * k, 13.0 * k, alpha(pcol(o.owner), 0.9));
            }
        }
        // Partikel
        for p in &self.particles {
            let f = p.life / p.max;
            let c = self.ws(p.x, p.y);
            match p.ring {
                Some((rr, inward)) => {
                    let rad = if inward { rr * f + 1.0 } else { rr * (1.0 - f) + 2.0 };
                    draw_circle_lines(c.x, c.y, rad * self.s, 2.0 * k, alpha(p.c, f));
                }
                None => draw_circle(c.x, c.y, p.r * self.s.max(0.6), alpha(p.c, f)),
            }
        }
    }

    fn draw_hud(&mut self) {
        let k = self.k;
        let sw = screen_width();
        let n = self.names();
        let fs = 17.0 * k;
        txt(&format!("{} · {}", n[0], self.scores[0]), 16.0 * k, 24.0 * k, fs, pcol(0));
        let s2 = format!("{} · {}", self.scores[1], n[1]);
        txt(&s2, sw - 16.0 * k - tw(&s2, fs), 24.0 * k, fs, pcol(1));
        let st = match self.phase {
            Phase::Aim => format!("{} {} dran", n[self.turn], if self.set.cpu && self.turn == 0 { "bist" } else { "ist" }),
            Phase::Ai => "Computer zielt …".to_string(),
            Phase::Flying => {
                let t = self.shot.as_ref().map(|s| s.t).unwrap_or(0.0);
                format!("Schuss unterwegs ({:.0} s) · Leertaste = beenden", t)
            }
            Phase::Over => "Treffer!".to_string(),
            Phase::Menu => String::new(),
        };
        txt_c(&st, sw / 2.0, 19.0 * k, 14.0 * k, MUTED);
        if self.toast_t > 0.0 {
            let a = self.toast_t.min(0.3) / 0.3;
            let fs = 15.0 * k;
            let w = tw(&self.toast, fs) + 28.0 * k;
            let r = Rect::new(sw / 2.0 - w / 2.0, 36.0 * k, w, 28.0 * k);
            draw_rectangle(r.x, r.y, r.w, r.h, alpha(PANEL, 0.95 * a));
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, alpha(LINE, a));
            txt_c(&self.toast.clone(), sw / 2.0, r.y + r.h / 2.0, fs, alpha(FG, a));
        }
    }

    fn draw_panel(&mut self) {
        let k = self.k;
        let sw = screen_width();
        let sh = screen_height();
        let (items, ph) = self.panel_items(sw);
        let col = match (&self.shot, self.phase) {
            (Some(s), Phase::Flying) => pcol(s.owner),
            _ => pcol(self.turn),
        };
        draw_rectangle(0.0, sh - ph, sw, ph, PANEL);
        draw_rectangle(0.0, sh - ph, sw, 2.0 * k, col);
        let fs = 15.0 * k;
        let n = self.names();
        let t = self.turn;
        txt(n[t], items[0].x, items[0].y + items[0].h / 2.0 + fs * 0.3, 17.0 * k, pcol(t));
        let enabled = self.human_aim();
        // Winkel / Geschwindigkeit
        for (idx, label, id) in [(1usize, "Winkel", 1u32), (2, "Geschwindigkeit", 2)] {
            let r = items[idx];
            txt(label, r.x, r.y + r.h / 2.0 + fs * 0.3, fs, MUTED);
            let lx = r.x + tw(label, fs) + 10.0 * k;
            let sr = Rect::new(lx, r.y, r.w - (lx - r.x) - 76.0 * k, r.h);
            let mut v = if id == 1 { self.players[t].angle } else { self.players[t].power };
            let max = if id == 1 { 359.9 } else { 100.0 };
            if self.ui.slider(id, sr, &mut v, 0.0, max, pcol(t), enabled) {
                if id == 1 {
                    self.set_angle(v);
                } else {
                    self.set_power(v);
                }
            }
            let val = if id == 1 { self.players[t].angle } else { self.players[t].power };
            let vr = Rect::new(r.x + r.w - 66.0 * k, r.y + 4.0 * k, 66.0 * k, r.h - 8.0 * k);
            draw_rectangle(vr.x, vr.y, vr.w, vr.h, Color::new(0.04, 0.055, 0.12, 1.0));
            draw_rectangle_lines(vr.x, vr.y, vr.w, vr.h, 1.0, LINE);
            let s = if id == 1 { format!("{:.1}°", val) } else { format!("{:.1}", val) };
            txt_c(&s, vr.x + vr.w / 2.0, vr.y + vr.h / 2.0, fs, alpha(FG, if enabled { 1.0 } else { 0.5 }));
        }
        let flying = self.phase == Phase::Flying;
        let label = if flying { "Schuss beenden" } else { "Feuer!" };
        let fire_on = enabled || flying;
        if self.ui.button(items[3], label, col, BG, 16.0 * k, fire_on) {
            if flying {
                self.end_shot();
            } else {
                self.fire();
            }
        }
        let snd = if self.set.sound { "Ton" } else { "Aus" };
        if self.ui.button(items[4], snd, BTN, FG, 13.0 * k, true) {
            self.set.sound = !self.set.sound;
            self.sync_audio();
        }
        if self.ui.button(items[5], "Menü", BTN, FG, 15.0 * k, true) {
            self.show_menu();
        }
    }

    fn draw_menu(&mut self) {
        let (sw, sh) = (screen_width(), screen_height());
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.01, 0.015, 0.04, 0.75));
        let k = self.k.min(sh / 670.0).min(sw / 540.0);
        let cw = 500.0 * k;
        let ch = 655.0 * k;
        let x = (sw - cw) / 2.0;
        let y = ((sh - ch) / 2.0).max(4.0);
        draw_rectangle(x, y, cw, ch, PANEL);
        draw_rectangle_lines(x, y, cw, ch, 1.0, LINE);
        let fs = 16.0 * k;
        let t1 = "GALAXY ";
        let tsz = 40.0 * k;
        let tot = tw("GALAXY WARS 2026", tsz);
        txt(t1, sw / 2.0 - tot / 2.0, y + 52.0 * k, tsz, pcol(0));
        txt("WARS 2026", sw / 2.0 - tot / 2.0 + tw(t1, tsz), y + 52.0 * k, tsz, pcol(1));
        txt_c("Artillerie zwischen Planeten", sw / 2.0, y + 76.0 * k, 14.0 * k, MUTED);

        let px = x + 28.0 * k;
        let pw = cw - 56.0 * k;
        let rh = 32.0 * k;
        let mut ry = y + 96.0 * k;
        let mut row = || {
            let r = Rect::new(px, ry, pw, rh);
            ry += rh + 3.0 * k;
            r
        };
        let r_mode = row();
        let r_ai = row();
        let r_pl = row();
        let r_sun = row();
        let r_bh = row();
        let r_bhh = row();
        let r_end = row();
        let r_fly = row();
        let r_snd = row();
        let r_tr = row();
        let by = ry + 10.0 * k;

        let mut mode = self.set.cpu as usize;
        if self.ui.choice(r_mode, "Modus", &["2 Spieler", "Gegen Computer"], &mut mode, fs, true) {
            self.set.cpu = mode == 1;
        }
        let cpu = self.set.cpu;
        self.ui.choice(r_ai, "Computer", &["Leicht", "Mittel", "Schwer"], &mut self.set.ai_lvl, fs, cpu);
        let mut np = self.set.planets - 1;
        if self.ui.choice(r_pl, "Planeten", &["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"], &mut np, fs, true) {
            self.set.planets = np + 1;
        }
        self.ui.checkbox(r_sun, "Sonne in der Mitte", &mut self.set.sun, fs);
        self.ui.choice(r_bh, "Schwarze Löcher", &["Keine", "1", "2"], &mut self.set.holes, fs, true);
        self.ui.checkbox(r_bhh, "Schwarze Löcher versteckt", &mut self.set.holes_hidden, fs);
        self.ui.checkbox(r_end, "Schüsse fliegen endlos (kein Zeitlimit)", &mut self.set.endless, fs);
        if self.ui.checkbox(r_fly, "Fluggeräusch", &mut self.set.fly_sound, fs) {
            self.sync_audio();
        }
        if self.ui.checkbox(r_snd, "Ton", &mut self.set.sound, fs) {
            self.sync_audio();
        }
        self.ui.checkbox(r_tr, "Alte Flugbahnen zeigen", &mut self.set.trails, fs);

        let bh = 40.0 * k;
        let gap = 10.0 * k;
        let bw = (pw - 2.0 * gap) / 3.0;
        if self.ui.button(Rect::new(px, by, bw, bh), "Fortsetzen", BTN, FG, fs, self.has_game) {
            self.resume();
        }
        if self.ui.button(Rect::new(px + bw + gap, by, bw, bh), "Neues Spiel", pcol(0), BG, fs, true) {
            self.start_game();
        }
        if self.ui.button(Rect::new(px + 2.0 * (bw + gap), by, bw, bh), "Beenden", BTN, FG, fs, true) {
            self.quit = true;
        }
        let help = [
            "Triff das gegnerische Schiff. Planeten, Sonne und Schwarze",
            "Löcher lenken die Schüsse durch ihre Schwerkraft ab.",
            "Pfeil links/rechts: Winkel   ·   Pfeil hoch/runter: Geschwindigkeit",
            "Shift: fein   ·   Maus auf dem Spielfeld halten: zielen",
            "Leertaste: Feuer / Schuss beenden   ·   N: neue Karte",
            "M: Ton   ·   F: Fluggeräusch   ·   Esc: Menü",
        ];
        let mut hy = by + bh + 26.0 * k;
        draw_line(px, hy - 12.0 * k, px + pw, hy - 12.0 * k, 1.0, LINE);
        for l in help {
            txt(l, px, hy + 4.0 * k, 13.0 * k, MUTED);
            hy += 19.0 * k;
        }
    }

    fn draw_over(&mut self) {
        let (sw, sh) = (screen_width(), screen_height());
        let k = self.k;
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.01, 0.015, 0.04, 0.6));
        let cw = (460.0 * k).min(sw - 32.0);
        let ch = 170.0 * k;
        let (x, y) = ((sw - cw) / 2.0, (sh - ch) / 2.0);
        draw_rectangle(x, y, cw, ch, PANEL);
        draw_rectangle_lines(x, y, cw, ch, 1.0, LINE);
        txt_c(&self.over_title.clone(), sw / 2.0, y + 40.0 * k, 26.0 * k, self.over_color);
        txt_c(&self.over_sub.clone(), sw / 2.0, y + 78.0 * k, 14.0 * k, MUTED);
        if self.ui.button(Rect::new(x + 28.0 * k, y + 106.0 * k, cw - 56.0 * k, 42.0 * k), "Nächste Runde", BTN, FG, 16.0 * k, true) {
            self.new_level();
        }
    }

    fn frame(&mut self) {
        let dt = get_frame_time().min(0.05);
        self.layout();
        self.ui.begin();
        self.keys(dt);

        // Zielen mit der Maus auf dem Spielfeld
        let field_h = screen_height() - self.panel_h;
        if self.ui.pressed && self.ui.m.y < field_h && self.phase != Phase::Menu {
            self.aiming = true;
        }
        if !self.ui.down {
            self.aiming = false;
        }
        if self.aiming && self.ui.active.is_none() && self.human_aim() {
            let sh = &self.ships[self.turn];
            let wx = ((self.ui.m.x - self.ox) / self.s) as f64;
            let wy = ((self.ui.m.y - self.oy) / self.s) as f64;
            let a = (-(wy - sh.y)).atan2(wx - sh.x).to_degrees();
            self.set_angle(a);
        }

        self.update(dt);

        self.draw_sky();
        if self.has_game {
            self.draw_world();
        }
        self.draw_hud();
        self.draw_panel();
        if self.phase == Phase::Over && self.over_show {
            self.draw_over();
        }
        if self.phase == Phase::Menu {
            self.draw_menu();
        }
    }
}

fn conf() -> Conf {
    Conf {
        window_title: "Galaxy Wars 2026".to_owned(),
        window_width: 1280,
        window_height: 820,
        high_dpi: true,
        sample_count: 4,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    rand::srand((macroquad::miniquad::date::now() * 1000.0) as u64);
    let audio = Audio::new().await;
    let mut g = Game::new(audio);
    loop {
        g.frame();
        if g.quit {
            break;
        }
        next_frame().await;
    }
}
