#include <metal_stdlib>
using namespace metal;

// TODO: Could we make some macros to auto generate the vertex_descriptor from this struct?
struct Instance {
    float3 start     [[attribute(0)]];
    float3 end       [[attribute(1)]];
    float3 color     [[attribute(2)]];
    float  thickness [[attribute(3)]];
    uint   style     [[attribute(4)]];
};

struct VertexOut {
    float4 position [[position]];
    float4 color;
};

struct Uniform {
    float4x4 camera;
    float3 camera_position;
};

vertex VertexOut vertex_main(Instance inst [[stage_in]],
                             uint vid [[vertex_id]],
                             const device float2 *geometry [[buffer(0)]],
                             constant Uniform &uniform [[buffer(2)]])
{
    float3 origin = (inst.start + inst.end) / 2.0;
    float size = length(inst.start - inst.end);
    float2 vert = geometry[vid + (4*inst.style)] * (float2(size, inst.thickness)/2.0);

    // Construct a plane facing the camera
    float3 to_camera = uniform.camera_position - origin;
    float3 u = normalize(inst.start - origin);
    float3 v = normalize(cross(u, to_camera));
    float3 pos = (u * vert.x) + (v * vert.y);

    VertexOut out;
    out.position = uniform.camera * float4(origin + pos, 1.0);
    out.color = float4(inst.color, 1.0);
    return out;
}

fragment float4 fragment_main(VertexOut inst [[stage_in]]) {
    return inst.color;
}
