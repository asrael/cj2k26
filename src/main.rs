mod color;
mod gfx;
mod grid;
mod screen;
mod sfx;
mod sprite;
mod tilemap;

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
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

#[cfg(target_arch = "wasm32")]
use pixels::PixelsBuilder;

use gfx::Drawable;
use grid::Grid;
use screen::Screen;
use sfx::Sfx;
use sprite::Sprite;
use tilemap::{Tilemap, Tileset};

pub(crate) const GAME_W: usize = 240;
pub(crate) const GAME_H: usize = 160;

const MAX_FRAME: Duration = Duration::from_millis(100);
const TICK: Duration = Duration::from_nanos(16_666_667);
const TILE: i32 = 8;

// palette colors by index
const TRANSPARENT: u8 = 0;
const ORANGE: u8 = 1;
const GRAY: u8 = 2;

#[derive(Default)]
struct Cj2k26 {
    pixels: Rc<RefCell<Option<Pixels<'static>>>>,
    screen: Screen,
    sfx: Option<Sfx>,
    last: Option<Instant>,
    accumulator: Duration,
}

impl Cj2k26 {
    fn update(&mut self) {
        for sprite in &mut self.screen.sprites {
            let pos = sprite.pos;
            let dim = sprite.dim;
            let mut bounced = false;

            let nx = pos.x + sprite.vel.x;
            if nx < 0
                || nx + dim.x > GAME_W as i32
                || self.screen.grid.aabb(IVec2::new(nx, pos.y), dim)
            {
                sprite.vel.x = -sprite.vel.x;
                bounced = true;
            }

            let ny = pos.y + sprite.vel.y;
            if ny < 0
                || ny + dim.y > GAME_H as i32
                || self.screen.grid.aabb(IVec2::new(pos.x, ny), dim)
            {
                sprite.vel.y = -sprite.vel.y;
                bounced = true;
            }

            sprite.pos += sprite.vel;

            if bounced {
                if let Some(sfx) = &self.sfx {
                    sfx.bounce();
                }
            }
        }
    }

    fn draw(&mut self) {
        let mut pixels = self.pixels.borrow_mut();
        let Some(pixels) = pixels.as_mut() else {
            return;
        };

        let frame = pixels.frame_mut();

        for pxl in frame.chunks_exact_mut(4) {
            pxl.copy_from_slice(&[0, 0, 0, 0xFF]);
        }

        for tilemap in &self.screen.tilemaps {
            tilemap.draw(frame, &self.screen.palette);
        }

        for sprite in &self.screen.sprites {
            sprite.draw(frame, &self.screen.palette);
        }
    }
}

impl ApplicationHandler for Cj2k26 {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut screen = Screen::new();
        let size = LogicalSize::new(1280, 720);
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

        screen.palette.set(ORANGE, 0xFF6633);
        screen.palette.set(GRAY, 0x808080);

        screen.sprites.push(Sprite::new(
            IVec2::new(cx, cy),
            16,
            16,
            vec![ORANGE; 16 * 16],
            IVec2::new(1, 1),
        ));

        let mut tiles = vec![TRANSPARENT; (TILE * TILE) as usize];
        tiles.resize((2 * TILE * TILE) as usize, GRAY);

        let cols = GAME_W as i32 / TILE;
        let rows = GAME_H as i32 / TILE;
        let mut cells = vec![0u16; (cols * rows) as usize];
        let mut grid = Grid::new(IVec2::ZERO, cols, rows, TILE, TILE);
        for col in 0..cols {
            for row in [rows - 2, rows - 1] {
                cells[(row * cols + col) as usize] = 1;
                grid.set(col, row, true);
            }
        }
        screen.grid = grid;

        screen.tilemaps.push(Tilemap::new(
            IVec2::ZERO,
            cols,
            rows,
            cells,
            Tileset::new(TILE, TILE, tiles),
        ));

        window.request_redraw();
        screen.window = Some(window.clone());

        self.screen = screen;
        self.sfx = Some(Sfx::new());
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

                if let Some(window) = self.screen.window.as_ref() {
                    window.request_redraw();
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
