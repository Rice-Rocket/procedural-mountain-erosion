@group(0) @binding(0)
var<uniform> settings: MountainSettings;
@group(0) @binding(1)
var map: texture_storage_2d<rgba32float, read_write>;
@group(0) @binding(2)
var<storage, read> brush_indices: array<vec2<i32>, 64>;
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
    strength: f32,
    center: vec2<f32>,

    time: f32,
    brush_length: u32,

    sun_direction: vec3<f32>,

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

    _padding: vec2<f32>,
}

// https://www.shadertoy.com/view/4djSRW
fn hash23(p3: vec3<f32>) -> vec2<f32> {
    var p = fract(p3 * vec3(0.1031, 0.1030, 0.0973));
    p += dot(p, p.yzx + 33.33);
    return fract((p.xx + p.yz) * p.zy);
}

fn get_height_gradient(pos: vec2<f32>) -> vec3<f32> {
    let coord = vec2<i32>(pos);
    let p = pos - vec2<f32>(coord);

    let height_nw = textureLoad(map, coord).x;
    let height_ne = textureLoad(map, coord + vec2(1, 0)).x;
    let height_sw = textureLoad(map, coord + vec2(0, 1)).x;
    let height_se = textureLoad(map, coord + vec2(1, 1)).x;

    let gx = (height_ne - height_nw) * (1.0 - p.y) + (height_se - height_sw) * p.y;
    let gy = (height_sw - height_nw) * (1.0 - p.x) + (height_se - height_ne) * p.x;
    
    let height = height_nw * (1.0 - p.x) * (1.0 - p.y) + height_ne * p.x * (1.0 - p.y) 
        + height_sw * (1.0 - p.x) * p.y + height_se * p.x * p.y;

    return vec3(height, gx, gy);
}

@compute @workgroup_size(1, 64, 1)
fn erode(@builtin(global_invocation_id) id: vec3<u32>) {
    var pos = hash23(vec3(vec2<f32>(id.xy), settings.time)) * f32(settings.map_size); // + f32(settings.erosion_radius);
    var dir = vec2(0.0);
    var speed = settings.start_speed;
    var water = settings.start_water;
    var sediment = 0.0;

    for (var lifetime = 0u; lifetime < settings.max_lifetime; lifetime++) {
        let node = vec2<i32>(pos);

        let cell_offset = pos - vec2<f32>(node);
        let height_gradient = get_height_gradient(pos);

        dir = (dir * settings.inertia - height_gradient.yz * (1.0 - settings.inertia));

        let len = max(0.01, length(dir));
        dir /= len;

        pos += dir;

        if (dir.x == 0.0 && dir.y == 0.0) || pos.x < f32(settings.erosion_radius) 
        || pos.x >= f32(settings.map_size) - f32(settings.erosion_radius) || pos.y < f32(settings.erosion_radius)
        || pos.y >= f32(settings.map_size) - f32(settings.erosion_radius) {
            break;
        }

        let new_height = get_height_gradient(pos).x;
        let delta_height = new_height - height_gradient.x;

        let sediment_capacity = max(-delta_height * speed * water * settings.sediment_capacity_factor, settings.min_sediment_capacity);

        if sediment > sediment_capacity || delta_height > 0.0 {
            let amount = mix((sediment - sediment_capacity) * settings.deposit_speed, min(delta_height, sediment), f32(delta_height > 0.0));
            sediment -= amount;

            let n1 = textureLoad(map, node);
            let n2 = textureLoad(map, node + vec2(1, 0));
            let n3 = textureLoad(map, node + vec2(0, 1));
            let n4 = textureLoad(map, node + vec2(1, 1));

            // storageBarrier();

            textureStore(map, node, vec4(n1.x + amount * (1.0 - cell_offset.x) * (1.0 - cell_offset.y), n1.yzw));
            textureStore(map, node + vec2(1, 0), vec4(n2.x + amount * cell_offset.x * (1.0 - cell_offset.y), n2.yzw));
            textureStore(map, node + vec2(0, 1), vec4(n3.x + amount * (1.0 - cell_offset.x) * cell_offset.y, n3.yzw));
            textureStore(map, node + vec2(1, 1), vec4(n4.x + amount * cell_offset.x * cell_offset.y, n4.yzw));
        } else {
            let amount = min((sediment_capacity - sediment) * settings.erode_speed, -delta_height);

            for (var i = 0u; i < settings.brush_length; i++) {
                let brush_pos = brush_indices[i];
                let erode_pos = node + brush_pos;

                let weighted_amount = amount * brush_weights[i];

                let h = textureLoad(map, erode_pos);
                // let delta_sediment = mix(weighted_amount, h, f32(h < weighted_amount))
                let delta_sediment = min(weighted_amount, h.x);

                // storageBarrier();

                textureStore(map, erode_pos, vec4(h.x - delta_sediment, h.yzw));
                sediment += delta_sediment;
            }
        }

        speed = sqrt(max(0.0, speed * speed + delta_height * settings.gravity));
        water *= (1.0 - settings.evaporation_speed);
    }
}
