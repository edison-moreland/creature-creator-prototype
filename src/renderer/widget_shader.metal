#include <metal_stdlib>
using namespace metal;

// TODO: Could we make some macros to auto generate the vertex_descriptor from this struct?
struct VertexIn {
    // Vertex position
    float2 position [[attribute(0)]];

    // Instance stuff...
    float3 start     [[attribute(1)]];
    float3 end       [[attribute(2)]];
    float3 color     [[attribute(3)]];
    float  thickness [[attribute(4)]];
};

struct VertexOut {
    float4 position [[position]];
    float4 color;
};

struct Uniform {
    float4x4 camera;
    float3 camera_position;
};

vertex VertexOut vertex_main(VertexIn in [[stage_in]],
                             constant Uniform &uniform [[buffer(2)]])
{
    float3 size = length(in.start - in.end);
    float3 origin = (in.start + in.end) / 2.0;
    float3 to_camera = uniform.camera_position - origin;

    // Construct a plane facing the camera
    float3 u = normalize(in.start - origin);
    float3 v = normalize(cross(u, to_camera));
    float3 pos = (u * in.position.x * (size/2.0)) + (v * in.position.y * (in.thickness / 2.0));

    VertexOut out;
    out.position = uniform.camera * float4(origin + pos, 1.0);
    out.color = float4(in.color, 1.0);
    return out;
}

fragment float4 fragment_main(VertexOut in [[stage_in]]) {
    return in.color;
}
