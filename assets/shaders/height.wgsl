@group(0) @binding(0)
var<uniform> settings: MountainSettings;
@group(0) @binding(1)
var map: texture_storage_2d<rgba32float, read_write>;
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
    strength: f32,
    center: vec2<f32>,

    sun_direction: vec3<f32>,
    _padding: f32,

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
};

var<private> perm: array<i32, 256> = array(
    151,160,137,91,90,15,
    131,13,201,95,96,53,194,233,7,225,140,36,103,30,69,142,8,99,37,240,21,10,23,
    190, 6,148,247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,57,177,33,
    88,237,149,56,87,174,20,125,136,171,168, 68,175,74,165,71,134,139,48,27,166,
    77,146,158,231,83,111,229,122,60,211,133,230,220,105,92,41,55,46,245,40,244,
    102,143,54, 65,25,63,161, 1,216,80,73,209,76,132,187,208, 89,18,169,200,196,
    135,130,116,188,159,86,164,100,109,198,173,186, 3,64,52,217,226,250,124,123,
    5,202,38,147,118,126,255,82,85,212,207,206,59,227,47,16,58,17,182,189,28,42,
    223,183,170,213,119,248,152, 2,44,154,163, 70,221,153,101,155,167, 43,172,9,
    129,22,39,253, 19,98,108,110,79,113,224,232,178,185, 112,104,218,246,97,228,
    251,34,242,193,238,210,144,12,191,179,162,241, 81,51,145,235,249,14,239,107,
    49,192,214, 31,181,199,106,157,184, 84,204,176,115,121,50,45,127, 4,150,254,
    138,236,205,93,222,114,67,29,24,72,243,141,128,195,78,66,215,61,156,180,
);

var<private> grad_2_lut: array<vec2<f32>, 8> = array(
    vec2(-1.0f, -1.0f), vec2(1.0f, 0.0f), vec2(-1.0f, 0.0f), vec2(1.0f, 1.0f),
    vec2(-1.0f, 1.0f), vec2(0.0f, -1.0f), vec2(0.0f, 1.0f), vec2(1.0f, -1.0f)
);

const F2: f32 = 0.366025403;
const G2: f32 = 0.211324865;

fn fastfloor(x: f32) -> i32 {
    if x > 0.0 { return i32(x); } else { return i32(x) - 1; }
}

fn grad2(hash: i32) -> vec2<f32> {
    let h = hash & 7;
    return grad_2_lut[h];
}

