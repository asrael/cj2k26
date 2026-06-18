mod sfx;

use sfx::Sfx;

use std::sync::Arc;

use glam::IVec2;
use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

const GAME_W: usize = 240;
const GAME_H: usize = 160;

// palette colors by index
const TRANSPARENT: u8 = 0;
const ORANGE: u8 = 1;

#[derive(Default)]
struct Cj2k26 {
    pixels: Option<Pixels<'static>>,
    screen: Screen,
    sfx: Option<Sfx>,
}

impl Cj2k26 {
    fn update(&mut self) {
        for sprite in self.screen.sprites_mut() {
            let mut bbox = sprite.bbox;
            let mut bounced = false;

            if bbox[0].x < 0 || bbox[0].x + bbox[1].x > GAME_W as i32 {
                sprite.vel.x = -sprite.vel.x;
                bounced = true;
            }

            if bbox[0].y < 0 || bbox[0].y + bbox[1].y > GAME_H as i32 {
                sprite.vel.y = -sprite.vel.y;
                bounced = true;
            }

            bbox[0] += sprite.vel;
            sprite.bbox = bbox;

            if bounced {
                if let Some(sfx) = &self.sfx {
                    sfx.bounce();
                }
            }
        }
    }

    fn render(&mut self) {
        let Some(pixels) = self.pixels.as_mut() else {
            return;
        };

        let frame = pixels.frame_mut();

        for pxl in frame.chunks_exact_mut(4) {
            pxl.fill(0);
        }

        for sprite in &self.screen.sprites {
            let pos = sprite.bbox[0];
            let w = sprite.bbox[1].x;

            for (i, &px) in sprite.pixels.iter().enumerate() {
                if px == TRANSPARENT {
                    continue;
                }

                let sx = pos.x + i as i32 % w;
                let sy = pos.y + i as i32 / w;
                if sx < 0 || sx >= GAME_W as i32 || sy < 0 || sy >= GAME_H as i32 {
                    continue;
                }

                let idx = (sy as usize * GAME_W + sx as usize) * 4;
                frame[idx..idx + 4].copy_from_slice(&self.screen.palette.rgba(px));
            }
        }
    }
}

type Color = u32;

trait Rgba {
    fn rgba(&self) -> [u8; 4];
}

impl Rgba for Color {
    fn rgba(&self) -> [u8; 4] {
        [(self >> 16) as u8, (self >> 8) as u8, *self as u8, 0xFF]
    }
}

#[derive(Clone, Copy)]
struct Palette([Color; 256]);

impl Default for Palette {
    fn default() -> Self {
        Self::new()
    }
}

impl Palette {
    pub const fn new() -> Self {
        Self([0; 256])
    }

    pub fn at(&self, index: u8) -> Color {
        self.0[index as usize]
    }

    pub fn cycle(&self, base: u8, offset: u8) -> Color {
        self.0[base.wrapping_add(offset) as usize]
    }

    pub fn index_of(&self, color: Color) -> Option<u8> {
        self.0.iter().position(|&c| c == color).map(|i| i as u8)
    }

    pub fn set(&mut self, index: u8, color: Color) {
        self.0[index as usize] = color;
    }

    pub fn rgba(&self, index: u8) -> [u8; 4] {
        let c = self.0[index as usize];
        c.rgba()
    }
}

#[derive(Clone)]
struct Screen {
    palette: Palette,
    sprites: Vec<Sprite>,
    window: Option<Arc<Window>>,
}

impl Default for Screen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen {
    pub fn new() -> Self {
        Self {
            palette: Palette::default(),
            sprites: Vec::with_capacity(4096),
            window: None,
        }
    }

    pub fn sprites_mut(&mut self) -> &mut Vec<Sprite> {
        &mut self.sprites
    }
}

#[derive(Clone, Default)]
struct Sprite {
    bbox: [IVec2; 2],
    pixels: Vec<u8>,
    vel: IVec2,
}

impl Sprite {
    pub fn new(pos: IVec2, w: i32, h: i32, pixels: Vec<u8>, vel: IVec2) -> Self {
        assert_eq!(
            pixels.len(),
            (w * h) as usize,
            "sprite pixel buffer length must equal width * height"
        );

        Self {
            bbox: [pos, IVec2::new(w, h)],
            pixels,
            vel,
        }
    }
}

impl ApplicationHandler for Cj2k26 {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut screen = Screen::new();
        let size = LogicalSize::new(1280, 720);
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("cj2k26")
                        .with_inner_size(size)
                        .with_min_inner_size(size),
                )
                .expect("failed to create window"),
        );

        let cx = GAME_W as i32 / 2;
        let cy = GAME_H as i32 / 2;
        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
        let pixels = Pixels::new(GAME_W as u32, GAME_H as u32, surface_texture)
            .expect("failed to create pixels");

        screen.palette.set(ORANGE, 0xFF6633);
        screen.sprites_mut().push(Sprite::new(
            IVec2::new(cx, cy),
            16,
            16,
            vec![ORANGE; 16 * 16],
            IVec2::new(1, 1),
        ));

        window.request_redraw();
        screen.window = Some(window.clone());

        self.pixels = Some(pixels);
        self.screen = screen;
        self.sfx = Some(Sfx::new());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(pixels) = self.pixels.as_mut() {
                    let _ = pixels.resize_surface(size.width, size.height);
                }
            }

            WindowEvent::RedrawRequested => {
                self.update();
                self.render();

                if let Some(pixels) = self.pixels.as_mut() {
                    if let Err(err) = pixels.render() {
                        error!("render error: {err}");
                        return;
                    }
                }

                if let Some(window) = self.screen.window.as_ref() {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut cj2k26 = Cj2k26::default();

    event_loop.run_app(&mut cj2k26).expect("failed to run app");
}
