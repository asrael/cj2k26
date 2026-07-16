mod color;
mod gfx;
mod sfx;
mod sprite;

use color::Palette;
use gfx::Drawable;
use sfx::Sfx;
use sprite::Sprite;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use glam::IVec2;
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

// palette colors by index
const TRANSPARENT: u8 = 0;
const ORANGE: u8 = 1;
const GRAY: u8 = 2;

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
    input: u32,
    last: Option<Instant>,
    palette: Palette,
    pixels: Rc<RefCell<Option<Pixels<'static>>>>,
    player: Player,
    sfx: Option<Sfx>,
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
struct Player {
    sprite: Sprite,
    vel: IVec2,
}

impl Cj2k26 {
    fn has_action(&self, action: Action) -> bool {
        self.input & (1 << action as u32) != 0
    }

    fn update(&mut self) {
        let player_dir = PLAYER_MOVES
            .iter()
            .filter(|(a, _)| self.has_action(*a))
            .fold(IVec2::ZERO, |acc, (_, v)| acc + *v);

        self.player.sprite.pos += player_dir * self.player.vel;
        self.player.sprite.pos.x = self.player.sprite.pos.x.clamp(0, GAME_W as i32 - 16);
        self.player.sprite.pos.y = self.player.sprite.pos.y.clamp(0, GAME_H as i32 - 16);

        if self.has_action(Action::Fire) {
            if let Some(sfx) = &self.sfx {
                sfx.shoot();
            }
        }
    }

    fn draw(&mut self) {
        let mut pixels = self.pixels.borrow_mut();
        let Some(pixels) = pixels.as_mut() else {
            return;
        };

        let frame = pixels.frame_mut();
        let player = &mut self.player;

        for pxl in frame.chunks_exact_mut(4) {
            pxl.copy_from_slice(&[0x1D, 0x20, 0x21, 0xFF]);
        }

        player.sprite.draw(frame, &self.palette);
    }
}

impl ApplicationHandler for Cj2k26 {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size = LogicalSize::new(GAME_W * 5, GAME_H * 5);
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

        let cx = GAME_W as i32 / 2;
        let cy = GAME_H as i32 / 2;
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

        self.palette.set(ORANGE, 0xFF6633);
        self.palette.set(GRAY, 0x808080);
        self.player = Player {
            sprite: Sprite::new(16, 16, IVec2::new(cx, cy), vec![ORANGE; 16 * 16]),
            vel: IVec2::new(1, 1),
        };
        self.window = Some(window.clone());
        self.sfx = Some(Sfx::new());

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

                self.draw();

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
                if let Some(action) = bind(code) {
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

fn bind(code: KeyCode) -> Option<Action> {
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
