#include <metal_stdlib>
using namespace metal;
struct Vertex { float4 position; float4 normal; float4 color; float2 uv; float2 padding; };
struct Uniforms { float4x4 view_projection; float4 eye; float4 selected; };
struct Varying { float4 position [[position]]; float3 world; float3 normal; float4 color; float2 uv; };
vertex Varying world_vertex(uint id [[vertex_id]], const device Vertex* vertices [[buffer(0)]], constant Uniforms& u [[buffer(1)]]) {
    Vertex v = vertices[id];
    return { u.view_projection * v.position, v.position.xyz, v.normal.xyz, v.color, v.uv };
}
fragment float4 world_fragment(Varying v [[stage_in]], constant Uniforms& u [[buffer(1)]]) {
    float light = 0.48 + 0.52 * max(dot(v.normal, normalize(float3(-0.5,1.0,0.3))),0.0);
    float2 pixel = floor(v.uv * 16.0);
    float noise = fract(sin(dot(pixel, float2(12.9898,78.233))) * 43758.5453);
    float edge = min(min(v.uv.x,1.0-v.uv.x),min(v.uv.y,1.0-v.uv.y));
    float3 color = v.color.rgb * light * (0.86 + noise * 0.22) * (edge < 0.012 ? 0.76 : 1.0);
    float3 cell = floor(v.world - v.normal * 0.001);
    if (u.selected.w > 0.0 && all(abs(cell - u.selected.xyz) < 0.1)) {
        color = mix(color, float3(1.0,0.86,0.36), edge < 0.035 ? 0.95 : 0.15);
    }
    float fog = smoothstep(22.0,65.0,distance(v.world,u.eye.xyz));
    return float4(mix(color,float3(0.46,0.68,0.86),fog),1.0);
}
vertex Varying hud_vertex(uint id [[vertex_id]], const device Vertex* vertices [[buffer(0)]]) {
    Vertex v = vertices[id];
    return { v.position, float3(0), float3(0), v.color, v.uv };
}
fragment float4 hud_fragment(Varying v [[stage_in]]) { return v.color; }
