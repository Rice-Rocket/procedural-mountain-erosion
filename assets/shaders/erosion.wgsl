@group(0) @binding(0)
var<uniform> settings: MountainSettings;
@group(0) @binding(1)
var map: texture_storage_2d_array<rgba32float, read_write>;
@group(0) @binding(2)
var<storage, read> brush_indices: array<i32, 64>;
@group(0) @binding(3)
var<storage, read> brush_weights: array<f32, 64>;

struct MountainSettings {
    map_size: u32,

    num_octaves: u32,
    roughness: f32,
    lacunarity: f32,
    persistence: f32,
    sharpness: f32,
    offset: f32,
    center: vec2<f32>,

    max_lifetime: u32,
    erosion_radius: i32,
    inertia: f32,
    sediment_capacity_factor: f32,
    min_sediment_capacity: f32,
    erode_speed: f32,
    deposit_speed: f32,
    evaporation_speed: f32,
    gravity: f32,
    start_speed: f32,
    start_water: f32,
}

@compute @workgroup_size(1, 1024, 1)
fn erode(@builtin(global_invocation_id) id: vec3<u32>) {

}
