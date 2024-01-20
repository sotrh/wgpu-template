use std::ops::Deref;

use winit::{
    event_loop::EventLoop,
    window::{Fullscreen, Window as WinitWindow, WindowBuilder}, dpi::PhysicalSize,
};

use crate::Config;

pub struct Window {
    window: WinitWindow,
}

impl Window {
    pub fn new(config: &Config, event_loop: &EventLoop<()>) -> anyhow::Result<Window> {
        let window = WindowBuilder::new()
            .with_visible(false)
            .build(&event_loop)?;

        if config.fullscreen {
            window.set_fullscreen(Some(Fullscreen::Borderless(find_or_first(
                window.available_monitors(),
                |m| m.name() == config.monitor,
            ))))
        } else {
            let _ = window.request_inner_size(PhysicalSize {
                width: config.width,
                height: config.height,
            });
        }

        Ok(Self { window })
    }

    pub fn show(&self) {
        self.window.set_visible(true);
    }

    pub fn toggle_fullscreen(&mut self) {
        if self.is_fullscreen() {
            self.window.set_fullscreen(None);
        } else {
            self.window
                .set_fullscreen(Some(Fullscreen::Borderless(None)));
        }
    }

    pub fn is_fullscreen(&self) -> bool {
        self.window.fullscreen().is_some()
    }

    pub fn modify_config(&self, config: &mut Config) {
        config.fullscreen = self.is_fullscreen();
        config.monitor = self.window.current_monitor().map(|m| m.name()).flatten();
    }
}

impl AsRef<WinitWindow> for Window {
    fn as_ref(&self) -> &WinitWindow {
        &self.window
    }
}

impl Deref for Window {
    type Target = WinitWindow;
    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

fn find_or_first<T>(
    mut iter: impl Iterator<Item = T>,
    predicate: impl Fn(&T) -> bool,
) -> Option<T> {
    let mut found = iter.next();

    for item in iter {
        if predicate(&item) {
            found = Some(item);
            break;
        }
    }

    found
}
