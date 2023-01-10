

@group(0) @binding(0)
var render_output: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16, 1)
fn raytrace(@builtin(global_invocation_id) input: vec3<u32>) {
    let uv: vec3<u32> = input;
    let coords: vec2<i32> = vec2<i32>(0u, 0u);

    textureStore(render_output, coords, vec4<f32>(0.0, 0.0, 0.0, 1.0));
}

@vertex
fn vs_main(@builtin(vertex_index) input: u32) -> @builtin(position) vec4<f32> {
    switch input {
        case 0u { return vec4<f32>( 0.0, 1.0, 0.0, 1.0); }
        case 1u { return vec4<f32>(-1.0, 0.0, 0.0, 1.0); }
        case 2u { return vec4<f32>( 1.0, 0.0, 0.0, 1.0); }
        default {}
    }
    
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}