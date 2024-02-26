#version 450

layout(location = 0) in vec2 Position;

layout(location = 0) out vec2 fsin_TextureUV;
layout(location = 1) flat out vec2 fsin_LayerSmooth;
layout(location = 2) flat out vec4 fsin_Color;
layout(location = 3) flat out vec4 fsin_Mode;

struct Instance 
{
    vec4 Color;       // contains color to tint sprite
    vec4 Mode;        // xyz contains draw mode options, w currently unused
    vec2 LayerSmooth; // x contains texture array layer, y contains smooth filtering option
    vec2 SourceTrans; // contains UV translation
    vec2 SourceScale; // contains UV scale
    vec2 WorldCol1;   // world matrix 2x2 1st col
    vec2 WorldCol2;   // world matrix 2x2 2nd col
    vec2 WorldTrans;  // world matrix translation
};

layout(push_constant) uniform CameraBlock
{
    mat4 ProjectionView;
};

layout(std430, set = 0, binding = 0) readonly buffer InstanceBuffer
{
    Instance Instances[];
};

void main()
{
    Instance instance = Instances[gl_InstanceIndex];
    
    mat4 world = mat4(
        vec4(instance.WorldCol1, 0, 0),
        vec4(instance.WorldCol2, 0, 0),
        vec4(0, 0, 1, 0),
        vec4(instance.WorldTrans, 0, 1)
    );

    gl_Position = ProjectionView * world * vec4(Position, 0, 1);

    fsin_TextureUV = Position * instance.SourceScale + instance.SourceTrans;
    fsin_LayerSmooth = instance.LayerSmooth;
    fsin_Color = instance.Color;
    fsin_Mode = instance.Mode;
}
