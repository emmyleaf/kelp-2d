// TODO: finish this at some point, then have the option of writing either glsl or wgsl for custom shaders :)

// --- UNIFORMS ---

struct Instance {
    @location(0) color: vec<f32>,   // contains color to tint sprite
    @location(1) mode: vec<f32>,    // xyz contains draw mode options, w contains texture array layer
    @location(2) source: vec<f32>,  // xy contains UV translation, zw contains UV scale
    @location(3) world: mat4x4<f32> // contains world matrix
};

@group(0) @binding(0) var<storage> instance_buffer: array<Instance>;
@group(0) @binding(1) var<uniform> texture_array: texture_2d_array<f32>;
@group(0) @binding(2) var<uniform> point_sampler: sampler;
@group(0) @binding(3) var<uniform> linear_sampler: sampler;

// --- VERTEX ---

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @builtin(instance_index) instance_index: vec4<f32>,
    @location(0) @interpolate(flat) color: vec4<f32>;
    @location(1) @interpolate(flat) mode: vec4<f32>;
    @location(2) uv: vec2<f32>;
};

@vertex fn vertex(position: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    let instance = instance_buffer[instance_index];

    return out;
}

// --- FRAGMENT ---
