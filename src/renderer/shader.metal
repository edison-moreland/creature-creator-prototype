#include <metal_stdlib>
using namespace metal;

struct VertexIn {
    float4 position [[attribute(0)]];
    float4 center   [[attribute(1)]];
    float radius    [[attribute(2)]];
    float4 normal    [[attribute(3)]];
};

struct VertexOut {
    float4 position [[position]];
    float4 color;
};

struct Uniform {
    float4x4 camera;
};

vertex VertexOut vertex_main(VertexIn in [[stage_in]],
                             constant Uniform &uniform [[buffer(2)]])
{
    float radius = in.radius;
    float4 origin = in.center;
    //float4 origin = in.center - (in.normal * radius);

    // TODO: better lighting

    VertexOut out;
    out.position = uniform.camera * float4(origin.xyz + radius * in.position.xyz, 1.0f);
    out.color = abs(in.normal);
    return out;
}

fragment float4 fragment_main(VertexOut in [[stage_in]]) {
    return in.color;
}
