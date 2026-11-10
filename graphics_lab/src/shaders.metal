#include <metal_stdlib>
using namespace metal;

struct Vertex {
    float4 position;
    float4 normal;
    float2 uv;
    float2 padding;
};

struct Uniforms {
    float4x4 view_projection;
    float4x4 model;
    float4 tint;
    float4 eye;
    float4 options;
};

struct Varyings {
    float4 position [[position]];
    float3 world_position;
    float3 normal;
    float2 uv;
};

vertex Varyings cube_vertex(uint id [[vertex_id]],
                           const device Vertex *vertices [[buffer(0)]],
                           constant Uniforms &u [[buffer(1)]]) {
    Vertex v = vertices[id];
    Varyings out;
    float4 world = u.model * v.position;
    out.position = u.view_projection * world;
    out.world_position = world.xyz;
    // Scene transforms have only rotation and uniform scale.
    out.normal = normalize((u.model * v.normal).xyz);
    out.uv = v.uv;
    return out;
}

fragment float4 cube_fragment(Varyings in [[stage_in]],
                              constant Uniforms &u [[buffer(1)]],
                              texture2d<float> checker [[texture(0)]],
                              sampler checker_sampler [[sampler(0)]]) {
    float3 n = normalize(in.normal);
    float3 light = normalize(float3(-0.4, 0.85, 0.55));
    float diffuse = max(dot(n, light), 0.0);
    float3 view = normalize(u.eye.xyz - in.world_position);
    float specular = pow(max(dot(n, normalize(light + view)), 0.0), 48.0);
    float3 texel = u.options.x > 0.5 ? checker.sample(checker_sampler, in.uv).rgb : float3(1.0);
    float3 color = texel * u.tint.rgb * (0.22 + 0.78 * diffuse) + 0.18 * specular;
    float fog = 1.0 - exp(-0.0025 * dot(u.eye.xyz - in.world_position, u.eye.xyz - in.world_position));
    color = mix(color, float3(0.035, 0.048, 0.075), fog);
    return float4(color, 1.0);
}
