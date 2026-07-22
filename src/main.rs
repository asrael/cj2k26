mod anim;
mod bullet;
mod color;
mod enemy;
mod gfx;
mod math;
mod player;
mod rng;
mod sfx;
mod sprite;
mod starfield;

use anim::Anim;
use bullet::Bullet;
use color::Palette;
use color::db32::{BLACK, CYAN, LIGHT_RED, LIME, RED, WHITE, YELLOW};
use enemy::Enemy;
use math::aabb;
use player::Player;
use rng::Rng;
use sfx::Sfx;
use sprite::Sprite;
use starfield::Starfield;

use std::f32::consts::TAU;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use aseprite::AsepriteFile;
use embedded_graphics::mono_font::ascii::{FONT_6X10, FONT_10X20};
use glam::{IVec2, Vec2};
use softbuffer::{Context, Surface};
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const MAX_FRAME: Duration = Duration::from_millis(100);
const TICK: Duration = Duration::from_nanos(16_666_667);

const PLAYER_MOVES: [(Action, IVec2); 4] = [
    (Action::Up, IVec2::new(0, -1)),
    (Action::Down, IVec2::new(0, 1)),
    (Action::Left, IVec2::new(-1, 0)),
    (Action::Right, IVec2::new(1, 0)),
];

pub(crate) const GAME_W: u32 = 240;
pub(crate) const GAME_H: u32 = 160;

const SPRITES: &[u8] = include_bytes!("../assets/sprites.aseprite");

const DONE_HOLD: u32 = 120;

const WIN_CYCLE: [u8; 4] = [LIME, WHITE, CYAN, YELLOW];
const OVER_CYCLE: [u8; 2] = [RED, LIGHT_RED];

#[derive(Default)]
struct Cj2k26 {
    accumulator: Duration,
    anims: Vec<Anim>,
    cursor: Option<Vec2>,
    difficulty: f32,
    dive_timer: u32,
    done_timer: u32,
    enemies: Vec<Enemy>,
    enemy_bullets: Vec<Bullet>,
    explosion_frames: Rc<Vec<Sprite>>,
    frame: Vec<u32>,
    input: u32,
    interacted: bool,
    last: Option<Instant>,
    state: State,
    palette: Palette,
    player: Player,
    player_bullets: Vec<Bullet>,
    rng: Rng,
    sfx: Option<Sfx>,
    starfield: Starfield,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    tick: u32,
    volley_timer: u32,
    window: Option<Arc<Window>>,
}

#[derive(Clone, Copy)]
enum Action {
    Up,
    Down,
    Left,
    Right,
    Fire,
}

#[derive(Default)]
enum State {
    #[default]
    Waiting,
    Playing,
    Win,
    GameOver,
}

#[cfg(target_arch = "wasm32")]
mod jam {
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name = jamShouldStart)]
        pub fn should_start() -> bool;

        #[wasm_bindgen(js_name = jamGetDifficulty)]
        pub fn difficulty() -> f32;

        #[wasm_bindgen(js_name = jamStarted)]
        pub fn started(verb: &str);

        #[wasm_bindgen(js_name = jamDone)]
        pub fn done(win: bool);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod jam {
    pub fn should_start() -> bool {
        true
    }

    pub fn difficulty() -> f32 {
        0.0
    }

    pub fn started(_verb: &str) {}

    pub fn done(_win: bool) {}
}

impl Cj2k26 {
    fn has_action(&self, action: Action) -> bool {
        self.input & (1 << action as u32) != 0
    }

    fn unlock_audio(&mut self) {
        if let Some(sfx) = &mut self.sfx {
            sfx.resume();
        }
    }

