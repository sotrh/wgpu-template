use anyhow::Context as _;

use crate::{Config, window::Window};

pub struct Context<'a> {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    surface: wgpu::Surface<'a>,
    surf_config: wgpu::SurfaceConfiguration,
}

impl<'a> Context<'a> {
    pub async fn new(window: &'a Window) -> anyhow::Result<Context<'a>> {
        let instance = wgpu::Instance::new(Default::default());

        let surface = instance.create_surface(window.as_ref())?;

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
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];

        println!("caps: {:?}", caps);

        let surf_config = surface.get_default_config(
            &adapter,
            window.inner_size().width.max(1),
            window.inner_size().height.max(1),
        ).unwrap();
        surface.configure(&device, &surf_config);

        println!("format: {:?}", format);

        Ok(Self {
            device,
            queue,
            surface,
            surf_config,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surf_config.width = width.max(1);
        self.surf_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surf_config);
    }

    pub fn render(&mut self, f: impl FnOnce(&mut Frame, &Self)) {
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

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surf_config.format
    }
}

pub struct Frame {
    pub target: wgpu::SurfaceTexture,
    pub encoder: wgpu::CommandEncoder,
}
