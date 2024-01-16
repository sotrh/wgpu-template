struct Camera {
    view_proj: mat4x4<f32>,
}

struct Vertex {
    @location(0)
    position: vec3<f32>,
    @location(1)
    color: vec3<f32>,
}

struct VsOut {
    @builtin(position)
    frag_position: vec4<f32>,
    @location(0)
    color: vec3<f32>,
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(in: Vertex) -> VsOut {
    let frag_position = camera.view_proj * vec4(in.position, 1.0);

    return VsOut(frag_position, in.color);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return vec4(in.color, 1.0);
}