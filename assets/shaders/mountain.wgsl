#import bevy_pbr::{
    mesh_functions,
    skinning,
    view_transformations::position_world_to_clip,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::prepass_io::{Vertex, VertexOutput}
#else
#import bevy_pbr::forward_io::{Vertex, VertexOutput}
#endif

@group(2) @binding(0)
var<uniform> settings: MountainRenderSettings;
@group(2) @binding(1)
var map: texture_2d<f32>;
@group(2) @binding(2)
var map_sampler: sampler;
@group(2) @binding(3)
var<storage, read> colors: array<ColorEntry>;

struct ColorEntry {
    color: vec4<f32>,
    elevation: f32,
    steepness: f32,
    _padding: vec2<f32>,
}

struct MountainRenderSettings {
    sun_direction: vec3<f32>,
    terrain_height: f32,
    blend_sharpness: f32,
    pixel_size: f32,

    normal_strength: f32,
    erosion_radius: i32,
}


@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let uv = vertex.uv;
    let sample = textureSampleLevel(map, map_sampler, uv, 0.0);
    let height = sample.x;

#ifdef SKINNED
    var model = skinning::skin_model(vertex.joint_indices, vertex.joint_weights);
#else
    var model = mesh_functions::get_model_matrix(vertex.instance_index);
#endif

    let position = vertex.position + vec3(0.0, height * settings.terrain_height, 0.0);
    var out: VertexOutput;

    out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);

    out.uv = vertex.uv;

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = vertex.instance_index;
#endif

    out.world_normal = vec3(0.0, 1.0, 0.0);

    return out;
}

fn hash(p: vec2<f32>) -> f32 {
    var q = vec2<u32>(p);
    q *= vec2(1597334677u, 3812015801u);
    var n = q.x ^ q.y;
    n = n * (n ^ (n >> 15));
    return f32(n) * (1.0 / f32(0xffffffffu));
}

fn terrain_color(h: f32, normal: vec3<f32>) -> vec3<f32> {
    let steepness = 1.0 - dot(normal, vec3(0.0, 1.0, 0.0));
    let pos = vec2(h, steepness);
    var col = vec3(0.0);
    var amount = 0.0;

    for (var i = 0u; i < 7; i++) {
        let entry = colors[i];
        let position = vec2(entry.elevation, entry.steepness);
        let dist = distance(pos, position);
        let weight = 1.0 / pow(dist, settings.blend_sharpness);

        col += weight * entry.color.rgb * entry.color.a;
        amount += weight * entry.color.a;
    }

    col /= amount;

    return col;
}

fn gradient(uv: vec2<f32>) -> vec2<f32> {
    let north = textureSample(map, map_sampler, uv + vec2(0.0, settings.pixel_size)).x;
    let south = textureSample(map, map_sampler, uv + vec2(0.0, -settings.pixel_size)).x;
    let east = textureSample(map, map_sampler, uv + vec2(settings.pixel_size, 0.0)).x;
    let west = textureSample(map, map_sampler, uv + vec2(-settings.pixel_size, 0.0)).x;

    return vec2((east - west) / (2.0 * settings.pixel_size), (south - north) / (2.0 * settings.pixel_size));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let map_size = 1.0 / settings.pixel_size;
    let coord = uv * map_size;
    let sample = textureSample(map, map_sampler, uv);
    let terrain_height = sample.x;
    var shadow = sample.y;

    if coord.x >= map_size - f32(settings.erosion_radius)
    || coord.y >= map_size - f32(settings.erosion_radius)
    || coord.x < f32(settings.erosion_radius) || coord.y < f32(settings.erosion_radius) {
        discard;
    }

    let grad = gradient(uv);
    let normal = normalize(vec3(grad.x, 1.0, grad.y));
    let sun = normalize(settings.sun_direction * vec3(1.0, settings.normal_strength, 1.0));

    shadow = max(shadow, max(dot(normal, sun) * 0.5 + 0.5, 0.0));
    shadow = min(shadow, 0.9);

    var col = terrain_color(terrain_height, normal);
    col = mix(col, vec3(0.0), shadow);

    return vec4(col, 1.0);
}