fn simplex(v: vec2<f32>) -> vec3<f32> {
    var n0 = 0.0;
    var n1 = 0.0;
    var n2 = 0.0;

    let s = (v.x + v.y) * F2;
    let xs = v.x + s;
    let ys = v.y + s;
    let i = fastfloor(xs);
    let j = fastfloor(ys);

    let t = f32(i + j) * G2;
    let x0 = v.x - (f32(i) - t);
    let y0 = v.y - (f32(j) - t);

    var i1 = 0; var j1 = 0;
    if x0 > y0 { i1 = 1; j1 = 0; }
    else { i1 = 0; j1 = 1; }

    let x1 = x0 - f32(i1) + G2;
    let y1 = y0 - f32(j1) + G2;
    let x2 = x0 - 1.0 + 2.0 * G2;
    let y2 = y0 - 1.0 + 2.0 * G2;

    let ii = i & 0xff;
    let jj = j & 0xff;

    var g0 = vec2(0.0);
    var g1 = vec2(0.0);
    var g2 = vec2(0.0);

    var t0 = 0.5 - x0 * x0 - y0 * y0;
    var t20 = 0.0; var t40 = 0.0;

    if t0 < 0.0 {
        t0 = 0.0;
    } else {
        g0 = grad2(perm[(ii + perm[jj]) & 0xff]);
        t20 = t0 * t0;
        t40 = t20 * t20;
        n0 = t40 * (g0.x * x0 + g0.y * y0);
    };

    var t1 = 0.5 - x1 * x1 - y1 * y1;
    var t21 = 0.0; var t41 = 0.0;

    if t1 < 0.0 {
        t1 = 0.0;
    } else {
        g1 = grad2(perm[(ii + i1 + perm[(jj + j1) & 0xff]) & 0xff]);
        t21 = t1 * t1;
        t41 = t21 * t21;
        n1 = t41 * (g1.x * x1 + g1.y * y1);
    };

    var t2 = 0.5 - x2 * x2 - y2 * y2;
    var t22 = 0.0; var t42 = 0.0;

    if t2 < 0.0 {
        t2 = 0.0;
    } else {
        g2 = grad2(perm[(ii + 1 + perm[(jj + 1) & 0xff]) & 0xff]);
        t22 = t2 * t2;
        t42 = t22 * t22;
        n2 = t42 * (g2.x * x2 + g2.y * y2);
    };

    let temp0 = t20 * t0 * (g0.x * x0 + g0.y * y0);
    var dnoise_dx = temp0 * x0;
    var dnoise_dy = temp0 * y0;
    let temp1 = t21 * t1 * (g1.x * x1 + g1.y * y1);
    dnoise_dx += temp1 * x1;
    dnoise_dy += temp1 * y1;
    let temp2 = t22 * t2 * (g2.x * x2 + g2.y * y2);
    dnoise_dx += temp2 * x2;
    dnoise_dy += temp2 * y2;
    dnoise_dx *= -8.0;
    dnoise_dy *= -8.0;
    dnoise_dx += t40 * g0.x + t41 * g1.x + t42 * g2.x;
    dnoise_dy += t40 * g0.y + t41 * g1.y + t42 * g2.y;
    dnoise_dx *= 40.0;
    dnoise_dy *= 40.0;

    return vec3(40.0 * (n0 + n1 + n2), dnoise_dx, dnoise_dy);
}

@compute @workgroup_size(8, 8, 1)
fn height(@builtin(global_invocation_id) id: vec3<u32>) {
    let uv = vec2<f32>(id.xy) / f32(settings.map_size);

    var height = 0.0;
    var f = settings.roughness;
    var amp = 1.0;
    var steepness = 0.0;
    var amount = 0.0;

    for (var i = 0u; i < settings.num_octaves; i++) {
        let noise = simplex(uv * f + settings.center);
        steepness += length(noise.yz);
        let weight = 1.0 / (1.0 + settings.sharpness * steepness);
        height += (noise.x + 1.0) * 0.5 * amp * weight;
        amount += amp * weight;
        f *= settings.lacunarity;
        amp *= settings.persistence;
    }

    height = clamp(height * settings.strength + settings.offset, 0.0, 1.0);

    textureStore(map, id.xy, vec4(height, 0.0, 0.0, 0.0));
}

@compute @workgroup_size(8, 8, 1)
fn shadow(@builtin(global_invocation_id) id: vec3<u32>) {
    let uv = vec2<f32>(id.xy) / f32(settings.map_size);
    let location = vec2<i32>(id.xy);

    let height = textureLoad(map, location).x;
    let pixel_size = 1.0 / f32(settings.map_size);
    var shadow = 0.0;
    var pos = vec3(uv.x, height, uv.y);
    var n = 0;

    for (var i = 0u; i < 128u; i++) {
        n++;

        if pos.x >= 1.0 || pos.x < 0.0 || pos.z >= 1.0 || pos.z < 0.0 {
            break;
        }

        let h = textureLoad(map, vec2<u32>(pos.xz * f32(settings.map_size))).x;
        if h > pos.y {
            shadow = 1.0;
            break;
        }

        if pos.y > 1.0 {
            break;
        }

        pos += settings.sun_direction * max((pos.y - h) * 0.05, pixel_size);
    }

    if n == 128 {
        shadow = 1.0;
    }

    textureStore(map, id.xy, vec4(height, clamp(shadow, 0.0, 1.0), 0.0, 0.0));
}
