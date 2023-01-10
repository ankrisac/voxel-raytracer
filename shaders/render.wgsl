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