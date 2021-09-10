use std::time::Instant;

use settings::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDHT};

mod controler;
mod entity;
mod settings;
mod systems;

fn main() {
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::Size::Physical(winit::dpi::PhysicalSize {
            width: WINDOW_WIDHT,
            height: WINDOW_HEIGHT,
        }))
        .with_title(WINDOW_TITLE)
        .build(&event_loop)
        .unwrap();
    let mut controler = pollster::block_on(controler::Controler::new(&window));
    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::WindowEvent { event, .. } => match event {
            winit::event::WindowEvent::CloseRequested => {
                *control_flow = winit::event_loop::ControlFlow::Exit;
            }
            winit::event::WindowEvent::KeyboardInput { input, .. } => match input {
                winit::event::KeyboardInput {
                    state: winit::event::ElementState::Pressed,
                    virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                    ..
                } => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                winit::event::KeyboardInput {
                    state: winit::event::ElementState::Pressed,
                    virtual_keycode: Some(winit::event::VirtualKeyCode::Space),
                    ..
                } => {
                    println!("Request redraw.");
                    controler.update();
                    window.request_redraw();
                }
                _ => {}
            },
            _ => {}
        },
        winit::event::Event::RedrawRequested(_) => {
            let start = Instant::now();
            controler.render();
            // controler.update();
            println!("Total time: {}", start.elapsed().as_millis());
        }
        winit::event::Event::RedrawEventsCleared => {
            // controler.update();
            // window.request_redraw();
        }
        _ => {}
    });
}