    fn update(&mut self) {
        self.tick = self.tick.wrapping_add(1);

        if matches!(self.state, State::Waiting) && jam::should_start() {
            jam::started("Shoot!");
            self.difficulty = jam::difficulty().min(7.0);
            self.state = State::Playing;
        }

        if self.input != 0 {
            self.interacted = true;
        }

        let playing = matches!(self.state, State::Playing);

        self.starfield.update();

        if playing {
            let player_dir = PLAYER_MOVES
                .iter()
                .filter(|(a, _)| self.has_action(*a))
                .fold(IVec2::ZERO, |acc, (_, v)| acc + *v);

            if player_dir != IVec2::ZERO {
                self.cursor = None;
            }

            let mut mouse = Vec2::ZERO;
            if let Some(cursor) = self.cursor {
                let target = cursor - self.player.size() / 2.0;
                mouse = target - self.player.pos();
            }

            self.player.update(player_dir, mouse);
        }

        let player_center = self.player.pos() + self.player.size() / 2.0;

        if playing && self.interacted && !self.enemies.is_empty() {
            let dive_after = (90.0 - self.difficulty * 6.0).max(45.0) as u32;
            self.dive_timer += 1;
            if self.dive_timer >= dive_after {
                self.dive_timer = 0;

                let i = self.rng.range(self.enemies.len() as u32) as usize;
                let shots = (2.0 + self.difficulty / 4.0).min(5.0) as u32;
                self.enemies[i].start_dive(player_center.x, shots, &mut self.rng);
            }

            let volley_after = (70.0 - self.difficulty * 4.0).max(30.0) as u32;
            self.volley_timer += 1;
            if self.volley_timer >= volley_after {
                self.volley_timer = 0;

                let volley = (2.0 + self.difficulty / 3.0).min(5.0) as u32;
                for _ in 0..volley {
                    let i = self.rng.range(self.enemies.len() as u32) as usize;
                    let enemy = &self.enemies[i];
                    let muzzle = enemy.center() + Vec2::new(0.0, enemy.size().y / 2.0);
                    let spread = Vec2::new((self.rng.f32() - 0.5) * 60.0, 0.0);

                    self.enemy_bullets
                        .push(Bullet::aimed(muzzle, player_center + spread));
                }

                if let Some(sfx) = &self.sfx {
                    sfx.enemy_shoot();
                }
            }
        }

        let seconds = self.tick as f32 / 60.0;
        let sway = 12.0 * (seconds * TAU / 3.0).sin();
        for enemy in &mut self.enemies {
            if let Some(muzzle_pos) = enemy.update(sway, player_center)
                && playing
            {
                self.enemy_bullets
                    .push(Bullet::aimed(muzzle_pos, self.player.pos()));
                if let Some(sfx) = &self.sfx {
                    sfx.enemy_shoot();
                }
            }
        }

        if playing
            && self.has_action(Action::Fire)
            && let Some(muzzle_pos) = self.player.try_fire()
        {
            if let Some(sfx) = &self.sfx {
                sfx.shoot();
            }

            self.player_bullets.push(Bullet::fired(muzzle_pos));
        }

        for bullet in self
            .player_bullets
            .iter_mut()
            .chain(&mut self.enemy_bullets)
        {
            bullet.update();
        }

        self.player_bullets.retain(|b| !b.offscreen());
        self.enemy_bullets.retain(|b| !b.offscreen());

        self.player_bullets.retain(|b| {
            match self.enemies.iter().position(|e| b.hits(e.pos(), e.size())) {
                Some(i) => {
                    if self.enemies[i].damage() {
                        let enemy = self.enemies.swap_remove(i);
                        self.anims
                            .push(Anim::new(self.explosion_frames.clone(), enemy.center()));
                        if let Some(sfx) = &self.sfx {
                            sfx.explode();
                        }
                    } else if let Some(sfx) = &self.sfx {
                        sfx.hit();
                    }
                    false
                }
                None => true,
            }
        });

        if playing {
            let p_pos = self.player.pos();
            let p_size = self.player.size();

            let mut player_hit = false;
            self.enemy_bullets.retain(|b| {
                !b.hits(p_pos, p_size) || {
                    player_hit = true;
                    false
                }
            });

            player_hit |= self
                .enemies
                .iter()
                .any(|e| aabb(p_pos, p_size, e.pos(), e.size()));

            if player_hit {
                self.state = State::GameOver;
                self.done_timer = DONE_HOLD;
                self.anims.push(Anim::new(
                    self.explosion_frames.clone(),
                    p_pos + p_size / 2.0,
                ));
                if let Some(sfx) = &self.sfx {
                    sfx.explode();
                    sfx.lose();
                }
            }
        }

        if matches!(self.state, State::Playing) && self.enemies.is_empty() {
            self.state = State::Win;
            self.done_timer = DONE_HOLD;
            if let Some(sfx) = &self.sfx {
                sfx.win();
            }
        }

        if matches!(self.state, State::Win) {
            self.player
                .win_anim(DONE_HOLD.saturating_sub(self.done_timer));
        }

        if self.done_timer > 0 {
            self.done_timer -= 1;
            if self.done_timer == 0 {
                jam::done(matches!(self.state, State::Win));
            }
        }

        for anim in &mut self.anims {
            anim.update();
        }
        self.anims.retain(|a| !a.done());
    }

