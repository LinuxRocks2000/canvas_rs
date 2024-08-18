// note: this is NOT final. I'm just trying to get wgpu to work at all, 'kay?
// we're gonna move to a set of distinct shaders for different shape primitives #soon
// bezier curves are going to SUCK

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vertex(@builtin(position) vertex_position : vec4<f32>,) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vertex_position;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}
