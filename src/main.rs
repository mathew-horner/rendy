#![feature(const_fn_floating_point_arithmetic)]

mod geo;
mod io;
mod renderer;
mod texture;

use std::process;
use std::sync::{Arc, Mutex};
use std::thread;

use io::PromptLines;
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
    let state = Arc::new(Mutex::new(SharedState {
        renderer,
        texture_index: 0,
    }));

    {
        let state = state.clone();
        thread::spawn(move || debug_console(state));
    }

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        match event {
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let mut state = state.lock().unwrap();
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
                    state.lock().unwrap().renderer.resize(size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.lock().unwrap().renderer.resize(*new_inner_size);
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
                        let mut state = state.lock().unwrap();
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

fn debug_console(state: Arc<Mutex<SharedState>>) {
    for line in std::io::stdin().prompt_lines(">") {
        let tokens: Vec<_> = line.split(' ').collect();
        match tokens[0] {
            "exit" => process::exit(0),
            "set" => match tokens[1] {
                "renderer.clear_color" => {
                    if let Some(color) = parse_clear_color(tokens[2]) {
                        state.lock().unwrap().renderer.set_background_color(color);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

const fn opaque_color(r: f64, g: f64, b: f64) -> wgpu::Color {
    wgpu::Color { r, g, b, a: 1.0 }
}

const COLOR_VALUES: &[(&str, wgpu::Color)] = &[
    ("red", opaque_color(1.0, 0.0, 0.0)),
    ("blue", opaque_color(0.0, 0.0, 1.0)),
    ("green", opaque_color(0.0, 1.0, 0.0)),
];

fn parse_clear_color(value: &str) -> Option<wgpu::Color> {
    COLOR_VALUES
        .iter()
        .find(|(name, _)| *name == value)
        .map(|(_, color)| *color)
}
