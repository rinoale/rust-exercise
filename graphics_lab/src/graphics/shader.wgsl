struct Uniforms {
    vp: mat4x4<f32>, model: mat4x4<f32>, tint: vec4<f32>, eye: vec4<f32>,
    options: vec4<f32>, selected: vec4<f32>,
}
@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var checker: texture_2d<f32>;
@group(0) @binding(2) var checker_sampler: sampler;
struct Vertex {
    @location(0) position: vec4<f32>, @location(1) normal: vec4<f32>,
    @location(2) color: vec4<f32>, @location(3) uv: vec2<f32>, @location(4) extra: vec2<f32>,
}
struct Varying {
    @builtin(position) position: vec4<f32>, @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>, @location(2) color: vec4<f32>, @location(3) uv: vec2<f32>,
    @location(4) @interpolate(flat) extra: vec2<f32>,
}
@vertex fn vertex(v: Vertex) -> Varying {
    var o: Varying;
    let world = u.model * v.position;
    o.position = u.vp * world;
    o.world = world.xyz;
    o.normal = (u.model * v.normal).xyz;
    o.color = v.color;
    o.uv = v.uv;
    o.extra = v.extra;
    return o;
}
@fragment fn cube(v: Varying) -> @location(0) vec4<f32> {
    let n = normalize(v.normal);
    let light = normalize(vec3<f32>(-0.4, 0.85, 0.55));
    let diffuse = max(dot(n, light), 0.0);
    let view = normalize(u.eye.xyz - v.world);
    let specular = pow(max(dot(n, normalize(light + view)), 0.0), 48.0);
    let sample_color = textureSample(checker, checker_sampler, v.uv).rgb;
    let texel = select(vec3<f32>(1.0), sample_color, u.options.x > 0.5);
    var color = texel * u.tint.rgb * v.color.rgb * (0.22 + 0.78 * diffuse) + 0.18 * specular;
    let delta = u.eye.xyz - v.world;
    let fog = 1.0 - exp(-0.0025 * dot(delta, delta));
    color = mix(color, vec3<f32>(0.035, 0.048, 0.075), fog);
    return vec4<f32>(color, 1.0);
}
@fragment fn world(v: Varying) -> @location(0) vec4<f32> {
    let light = 0.48 + 0.52 * max(dot(v.normal, normalize(vec3<f32>(-0.5,1.0,0.3))),0.0);
    let pixel = floor(v.uv * 16.0);
    let noise = fract(sin(dot(pixel, vec2<f32>(12.9898,78.233))) * 43758.5453);
    let tile = select(v.uv, fract(v.uv), v.extra.x > 0.0);
    let edge = min(min(tile.x,1.0-tile.x),min(tile.y,1.0-tile.y));
    var color = v.color.rgb * light * (0.86 + noise * 0.22) * select(1.0,0.76,edge < 0.012);
    let cell = floor(v.world - v.normal * 0.001);
    let selected = select(u.selected.w > 0.0 && all(abs(cell-u.selected.xyz)<vec3<f32>(0.1)), abs(v.extra.x-u.selected.x)<0.1, u.selected.w > 1.5);
    if selected { color = mix(color,vec3<f32>(1.0,0.86,0.36),select(0.15,0.95,edge < 0.035)); }
    if v.extra.y > 0.0 {
        if (floor(v.position.x/3.0)+floor(v.position.y/3.0)) % 2.0 < 1.0 { discard; }
        color = v.color.rgb;
    }
    let fog = select(smoothstep(22.0,65.0,distance(v.world,u.eye.xyz)),smoothstep(60.0,150.0,distance(v.world,u.eye.xyz)),u.selected.w > 1.5);
    return vec4<f32>(mix(color,vec3<f32>(0.46,0.68,0.86),fog),1.0);
}
@fragment fn hud(v: Varying) -> @location(0) vec4<f32> { return v.color; }
