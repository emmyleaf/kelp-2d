#version 450

layout(location = 0) in vec2 fsin_TextureUV;
layout(location = 1) flat in vec2 fsin_LayerSmooth;
layout(location = 2) flat in vec4 fsin_Color;
layout(location = 3) flat in vec4 fsin_Mode;

layout(location = 0) out vec4 fsout_Color;

layout(set = 0, binding = 1) uniform texture2DArray Texture;
layout(set = 0, binding = 2) uniform sampler PointSampler;
layout(set = 0, binding = 3) uniform sampler LinearSampler;

void main()
{
    // Sample the texture atlases
    vec3 coords = vec3(fsin_TextureUV, fsin_LayerSmooth.x);
    vec4 pixel;
    if (fsin_LayerSmooth.y > 0) {
        pixel = texture(sampler2DArray(Texture, LinearSampler), coords);
    } else {
        pixel = texture(sampler2DArray(Texture, PointSampler), coords);
    }

    // Apply basic sprite modes (based on MVW shader by ChevyRay)
    fsout_Color = 
        fsin_Mode.x * fsin_Color * pixel +   // multiply
        fsin_Mode.y * fsin_Color * pixel.a + // wash
        fsin_Mode.z * fsin_Color;             // veto

    // MSDF rendering idea - to be investigated later
    // if (fsin_Mode.w != 0) {
    //     float median = max(min(pixel.r, pixel.g), min(max(pixel.r, pixel.g), pixel.b))
    //     float distance = fsin_Mode.w * (median - 0.5);
    //     fsout_Color = fsin_Color;
    //     fsout_Color.a *= clamp(distance + 0.5, 0.0, 1.0);
    // }
}
