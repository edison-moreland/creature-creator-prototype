#include <metal_stdlib>
using namespace metal;

struct VertexIn {
    float4 position [[attribute(0)]];
    float4 center   [[attribute(1)]];
    float radius    [[attribute(2)]];
    float4 color    [[attribute(3)]];
};

struct VertexOut {
    float4 position [[position]];
    float4 color;
};

struct Uniform {
    float4x4 projection;
    float4x4 view;
};

vertex VertexOut vertex_main(VertexIn in [[stage_in]],
                             constant Uniform &uniform [[buffer(2)]])
{
    VertexOut out;
    out.position = (uniform.projection * uniform.view) * float4(in.center.xyz + in.radius * in.position.xyz, 1.0f);
    out.color = in.color;
    return out;
}

fragment float4 fragment_main(VertexOut in [[stage_in]]) {
    return in.color;
}
