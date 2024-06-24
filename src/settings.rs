use bevy::{math::Vec3, reflect::Reflect, render::render_resource::ShaderType};

use crate::{compute::uniforms::EROSION_RADIUS, material::MountainMaterial};

#[derive(Debug, Clone, Reflect, ShaderType)]
pub struct MountainRenderSettings {
    pub sun_direction: Vec3,
    pub terrain_height: f32,
    pub blend_sharpness: f32,
    pub pixel_size: f32,

    pub normal_strength: f32,
    pub erosion_radius: i32,
}

impl Default for MountainRenderSettings {
    fn default() -> Self {
        Self {
            sun_direction: Vec3::ZERO,
            terrain_height: 60.0,
            blend_sharpness: 10.0,
            pixel_size: 0.0,
            
            normal_strength: 0.1,
            erosion_radius: EROSION_RADIUS,
        }
    }
}

#[derive(Debug, Reflect, Clone, Copy, ShaderType)]
pub struct ColorEntry {
    pub color: [f32; 4],
    pub elevation: f32,
    pub steepness: f32,
    _padding: [f32; 2],
}

impl ColorEntry {
    #[inline]
    pub const fn new(color: [f32; 4], elevation: f32, steepness: f32) -> Self {
        Self { color, elevation, steepness, _padding: [0.0; 2] }
    }
}

pub const MOUNTAIN_COLORS: [ColorEntry; 7] =  [
    ColorEntry::new([0.52511, 0.33674, 0.22867, 0.0], 0.2, 0.2), // dirt
    ColorEntry::new([0.522, 0.698, 0.349, 1.0], 0.25, 0.05), // grass
    ColorEntry::new([0.396, 0.522, 0.255, 1.0], 0.5, 0.15), // bush
    ColorEntry::new([0.278, 0.463, 0.271, 1.0], 0.8, 0.1), // forest
    ColorEntry::new([0.427, 0.463, 0.529, 1.0], 0.3, 0.8), // stone
    ColorEntry::new([0.518, 0.553, 0.604, 1.0], 0.9, 0.25), // slate
    ColorEntry::new([0.824, 0.878, 0.871, 1.0], 0.99, 0.3), // snow
];


impl Default for MountainMaterial {
    fn default() -> Self {
        Self {
            settings: MountainRenderSettings::default(),
            map: None,
            colors: MOUNTAIN_COLORS,
        }
    }
}
