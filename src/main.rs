#![feature(const_fn_floating_point_arithmetic)]

mod geo;
mod renderer;
mod texture;

use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::Window;

use renderer::Renderer;

struct SharedState {
    renderer: Renderer,
    texture_index: usize,
}

const TEXTURES: &[&str] = &["assets/happy-tree.png", "assets/sakura-trees.jpg"];

#[tokio::main]
async fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).expect("Failed to create window.");
    let renderer = Renderer::new(&window, TEXTURES[0], "src/shaders/shader.wgsl").await;
    let mut state = SharedState {
        renderer,
        texture_index: 0,
    };

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        match event {
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                match state.renderer.draw() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        let size = state.renderer.size();
                        state.renderer.resize(size);
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => control_flow.set_exit_with_code(1),
                    Err(error) => eprintln!("{}", error),
                };
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    control_flow.set_exit();
                }
                WindowEvent::Resized(size) => {
                    state.renderer.resize(size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.renderer.resize(*new_inner_size);
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(code),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match code {
                    VirtualKeyCode::Space => {
                        state.texture_index = (state.texture_index + 1) % TEXTURES.len();
                        let index = state.texture_index;
                        state.renderer.set_texture(TEXTURES[index]);
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    });
}
