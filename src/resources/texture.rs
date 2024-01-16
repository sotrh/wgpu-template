use crate::context::Context;

pub struct Texture {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
}

impl Texture {
    pub fn depth_texture(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let format = wgpu::TextureFormat::Depth32Float;
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let (texture, view) = create_2d(device, width, height, format, usage);
        Self { texture, view, format, usage }
    }

    pub fn resize(&mut self, context: &Context, width: u32, height: u32) {
        let (texture, view) = create_2d(&context.device, width, height, self.format, self.usage);
        self.texture = texture;
        self.view = view;
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

fn create_2d(device: &wgpu::Device, width: u32, height: u32, format: wgpu::TextureFormat, usage: wgpu::TextureUsages) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth_texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}
