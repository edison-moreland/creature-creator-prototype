#include <metal_stdlib>

using namespace metal;

// https://coolors.co/d60270-9b4f96-0038a8-302b27-f5f3f5
#define backgroundColor float3(0.960, 0.991, 0.960)
#define foregroundColor float3(0.188, 0.168, 0.152)
#define pinkColor float3(0.839, 0.007, 0.497)
#define purpleColor float3(0.607, 0.309, 0.588)
#define blueColor float3(0.0, 0.219, 0.658)


struct VertexIn {
    float3 position [[attribute(0)]];
    float3 center   [[attribute(1)]];
    float radius    [[attribute(2)]];
    float3 normal    [[attribute(3)]];
};

struct VertexOut {
    float4 position [[position]];
    float4 color;
};

struct Uniform {
    float4x4 camera;
    float3 camera_position;
};

float3 lambert_contribution(float3 object_normal,
                            float3 object_origin,
                            float object_k,
                            float3 light_origin,
                            float3 light_color,
                            float light_intensity)
{
    return light_color * light_intensity * object_k * dot(object_normal, normalize(object_origin - light_origin));
}

float3 light_sample(float3 sample_point,
                    float3 sample_normal)
{
    return (0.3 * backgroundColor) +
    lambert_contribution(
                         sample_normal,
                         sample_point,
                         0.5,
                         float3(-50.0, -50.0, -50.0),
                         pinkColor,
                         0.50
                         ) +
    lambert_contribution(
                         sample_normal,
                         sample_point,
                         0.5,
                         float3(0.0, -50.0, -50.0),
                         purpleColor,
                         0.75
                         ) +
    lambert_contribution(
                         sample_normal,
                         sample_point,
                         0.5,
                         float3(-50.0, -50.0, 0.0),
                         blueColor,
                         1.00
                         );
}

vertex VertexOut
vertex_main(VertexIn in [[stage_in]],
            constant Uniform &uniform [[buffer(2)]])
{
    float radius = in.radius * 2;
    
    float3 sphere_center = (in.center - (in.normal * radius));
    
    VertexOut out;
    out.position = uniform.camera * float4(sphere_center + radius * in.position, 1.0);
    out.color = float4(light_sample(sphere_center, in.normal), 1.0);
    return
    out;
}

fragment float4
fragment_main(VertexOut in [[stage_in]]) {
    return in.color;
}
