use crate::textures::MountainTexture;
use bevy::{math::Vec2, reflect::Reflect};
use bevy_inspector_egui::InspectorOptions;


#[derive(Reflect, Clone, InspectorOptions)]
pub struct NoiseGeneratorSettings {
    pub num_octaves: u32,
    #[inspector(min = 0.0, speed = 0.025)]
    pub roughness: f32,
    #[inspector(min = 0.0, speed = 0.025)]
    pub lacunarity: f32,
    #[inspector(min = 0.0, max = 1.0, speed = 0.025)]
    pub persistence: f32,
    #[inspector(min = 0.0)]
    pub sharpness: f32,
    pub offset: f32,
    pub center: Vec2,
}

impl Default for NoiseGeneratorSettings {
    fn default() -> Self {
        NoiseGeneratorSettings {
            num_octaves: 8,
            roughness: 1.4,
            lacunarity: 5.0,
            persistence: 0.2,
            sharpness: 0.15,
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
    let width = map.width();
    let height = map.height();
    let mut min = f32::MAX;
    let mut max = f32::MIN;

    for (x, y, px) in map.enumerate_pixels_mut() {
        let uv = Vec2::new(x as f32 / width as f32, y as f32 / height as f32);
        let mut noise_val = 0.0;
        let mut f = settings.roughness;
        let mut amp = 1.0;
        let mut steepness = 0.0;

        for _ in 0..settings.num_octaves {
            let (v, grad) = noise.evaluate(uv * f + settings.center);
            steepness += grad.length();
            let weight = 1.0 / (1.0 + settings.sharpness * steepness);
            noise_val += (v + 1.0) * 0.5 * amp * weight;
            f *= settings.lacunarity;
            amp *= settings.persistence;
        }

        *px = noise_val - settings.offset;
        min = min.min(*px);
        max = max.max(*px);
    }

    for px in map.pixels_mut() {
        *px = (*px - min) / (max - min);
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

    const GRAD_2_LUT: [[f32; 2]; 8] = [
		[-1.0, -1.0], [1.0, 0.0], [-1.0, 0.0], [1.0, 1.0],
		[-1.0, 1.0], [0.0, -1.0], [0.0, 1.0], [1.0, -1.0]
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

    pub fn evaluate(&self, p: Vec2) -> (f32, Vec2) {
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

        let (mut gx0, mut gy0, mut gx1, mut gy1, mut gx2, mut gy2) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

        let mut t0 = 0.5 - x0 * x0 - y0 * y0;
        let (mut t20, mut t40) = (0.0, 0.0);
        let n0 = if t0 < 0.0 {
            t0 = 0.0;
            0.0
        } else {
            (gx0, gy0) = Self::grad2(gi0);
            t20 = t0 * t0;
            t40 = t20 * t20;
            t40 * (gx0 * x0 + gy0 * y0)
        };

        let mut t1 = 0.5 - x1 * x1 - y1 * y1;
        let (mut t21, mut t41) = (0.0, 0.0);
        let n1 = if t1 < 0.0 {
            t1 = 0.0;
            0.0
        } else {
            (gx1, gy1) = Self::grad2(gi1);
            t21 = t1 * t1;
            t41 = t21 * t21;
            t41 * (gx1 * x1 + gy1 * y1)
        };

        let mut t2 = 0.5 - x2 * x2 - y2 * y2;
        let (mut t22, mut t42) = (0.0, 0.0);
        let n2 = if t2 < 0.0 {
            t2 = 0.0;
            0.0
        } else {
            (gx2, gy2) = Self::grad2(gi2);
            t22 = t2 * t2;
            t42 = t22 * t22;
            t42 * (gx2 * x2 + gy2 * y2)
        };

        let temp0 = t20 * t0 * (gx0 * x0 + gy0 * y0);
        let mut dnoise_dx = temp0 * x0;
        let mut dnoise_dy = temp0 * y0;

        let temp1 = t21 * t1 * (gx1 * x1 + gy1 * y1);
        dnoise_dx += temp1 * x1;
        dnoise_dy += temp1 * y1;

        let temp2 = t22 * t2 * (gx2 * x2 + gy2 * y2);
        dnoise_dx += temp2 * x2;
        dnoise_dy += temp2 * y2;

        dnoise_dx *= -8.0;
        dnoise_dy *= -8.0;

        dnoise_dx += t40 * gx0 + t41 * gx1 + t42 * gx2;
        dnoise_dy += t40 * gy0 + t41 * gy1 + t42 * gy2;

        dnoise_dx *= 40.0;
        dnoise_dy *= 40.0;

        (40.0 * (n0 + n1 + n2), Vec2::new(dnoise_dx, dnoise_dy))
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

    fn grad2(hash: u8) -> (f32, f32) {
        let h = hash & 7;
        (Self::GRAD_2_LUT[h as usize][0], Self::GRAD_2_LUT[h as usize][1])
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
