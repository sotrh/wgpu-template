use std::mem;

use wgpu::PrimitiveState;

use crate::{context::Context, resources::{camera::{CameraBinder, self}, buffer}};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: glam::Vec3,
    color: glam::Vec3,
}

impl Vertex {
    pub const VERTEX: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<Self>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
        ],
    };
}

pub struct DebugPipeline {
    pipeline: wgpu::RenderPipeline,
    vertices: buffer::CpuBuffer<Vertex>,
}

impl DebugPipeline {
    pub fn new(
        context: &Context,
        camera_binder: &CameraBinder,
        surface_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let vertices = buffer::CpuBuffer::with_capacity(context, 64, wgpu::BufferUsages::VERTEX);

        let layout = context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[camera_binder.layout()],
                ..Default::default()
            });

        let module = context
            .device
            .create_shader_module(wgpu::include_wgsl!("debug.wgsl"));

        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("DebugPipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: "vs_main",
                    buffers: &[Vertex::VERTEX],
                },
                primitive: PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: depth_format,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: Default::default(),
            });

        Self { pipeline, vertices }
    }

    pub fn line_batch<'a>(&'a mut self, context: &'a Context) -> LineBatch<'a> {
        LineBatch {
            batch: self.vertices.batch(context),
        }
    }

    pub(crate) fn draw<'a: 'b, 'b>(&'a self, pass: &'b mut wgpu::RenderPass<'a>, camera_binding: &'a camera::CameraBinding) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, camera_binding.bind_group(), &[]);
        pass.set_vertex_buffer(0, self.vertices.buffer.slice(..));
        pass.draw(0..self.vertices.len(), 0..1);
    }
}

pub struct LineBatch<'a> {
    batch: buffer::BufferBatch<'a, Vertex>,
}

impl<'a> LineBatch<'a> {
    pub fn push(&mut self, a: glam::Vec3, b: glam::Vec3, color: glam::Vec3) -> &mut Self {
        self.batch
            .push(Vertex { position: a, color })
            .push(Vertex { position: b, color });
        self
    }

    pub fn push_axes(&mut self, length: f32) -> &mut Self {
        self.push(
            glam::vec3(0.0, 0.0, 0.0),
            glam::vec3(length, 0.0, 0.0),
            glam::vec3(1.0, 0.0, 0.0),
        )
        .push(
            glam::vec3(0.0, 0.0, 0.0),
            glam::vec3(0.0, length, 0.0),
            glam::vec3(0.0, 1.0, 0.0),
        )
        .push(
            glam::vec3(0.0, 0.0, 0.0),
            glam::vec3(0.0, 0.0, length),
            glam::vec3(0.0, 0.0, 1.0),
        )
    }
}
