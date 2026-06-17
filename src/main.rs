const GAME_W: u32 = 240;
const GAME_H: u32 = 160;

use std::sync::Arc;

use glam::IVec2;
use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct Cj2k26 {
    pixels: Option<Pixels<'static>>,
    window: Option<Arc<Window>>,
    world: Option<World>,
}

struct Sprite {
    pos: IVec2,
    vel: IVec2,
    size: IVec2,
}

#[derive(Default)]
struct World {
    sprites: Vec<Sprite>,
}

impl World {
    fn update(&mut self) {}

    fn render(&self, frame: &mut [u8]) {}

    fn sprites(&mut self) -> &mut Vec<Sprite> {
        &mut self.sprites
    }
}

impl ApplicationHandler for Cj2k26 {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
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

        let win_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(win_size.width, win_size.height, window.clone());
        let pixels = Pixels::new(GAME_W, GAME_H, surface_texture).expect("failed to create pixels");
        let mut world = World::default();

        world.sprites().push(Sprite {
            pos: IVec2::new((GAME_W / 2) as i32, (GAME_H / 2) as i32),
            vel: IVec2::new(2, 3),
            size: IVec2::new(16, 16),
        });

        self.pixels = Some(pixels);
        self.window = Some(window.clone());
        self.world = Some(world);

        window.request_redraw();
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
                if let (Some(pixels), Some(world)) = (self.pixels.as_mut(), self.world.as_mut()) {
                    world.update();
                    world.render(pixels.frame_mut());

                    if let Err(err) = pixels.render() {
                        error!("render error: {err}");
                        return;
                    }
                }

                if let Some(window) = self.window.as_ref() {
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
