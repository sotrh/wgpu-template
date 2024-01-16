mod debug;

use std::f32::consts::PI;

use wgpu::RenderPassDescriptor;
use winit::{keyboard::{PhysicalKey, KeyCode}, event::DeviceId};

use crate::{
    context::{Context, Frame},
    resources::{camera, texture},
};

pub enum Event {}

pub struct Demo {
    #[allow(dead_code)]
    events: Vec<Event>,
    depth_texture: texture::Texture,
    debug: debug::DebugPipeline,
    #[allow(dead_code)]
    camera: camera::Camera,
    camera_binding: camera::CameraBinding,
    pub running: bool,
}

impl Demo {
    pub fn new(context: &Context, width: u32, height: u32) -> anyhow::Result<Self> {
        let depth_texture = texture::Texture::depth_texture(&context.device, width, height);

        let camera = camera::Camera::look_at(
            glam::vec3(1.0, 1.0, 2.0),
            glam::vec3(0.0, 0.0, 0.0),
            width as _,
            height as _,
            PI / 4.0,
            0.1,
            100.0,
        );

        let camera_binder = camera::CameraBinder::new(&context.device);
        let camera_binding = camera_binder.bind(&context.device, &camera);

        let mut debug = debug::DebugPipeline::new(
            context,
            &camera_binder,
            context.surface_format(),
            depth_texture.format(),
        );

        { 
            let mut batch = debug.line_batch(context);
            batch.push_axes(0.5);
        }

        Ok(Self {
            events: Vec::new(),
            depth_texture,
            debug,
            camera,
            camera_binding,
            running: true,
        })
    }

    pub fn resize(&mut self, context: &Context, width: u32, height: u32) {
        self.depth_texture.resize(context, width, height);
    }

    pub fn render(&mut self, frame: &mut Frame, _context: &Context) {
        let view = frame.target.texture.create_view(&Default::default());

        let mut pass = frame.encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: self.depth_texture.view(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });

        self.debug.draw(&mut pass, &self.camera_binding);
    }

    pub fn close(&mut self) -> bool {
        true
    }

    pub(crate) fn on_cursor_moved(&mut self, _x: f64, _y: f64) {}

    pub(crate) fn on_cursor_entered(&mut self) {}

    pub(crate) fn on_cursor_left(&mut self) {}

    pub(crate) fn on_device_added(&mut self, _device_id: DeviceId) {}

    pub(crate) fn on_device_removed(&mut self, _device_id: DeviceId) {}

    pub(crate) fn on_mouse_scoll(&mut self, _x: f32, _y: f32) {}

    pub(crate) fn on_axis(&mut self, _axis: u32, _value: f64) {}

    pub(crate) fn on_button(&mut self, _button: u32, _pressed: bool) {}

    pub(crate) fn on_key(&mut self, physical_key: PhysicalKey, pressed: bool) {
        match (physical_key, pressed) {
            (PhysicalKey::Code(KeyCode::Escape), true) => self.running = false,
            _ => {}
        }
    }
}
