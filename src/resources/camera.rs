use std::f32::consts::PI;

use bytemuck::bytes_of;
use glam::Vec3Swizzles;
use wgpu::util::{DeviceExt, BufferInitDescriptor};

const MAX_PITCH: f32 = PI - 0.01;
const MIN_PITCH: f32 = -MAX_PITCH;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraData {
    view_proj: glam::Mat4,
}

pub struct CameraBinder {
    layout: wgpu::BindGroupLayout,
}

impl CameraBinder {
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("CameraBinder"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        Self { layout }
    }

    pub fn bind(&self, device: &wgpu::Device, camera: &Camera) -> CameraBinding {
        let view_proj = camera.calc_proj() * camera.calc_view();
        let data = CameraData { view_proj };
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("CameraBinding::buffer"),
            contents: bytemuck::bytes_of(&data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("CameraBinding::bind_group"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        CameraBinding { bind_group, buffer, data }
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }
}

pub struct CameraBinding {
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    data: CameraData,
}

impl CameraBinding {
    pub fn update(&mut self, queue: &wgpu::Queue, camera: &Camera) {
        self.data.view_proj = camera.calc_proj() * camera.calc_view();
        queue.write_buffer(&self.buffer, 0, bytes_of(&self.data));
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

#[derive(Debug)]
pub struct Camera {
    aspect: f32,
    fovy: f32,
    near: f32,
    far: f32,
    eye: glam::Vec3,
    yaw: f32,
    pitch: f32,
    forward: glam::Vec3,
    right: glam::Vec3,
    up: glam::Vec3,
}

impl Camera {
    pub fn look_at(eye: glam::Vec3, position: glam::Vec3, width: f32, height: f32, fovy: f32, near: f32, far: f32) -> Self {
        let forward = (position - eye).normalize();
        let right = forward.cross(glam::Vec3::Y);
        let up = right.cross(forward);
        // let up = glam::Vec3::Y;
        let pitch = (-forward.y).asin();
        let yaw = forward.z.atan2(forward.x);

        Self {
            eye,
            yaw,
            pitch,
            forward,
            right,
            up,
            aspect: width / height,
            fovy,
            near,
            far,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn rotate_right(&mut self, amount: f32) {
        self.yaw += amount;
        // self.yaw = self.yaw - TAU * (self.yaw / TAU);
        let (syaw, cyaw) = self.yaw.sin_cos();
        let cpitch = self.pitch.cos();
        self.forward.x = cyaw * cpitch;
        self.forward.z = syaw * cpitch;

        // I could figure out how to do this with sin/cos
        // but I don't feel like it.
        self.right = self.forward.cross(glam::Vec3::Y);
        // self.up = self.forward.cross(self.right);
    }

    pub fn rotate_up(&mut self, amount: f32) {
        self.pitch += amount;
        self.pitch = self.pitch.min(MAX_PITCH).max(MIN_PITCH);
        self.forward.y = self.pitch.sin();
    }

    pub fn walk_forward(&mut self, amount: f32) {
        let movement = self.forward.xz().normalize() * amount;
        self.eye.x += movement.x;
        self.eye.z += movement.y;
    }

    pub fn walk_right(&mut self, amount: f32) {
        let movement = self.right.xz().normalize() * amount;
        self.eye.x += movement.x;
        self.eye.z += movement.y;
    }

    pub fn levitate_up(&mut self, amount: f32) {
        self.eye.y += amount;
    }

    pub fn calc_view(&self) -> glam::Mat4 {
        glam::Mat4::look_to_rh(self.eye, self.forward, self.up)
    }

    pub fn calc_proj(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fovy, self.aspect, self.near, self.far)
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }
}
