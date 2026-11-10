// Initial WGSL shader placeholder for the wgpu renderer.
// The production renderer will add camera transforms, material sampling,
// sunlight, ambient occlusion, shadow maps, and optional post-processing.

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
) -> VertexOut {
    var out: VertexOut;
    // TODO: multiply by view-projection matrix.
    out.clip_position = vec4<f32>(position, 1.0);
    out.world_normal = normal;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.4, 0.8, 0.3));
    let diffuse = max(dot(normalize(in.world_normal), light_dir), 0.15);
    let base = vec3<f32>(0.42, 0.58, 0.34);
    return vec4<f32>(base * diffuse, 1.0);
}
