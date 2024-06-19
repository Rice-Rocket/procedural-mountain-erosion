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
var map: texture_2d<f32>;
@group(2) @binding(1)
var map_sampler: sampler;


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

    let position = vertex.position + vec3(0.0, height, 0.0);
    var out: VertexOutput;

    out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);

#ifdef VERTEX_UVS
#ifdef SKINNED
    out.world_normal = skinning::skin_normals(model, vertex.normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
#endif
#endif

    out.uv = vertex.uv;

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(model, vertex.tangent, vertex.instance_index);
#endif

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
    let shadow_height = sample.y;

    var col = vec3(1.0);

    col = mix(col, vec3(0.0), f32(shadow_height > terrain_height) * (shadow_height - terrain_height) * 0.1);
    // col = mix(col, vec3(1.0, 0.0, 0.0), f32(shadow_height > terrain_height));
    // col = vec3(terrain_height - shadow_height);

    return vec4(col, 1.0);
}
