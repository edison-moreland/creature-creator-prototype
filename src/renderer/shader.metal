#include <metal_stdlib>
using namespace metal;

#define backgroundColor float4(1.0, 1.0, 1.0, 0.0)
#define pinkColor float4(0.839, 0.007, 0.497, 0.0)
#define purpleColor float4(0.607, 0.309, 0.588, 0.0)
#define blueColor float4(0.0, 0.219, 0.658, 0.0)


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

float4 lambert_contribution(
    float4 object_normal,
    float4 object_origin,
    float  object_k,
    float4 light_origin,
    float4 light_color,
    float  light_intensity
) {
    return light_color * light_intensity * object_k * dot(object_normal, normalize(object_origin-light_origin));
}

float4 light_sample(
    float4 sample_point,
    float4 sample_normal
) {
    return (0.3 * backgroundColor) +
    lambert_contribution(
        sample_normal,
        sample_point,
        0.5,
        float4(-50.0, -50.0, -50.0, 0.0),
        pinkColor,
        1.00
    ) +
    lambert_contribution(
        sample_normal,
        sample_point,
        0.5,
        float4(0.0, -50.0, -50.0, 0.0),
        purpleColor,
        1.00
    ) +
    lambert_contribution(
        sample_normal,
        sample_point,
        0.5,
        float4(-50.0, -50.0, 0.0, 0.0),
        blueColor,
        1.00
    );
}

vertex VertexOut vertex_main(VertexIn in [[stage_in]],
                             constant Uniform &uniform [[buffer(2)]])
{
    float radius = in.radius * 2;
    float4 origin = in.center - (in.normal * radius);

    VertexOut out;
    out.position = uniform.camera * float4(origin.xyz + radius * in.position.xyz, 1.0f);
    out.color = float4(light_sample(origin, in.normal).xyz, 1.0);
    return out;
}

fragment float4 fragment_main(VertexOut in [[stage_in]]) {
    return in.color;
}
