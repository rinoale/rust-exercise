#include <metal_stdlib>
using namespace metal;
struct Vertex { float4 position; float4 normal; float4 color; float2 uv; float2 padding; };
struct Uniforms { float4x4 view_projection; float4 eye; float4 selected; };
struct Varying { float4 position [[position]]; float3 world; float3 normal; float4 color; float2 uv; float piece [[flat]]; float preview [[flat]]; };
vertex Varying world_vertex(uint id [[vertex_id]], const device Vertex* vertices [[buffer(0)]], constant Uniforms& u [[buffer(1)]]) {
    Vertex v = vertices[id];
    return { u.view_projection * v.position, v.position.xyz, v.normal.xyz, v.color, v.uv, v.padding.x, v.padding.y };
}
fragment float4 world_fragment(Varying v [[stage_in]], constant Uniforms& u [[buffer(1)]]) {
    float light = 0.48 + 0.52 * max(dot(v.normal, normalize(float3(-0.5,1.0,0.3))),0.0);
    float2 pixel = floor(v.uv * 16.0);
    float noise = fract(sin(dot(pixel, float2(12.9898,78.233))) * 43758.5453);
    float2 tile = v.piece > 0.0 ? fract(v.uv) : v.uv;
    float edge = min(min(tile.x,1.0-tile.x),min(tile.y,1.0-tile.y));
    float3 color = v.color.rgb * light * (0.86 + noise * 0.22) * (edge < 0.012 ? 0.76 : 1.0);
    float3 cell = floor(v.world - v.normal * 0.001);
    bool selected = u.selected.w > 1.5 ? abs(v.piece-u.selected.x) < 0.1
        : u.selected.w > 0.0 && all(abs(cell-u.selected.xyz)<0.1);
    if (selected) {
        color = mix(color, float3(1.0,0.86,0.36), edge < 0.035 ? 0.95 : 0.15);
    }
    if (v.preview > 0.0) {
        if (fmod(floor(v.position.x/3.0)+floor(v.position.y/3.0),2.0)<1.0) discard_fragment();
        color = v.color.rgb;
    }
    float fog = u.selected.w > 1.5
        ? smoothstep(60.0,150.0,distance(v.world,u.eye.xyz))
        : smoothstep(22.0,65.0,distance(v.world,u.eye.xyz));
    return float4(mix(color,float3(0.46,0.68,0.86),fog),1.0);
}
vertex Varying hud_vertex(uint id [[vertex_id]], const device Vertex* vertices [[buffer(0)]]) {
    Vertex v = vertices[id];
    return { v.position, float3(0), float3(0), v.color, v.uv, 0.0, 0.0 };
}
fragment float4 hud_fragment(Varying v [[stage_in]]) { return v.color; }
