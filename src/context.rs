use anyhow::Context as _;

use winit::{
    dpi::PhysicalSize,
    window::{Fullscreen, Window},
};

use crate::Config;

pub struct Context {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    surface: wgpu::Surface,
    surf_config: wgpu::SurfaceConfiguration,
    running: bool,
    window: Window,
}

impl Context {
    pub async fn new(config: &Config, window: Window) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(Default::default());

        // Safety: surface and window are owned by game
        let surface = unsafe { instance.create_surface(&window)? };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .context("No valid adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await?;

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

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];

        println!("caps: {:?}", caps);

        let surf_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: Vec::new(),
        };
        surface.configure(&device, &surf_config);

        println!("format: {:?}", format);

        Ok(Self {
            device,
            queue,
            surface,
            surf_config,
            window,
            running: true,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surf_config.width = width;
        self.surf_config.height = height;
        self.surface.configure(&self.device, &self.surf_config);
    }

    pub fn render(&mut self, f: impl FnOnce(&mut Frame, &Self)) {
        self.window.request_redraw();

        let target = match self.surface.get_current_texture() {
            Ok(target) => target,
            Err(wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.surf_config);
                return;
            }
            Err(e) => {
                panic!("{}", e);
            }
        };

        let encoder = self.device.create_command_encoder(&Default::default());
        let mut frame = Frame { encoder, target };

        f(&mut frame, self);

        self.queue.submit([frame.encoder.finish()]);
        frame.target.present();
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

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn modify_config(&self, config: &mut Config) {
        config.fullscreen = self.is_fullscreen();
        config.monitor = self.window.current_monitor().map(|m| m.name()).flatten();
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surf_config.format
    }
}

pub struct Frame {
    pub target: wgpu::SurfaceTexture,
    pub encoder: wgpu::CommandEncoder,
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
