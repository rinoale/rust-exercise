#include <metal_stdlib>
using namespace metal;

constant float3 corners[] = {
    {-1,-1,-1}, {1,-1,-1}, {1,1,-1}, {-1,1,-1},
    {-1,-1, 1}, {1,-1, 1}, {1,1, 1}, {-1,1, 1}
};
// Two triangles per face; assembly submits 36 vertices in one draw call.
constant uint indices[] = {
    0,1,2, 0,2,3,  4,6,5, 4,7,6,
    1,5,6, 1,6,2,  0,3,7, 0,7,4,
    3,2,6, 3,6,7,  0,4,5, 0,5,1
};
constant float3 colors[] = {
    {1.0,0.42,0.12}, {0.88,0.20,0.30},
    {0.12,0.55,0.95}, {0.60,0.30,0.90},
    {0.12,0.85,0.65}, {0.95,0.80,0.15}
};

struct VertexOut {
    float4 position [[position]];
    float3 color;
};

vertex VertexOut cube_vertex(uint id [[vertex_id]], constant float4 &uniforms [[buffer(0)]]) {
    float3 p = corners[indices[id]];
    float sy = sin(uniforms.x), cy = cos(uniforms.x);
    p = float3(cy*p.x + sy*p.z, p.y, -sy*p.x + cy*p.z);
    float sx = sin(0.45f), cx = cos(0.45f);
    p = float3(p.x, cx*p.y - sx*p.z, sx*p.y + cx*p.z);
    p.z += 4.8f;
    // Perspective projection with Metal's 0..1 depth range (near .1, far 100).
    float4 clip = float4(p.x * 1.9f / uniforms.y, p.y * 1.9f,
                        (100.0f / 99.9f) * p.z - (10.0f / 99.9f), p.z);
    return {clip, colors[id / 6]};
}

fragment float4 cube_fragment(VertexOut in [[stage_in]]) {
    return float4(in.color, 1.0f);
}