    fn draw(&mut self, a: f32) {
        self.frame.fill(self.palette.at(BLACK));

        self.starfield.draw(&mut self.frame, &self.palette, a);

        if !matches!(self.state, State::GameOver) {
            self.player
                .draw(&mut self.frame, &self.palette, a, self.tick);
        }

        for enemy in &self.enemies {
            enemy.draw(&mut self.frame, &self.palette, a, self.tick);
        }

        for bullet in self.player_bullets.iter().chain(&self.enemy_bullets) {
            bullet.draw(&mut self.frame, &self.palette, a);
        }

        for anim in &self.anims {
            anim.draw(&mut self.frame, &self.palette);
        }

        let cycle = (self.tick / 8) as usize;
        let banner = match self.state {
            State::Win => Some(("YOU WIN", WIN_CYCLE[cycle % WIN_CYCLE.len()])),
            State::GameOver => Some(("GAME OVER", OVER_CYCLE[cycle % OVER_CYCLE.len()])),
            _ => None,
        };

        if let Some((text, color)) = banner {
            let w = text.len() as i32 * FONT_10X20.character_size.width as i32;
            let pos = IVec2::new((GAME_W as i32 - w) / 2, (GAME_H as i32 - 20) / 2);
            gfx::draw_text(
                &mut self.frame,
                &FONT_10X20,
                text,
                pos,
                self.palette.at(color),
            );
        }

        if !self.interacted && matches!(self.state, State::Waiting | State::Playing) {
            let lines = ["Mouse or WASD to Move", "Left Click or Space to Fire"];
            for (i, hint) in lines.iter().enumerate() {
                let w = hint.len() as i32 * FONT_6X10.character_size.width as i32;
                let y = GAME_H as i32 / 2 + 20 + i as i32 * 12;
                let pos = IVec2::new((GAME_W as i32 - w) / 2, y);
                gfx::draw_text(
                    &mut self.frame,
                    &FONT_6X10,
                    hint,
                    pos,
                    self.palette.at(WHITE),
                );
            }
        }

        self.present();
    }

    fn present(&mut self) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let Some(surface) = self.surface.as_mut() else {
            return;
        };

        let size = window.inner_size();
        let (w, h) = (size.width as usize, size.height as usize);
        let scale = (w / GAME_W as usize).min(h / GAME_H as usize);
        if scale == 0 {
            return;
        }

        let Ok(mut buffer) = surface.buffer_mut() else {
            return;
        };
        if buffer.len() != w * h {
            return;
        }
        let out_w = GAME_W as usize * scale;
        let x0 = w.saturating_sub(out_w) / 2;
        let y0 = h.saturating_sub(GAME_H as usize * scale) / 2;

        buffer.fill(0);

        let mut row = vec![0u32; out_w];
        for gy in 0..GAME_H as usize {
            let src = &self.frame[gy * GAME_W as usize..(gy + 1) * GAME_W as usize];
            for (i, &c) in src.iter().enumerate() {
                row[i * scale..(i + 1) * scale].fill(c);
            }

            for sy in 0..scale {
                let start = (y0 + gy * scale + sy) * w + x0;
                buffer[start..start + out_w].copy_from_slice(&row);
            }
        }

        let _ = buffer.present();
    }
}

impl ApplicationHandler for Cj2k26 {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = Window::default_attributes().with_title("cj2k26");

        #[cfg(not(target_arch = "wasm32"))]
        let attributes = {
            use winit::dpi::LogicalSize;

            let size = LogicalSize::new(GAME_W * 4, GAME_H * 4);
            attributes.with_inner_size(size).with_min_inner_size(size)
        };

