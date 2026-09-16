@group(0) @binding(0)
var glyph_texture: texture_2d<f32>;

@group(0) @binding(1)
var glyph_sampler: sampler;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,

};

struct VertexPayload {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(vertex: Vertex) -> VertexPayload {
    var out: VertexPayload;
    out.position = vec4<f32>(vertex.position, 1.0);
    out.color = vertex.color;
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(in: VertexPayload) -> @location(0) vec4<f32> {
    let coverage = textureSample(
        glyph_texture,
        glyph_sampler,
        in.uv
    ).r;

    return vec4<f32>(
        in.color.rgb,
        in.color.a * coverage
    );
}