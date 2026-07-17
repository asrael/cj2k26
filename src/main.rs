mod bullet;
mod color;
mod enemy;
mod gfx;
mod player;
mod rng;
mod sfx;
mod sprite;
mod starfield;

use bullet::Bullet;
use color::Palette;
use color::db32::BLACK;
use enemy::Enemy;
use player::Player;
use rng::Rng;
use sfx::Sfx;
use starfield::Starfield;

use std::cell::RefCell;
use std::f32::consts::TAU;
use std::fs;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use aseprite::AsepriteFile;
use glam::{IVec2, Vec2};
use log::error;
use pixels::{Pixels, SurfaceTexture};
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

#[cfg(target_arch = "wasm32")]
use pixels::PixelsBuilder;

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

#[derive(Default)]
struct Cj2k26 {
    accumulator: Duration,
    dive_timer: u32,
    enemies: Vec<Enemy>,
    enemy_bullets: Vec<Bullet>,
    enemy_sway: f32,
    input: u32,
    last: Option<Instant>,
    palette: Palette,
    pixels: Rc<RefCell<Option<Pixels<'static>>>>,
    player: Player,
    player_bullets: Vec<Bullet>,
    rng: Rng,
    sfx: Option<Sfx>,
    starfield: Starfield,
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

impl Cj2k26 {
    fn has_action(&self, action: Action) -> bool {
        self.input & (1 << action as u32) != 0
    }

    fn update(&mut self) {
        self.starfield.update();

        let player_dir = PLAYER_MOVES
            .iter()
            .filter(|(a, _)| self.has_action(*a))
            .fold(IVec2::ZERO, |acc, (_, v)| acc + *v);

        self.player.update(player_dir);

        self.dive_timer += 1;
        if self.dive_timer >= 180 && !self.enemies.is_empty() {
            self.dive_timer = 0;

            let i = self.rng.range(self.enemies.len() as u32) as usize;
            let player_x = self.player.pos().x;
            self.enemies[i].start_dive(player_x, &mut self.rng);
        }

        self.enemy_sway += 1.0 / 60.0;
        let sway = 12.0 * (self.enemy_sway * TAU / 3.0).sin();
        for enemy in &mut self.enemies {
            if let Some(muzzle_pos) = enemy.update(sway) {
                self.enemy_bullets
                    .push(Bullet::aimed(muzzle_pos, self.player.pos()));
            }
        }

        if self.has_action(Action::Fire) {
            if let Some(muzzle_pos) = self.player.try_fire() {
                if let Some(sfx) = &self.sfx {
                    sfx.shoot();
                }

                self.player_bullets.push(Bullet::fired(muzzle_pos));
            }
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
    }

    fn draw(&mut self, a: f32) {
        let mut pixels = self.pixels.borrow_mut();
        let Some(pixels) = pixels.as_mut() else {
            return;
        };

        let frame = pixels.frame_mut();

        gfx::clear(frame, self.palette.rgb(BLACK));

        self.starfield.draw(frame, &self.palette, a);
        self.player.draw(frame, &self.palette, a);

        for enemy in &self.enemies {
            enemy.draw(frame, &self.palette, a);
        }

        for bullet in self.player_bullets.iter().chain(&self.enemy_bullets) {
            bullet.draw(frame, &self.palette, a);
        }
    }
}

impl ApplicationHandler for Cj2k26 {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size = LogicalSize::new(GAME_W * 4, GAME_H * 4);
        let attributes = Window::default_attributes()
            .with_title("cj2k26")
            .with_inner_size(size)
            .with_min_inner_size(size);

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

        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());

        #[cfg(not(target_arch = "wasm32"))]
        {
            let pixels = Pixels::new(GAME_W as u32, GAME_H as u32, surface_texture)
                .expect("failed to create pixels");
            *self.pixels.borrow_mut() = Some(pixels);
        }

        #[cfg(target_arch = "wasm32")]
        {
            let slot = self.pixels.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let pixels = PixelsBuilder::new(GAME_W as u32, GAME_H as u32, surface_texture)
                    .wgpu_backend(pixels::wgpu::Backends::GL)
                    .build_async()
                    .await
                    .expect("failed to create pixels");
                *slot.borrow_mut() = Some(pixels);
            });
        }

        let rng = Rng::default();
        let sprites_buf =
            fs::read("assets/sprites.aseprite").expect("failed to load player aseprite!");
        let sprites =
            AsepriteFile::from_reader(&sprites_buf[..]).expect("failed to read player aseprite!");

        self.enemies = (0..8)
            .map(|i| Enemy::new(&sprites, "enemy0", Vec2::new(24.0 + i as f32 * 24.0, 24.0)))
            .collect();

        self.palette = Palette::from_ase(sprites.palette());
        self.player = Player::new(&sprites, 2.0);
        self.rng = rng;
        self.sfx = Some(Sfx::new());
        self.starfield = Starfield::new(60, &mut self.rng);
        self.window = Some(window.clone());

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(pixels) = self.pixels.borrow_mut().as_mut() {
                    let _ = pixels.resize_surface(size.width, size.height);
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

                if let Some(pixels) = self.pixels.borrow_mut().as_mut() {
                    if let Err(err) = pixels.render() {
                        error!("render error: {err}");
                        return;
                    }
                }

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
                if let Some(action) = bind_key(code) {
                    let bit = 1 << action as u32;

                    match state {
                        ElementState::Pressed => self.input |= bit,
                        ElementState::Released => self.input &= !bit,
                    }
                }
            }

            WindowEvent::MouseInput { .. } | WindowEvent::KeyboardInput { .. } => {
                if let Some(sfx) = &self.sfx {
                    sfx.resume();
                }
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
    #[cfg(not(target_arch = "wasm32"))]
    {
        desktop_main()
    }
    #[cfg(target_arch = "wasm32")]
    {
        web_main()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn desktop_main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut cj2k26 = Cj2k26::default();

    event_loop.run_app(&mut cj2k26).expect("failed to run app");
}

#[cfg(target_arch = "wasm32")]
fn web_main() {
    use winit::platform::web::EventLoopExtWebSys;

    console_error_panic_hook::set_once();

    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.spawn_app(Cj2k26::default());
}