        #[cfg(target_arch = "wasm32")]
        let attributes = {
            use winit::platform::web::WindowAttributesExtWebSys;
            attributes.with_append(true)
        };

        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("failed to create window"),
        );
        window.set_cursor_visible(false);

        #[cfg(not(target_arch = "wasm32"))]
        {
            use winit::window::CursorGrabMode;
            let _ = window
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));
        }

        let context = Context::new(window.clone()).expect("failed to create softbuffer context");
        let mut surface = Surface::new(&context, window.clone()).expect("failed to create surface");

        let inner = window.inner_size();
        if let (Some(w), Some(h)) = (NonZeroU32::new(inner.width), NonZeroU32::new(inner.height)) {
            let _ = surface.resize(w, h);
        }

        self.frame = vec![0; (GAME_W * GAME_H) as usize];
        self.surface = Some(surface);

        let sprites = AsepriteFile::from_reader(SPRITES).expect("failed to read sprites aseprite!");

        self.enemies = (0..8)
            .map(|i| {
                let hp = if i % 3 == 0 { 2 } else { 1 };
                Enemy::new(
                    &sprites,
                    "enemy",
                    Vec2::new(24.0 + i as f32 * 24.0, 24.0),
                    hp,
                )
            })
            .collect();

        self.explosion_frames = Rc::new(Sprite::frames_from_ase(&sprites, "explosion"));
        self.palette = Palette::from_ase(sprites.palette());
        self.player = Player::new(&sprites, 2.0);
        self.rng = Rng::default();
        self.sfx = Some(Sfx::new());
        self.starfield = Starfield::new(60, &mut self.rng);
        self.window = Some(window.clone());

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(surface) = self.surface.as_mut()
                    && let (Some(w), Some(h)) =
                        (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                {
                    let _ = surface.resize(w, h);
                }
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = self
                    .last
                    .replace(now)
                    .map(|prev| (now - prev).min(MAX_FRAME))
                    .unwrap_or_default();
                self.accumulator += dt;

                while self.accumulator >= TICK {
                    self.update();
                    self.accumulator -= TICK;
                }

                let alpha = self.accumulator.as_secs_f32() / TICK.as_secs_f32();
                self.draw(alpha);

                #[cfg(not(target_arch = "wasm32"))]
                std::thread::sleep(TICK.saturating_sub(now.elapsed()));

                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => {
                self.unlock_audio();

                if let Some(action) = bind_key(code) {
                    let bit = 1 << action as u32;

                    match state {
                        ElementState::Pressed => self.input |= bit,
                        ElementState::Released => self.input &= !bit,
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                if let Some(window) = self.window.as_ref() {
                    let size = window.inner_size();
                    let (w, h) = (size.width as i32, size.height as i32);
                    let scale = (w / GAME_W as i32).min(h / GAME_H as i32);

                    if scale > 0 {
                        let x0 = (w - GAME_W as i32 * scale) / 2;
                        let y0 = (h - GAME_H as i32 * scale) / 2;
                        let gx = (position.x as i32 - x0) as f32 / scale as f32;
                        let gy = (position.y as i32 - y0) as f32 / scale as f32;

                        self.cursor = Some(Vec2::new(gx, gy));
                        if matches!(self.state, State::Playing) {
                            self.interacted = true;
                        }
                    }
                }
            }

            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.unlock_audio();

                let bit = 1 << Action::Fire as u32;
                match state {
                    ElementState::Pressed => self.input |= bit,
                    ElementState::Released => self.input &= !bit,
                }
            }

            WindowEvent::MouseInput { .. } | WindowEvent::KeyboardInput { .. } => {
                self.unlock_audio();
            }

            _ => {}
        }
    }
}

fn bind_key(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::KeyW | KeyCode::ArrowUp => Some(Action::Up),
        KeyCode::KeyS | KeyCode::ArrowDown => Some(Action::Down),
        KeyCode::KeyA | KeyCode::ArrowLeft => Some(Action::Left),
        KeyCode::KeyD | KeyCode::ArrowRight => Some(Action::Right),
        KeyCode::Space => Some(Action::Fire),

        _ => None,
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("failed to create event loop");

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
        event_loop
            .run_app(&mut Cj2k26::default())
            .expect("failed to run app");
    }

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;

        console_error_panic_hook::set_once();
        event_loop.spawn_app(Cj2k26::default());
    }
}
