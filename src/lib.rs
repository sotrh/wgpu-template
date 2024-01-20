use std::{cell::RefCell, rc::Rc};

use context::Context;
use demo::Demo;
use resources::fs::{load_json, save_json};
use winit::{
    event::{DeviceEvent, ElementState, Event, MouseScrollDelta, StartCause, WindowEvent},
    event_loop::EventLoop,
};

pub mod context;
pub mod demo;
pub mod resources;
mod window;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    width: u32,
    height: u32,
    scroll_sensitivity: f32,
    touch_sensitivity: f32,
    fullscreen: bool,
    monitor: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            scroll_sensitivity: 0.5,
            touch_sensitivity: 0.1,
            monitor: None,
            fullscreen: false,
        }
    }
}

pub async fn run() -> anyhow::Result<()> {
    let config: Config = match load_json("config.json").await {
        Ok(config) => config,
        Err(_) => Default::default(),
    };

    let event_loop = EventLoop::new()?;
    let window = window::Window::new(&config, &event_loop)?;

    let mut context = Context::new(&window).await?;
    let mut demo = Demo::new(&context, config.width, config.height)?;

    let config = Rc::new(RefCell::new(config));
    let final_config = config.clone();

    let window = &window;
    event_loop.run(move |event, target| {
        if !demo.running {
            target.exit();
        }

        match event {
            Event::NewEvents(StartCause::Init) => {
                window.show();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::ActivationTokenDone { serial, token } => {
                    log::info!("ActivationTokenDone {{ {serial:?}, {token:?} }}");
                }
                WindowEvent::Resized(size) => {
                    context.resize(size.width, size.height);
                    demo.resize(&context, size.width, size.height);
                }
                WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    if demo.close() {
                        target.exit()
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    demo.on_cursor_moved(position.x, position.y);
                }
                WindowEvent::CursorEntered { .. } => {
                    demo.on_cursor_entered();
                }
                WindowEvent::CursorLeft { .. } => demo.on_cursor_left(),
                WindowEvent::RedrawRequested => {
                    context.render(|frame, context| {
                        demo.render(frame, context);
                    });
                }
                _ => {}
            },
            Event::DeviceEvent { device_id, event } => match event {
                DeviceEvent::Added => {
                    demo.on_device_added(device_id);
                }
                DeviceEvent::Removed => {
                    demo.on_device_removed(device_id);
                }
                DeviceEvent::MouseWheel { delta } => {
                    let (x, y) = match delta {
                        MouseScrollDelta::LineDelta(x, y) => {
                            let config = final_config.borrow();
                            (x * config.scroll_sensitivity, y * config.scroll_sensitivity)
                        }
                        MouseScrollDelta::PixelDelta(_) => todo!(),
                    };
                    demo.on_mouse_scoll(x, y);
                }
                DeviceEvent::Motion { axis, value } => {
                    demo.on_axis(axis, value);
                }
                DeviceEvent::Button { button, state } => {
                    demo.on_button(button, state == ElementState::Pressed);
                }
                DeviceEvent::Key(key) => {
                    demo.on_key(key.physical_key, key.state == ElementState::Pressed);
                }
                _ => (),
            },
            // winit::event::Event::Suspended => todo!(),
            // winit::event::Event::Resumed => todo!(),
            // winit::event::Event::AboutToWait => todo!(),
            // winit::event::Event::MemoryWarning => todo!(),
            Event::LoopExiting => {
                window.modify_config(&mut final_config.borrow_mut());
            }
            _ => {}
        }
    })?;

    save_json("config.json", &*config.borrow()).await?;

    Ok(())
}
