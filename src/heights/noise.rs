use crate::textures::MountainTexture;
use bevy::{math::Vec2, reflect::Reflect};
use bevy_inspector_egui::InspectorOptions;


#[derive(Reflect, Clone, InspectorOptions)]
pub struct NoiseGeneratorSettings {
    pub num_octaves: u32,
    #[inspector(min = 0.0, speed = 0.05)]
    pub strength: f32,
    #[inspector(min = 0.0, speed = 0.025)]
    pub roughness: f32,
    #[inspector(min = 0.0, speed = 0.025)]
    pub lacunarity: f32,
    #[inspector(min = 0.0, max = 1.0, speed = 0.025)]
    pub persistence: f32,
    pub offset: f32,
    pub center: Vec2,
}

impl Default for NoiseGeneratorSettings {
    fn default() -> Self {
        NoiseGeneratorSettings {
            num_octaves: 8,
            strength: 40.0,
            roughness: 0.005,
            lacunarity: 5.0,
            persistence: 0.2,
            offset: 0.0,
            center: Vec2::new(0.0, 0.0),
        }
    }
}

pub fn generate_heights(
    map: &mut MountainTexture,
    settings: &NoiseGeneratorSettings,
) {
    let noise = NoiseSimplex2d::new(0);

    for (x, y, px) in map.enumerate_pixels_mut() {
        let mut noise_val = 0.0;
        let mut f = settings.roughness;
        let mut amp = 1.0;

        for _ in 0..settings.num_octaves {
            let v = noise.evaluate(Vec2::new(x as f32, y as f32) * f + settings.center);
            noise_val += (v + 1.0) * 0.5 * amp;
            f *= settings.lacunarity;
            amp *= settings.persistence;
        }

        *px = noise_val * settings.strength - settings.offset;
    }
}

#[derive(Clone)]
pub struct NoiseSimplex2d {
    pub random: [u8; Self::SIZE],
}

impl Default for NoiseSimplex2d {
    fn default() -> Self {
        Self {
            random: [0; Self::SIZE],
        }
    }
}

impl NoiseSimplex2d {
    const SOURCE: [u8; 256] = [
        151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69, 142,
        8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219, 203,
        117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175, 74, 165,
        71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230, 220, 105, 92, 41,
        55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76, 132, 187, 208, 89,
        18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173, 186, 3, 64, 52, 217, 226, 250,
        124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189,
        28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9,
        129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34,
        242, 193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31,
        181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114,
        67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180
    ];
    pub const SIZE: usize = 256;

    #[allow(clippy::excessive_precision)]
    const F2: f32 = 0.366025403; // (sqrt(3) - 1) / 2
    #[allow(clippy::excessive_precision)]
    const G2: f32 = 0.211324865; // (3 - sqrt(3)) / 6 = F2 / (1 + 2 * K)

    pub fn new(seed: u32) -> Self {
        let mut noise_gen = Self {
            random: [0; Self::SIZE]
        };
        noise_gen.randomize(seed);
        noise_gen
    }

    pub fn evaluate(&self, p: Vec2) -> f32 {
        let x = p.x;
        let y = p.y;

        let s = (x + y) * Self::F2;
        let xs = x + s;
        let ys = y + s;

        let i = xs.floor() as i128;
        let j = ys.floor() as i128;

        let t = (i + j) as f32 * Self::G2;

        let x0 = x - (i as f32 - t);
        let y0 = y - (j as f32 - t);

        let (i1, j1);

        if x0 > y0 {
            i1 = 1;
            j1 = 0;
        } else {
            i1 = 0;
            j1 = 1;
        }

        let x1 = x0 - i1 as f32 + Self::G2;
        let y1 = y0 - j1 as f32 + Self::G2;

        let x2 = x0 - 1.0 + 2.0 * Self::G2;
        let y2 = y0 - 1.0 + 2.0 * Self::G2;
        
        let gi0 = self.hash(i + self.hash(j) as i128);
        let gi1 = self.hash(i + i1 + self.hash(j + j1) as i128);
        let gi2 = self.hash(i + 1 + self.hash(j + 1) as i128);

        let mut t0 = 0.5 - x0 * x0 - y0 * y0;
        let n0 = if t0 < 0.0 {
            0.0
        } else {
            t0 *= t0;
            t0 * t0 * Self::grad(gi0, x0, y0)
        };

        let mut t1 = 0.5 - x1 * x1 - y1 * y1;
        let n1 = if t1 < 0.0 {
            0.0
        } else {
            t1 *= t1;
            t1 * t1 * Self::grad(gi1, x1, y1)
        };

        let mut t2 = 0.5 - x2 * x2 - y2 * y2;
        let n2 = if t2 < 0.0 {
            0.0
        } else {
            t2 *= t2;
            t2 * t2 * Self::grad(gi2, x2, y2)
        };

        45.23065 * (n0 + n1 + n2)
    }

    fn randomize(&mut self, seed: u32) {
        if seed != 0 {
            let bytes = Self::unpack_u32(seed);
            for i in 0..Self::SIZE {
                self.random[i] = Self::SOURCE[i] ^ bytes[0];
                self.random[i] ^= bytes[1];
                self.random[i] ^= bytes[2];
                self.random[i] ^= bytes[3];
            }
        } else {
            for i in 0..Self::SIZE {
                self.random[i] = Self::SOURCE[i];
            }
        }
    }

    fn hash(&self, i: i128) -> u8 {
        self.random[i as usize % Self::SIZE]
    }

    fn grad(hash: u8, x: f32, y: f32) -> f32 {
        let h = hash & 0x3F;
        let u = if h < 4 { x } else { y };
        let v = if h < 4 { y } else { x };
        (if h & 1 != 0 { -u } else { u }) + (if h & 2 != 0 { -2.0 * v } else { 2.0 * v })
    }

    fn unpack_u32(val: u32) -> [u8; 4] {
        [
            (val & 0x00ff) as u8,
            ((val & 0xff00) >> 8) as u8,
            ((val & 0x00ff0000) >> 16) as u8,
            ((val & 0xff000000) >> 24) as u8,
        ]
    }
}
