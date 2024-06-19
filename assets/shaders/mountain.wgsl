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

struct MountainRenderSettings {
    sun_direction: vec3<f32>,
    terrain_height: f32,
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

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let sample = textureSample(map, map_sampler, uv);
    let terrain_height = sample.x;
    var shadow = sample.y;

    let gradient = sample.zw;
    let steepness = length(gradient);
    let normal = normalize(vec3(gradient.x, 1.0, gradient.y));

    shadow = max(shadow, max(dot(normal, settings.sun_direction), 0.0));
    shadow = min(shadow, 0.9);

    var col = vec3(1.0);

    col = mix(col, vec3(0.0), shadow);

    return vec4(col, 1.0);
}
