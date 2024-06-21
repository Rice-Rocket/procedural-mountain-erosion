use bevy::{math::Vec3, prelude::{ReflectResource, Resource}, reflect::Reflect, render::render_resource::ShaderType};

use crate::material::MountainMaterial;

#[derive(Debug, Clone, Reflect, ShaderType)]
pub struct MountainRenderSettings {
    pub sun_direction: Vec3,
    pub terrain_height: f32,
    pub blend_sharpness: f32,
    pub pixel_size: f32,
}

impl Default for MountainRenderSettings {
    fn default() -> Self {
        Self {
            sun_direction: Vec3::ZERO,
            terrain_height: 60.0,
            blend_sharpness: 10.0,
            pixel_size: 0.0,
        }
    }
}

#[derive(Debug, Reflect, Clone, Copy, ShaderType)]
pub struct ColorEntry {
    pub color: [f32; 3],
    pub elevation: f32,
    _padding: [f32; 3],
    pub steepness: f32,
}

impl ColorEntry {
    #[inline]
    pub const fn new(color: [f32; 3], elevation: f32, steepness: f32) -> Self {
        Self { color, elevation, steepness, _padding: [0.0; 3] }
    }
}

pub const MOUNTAIN_COLORS: [ColorEntry; 7] =  [
    ColorEntry::new([0.757, 0.663, 0.538], 0.1, 0.2), // dirt
    ColorEntry::new([0.596, 0.678, 0.353], 0.3, 0.15), // grass
    ColorEntry::new([0.396, 0.522, 0.255], 0.5, 0.15), // bush
    ColorEntry::new([0.278, 0.463, 0.271], 0.7, 0.2), // forest
    ColorEntry::new([0.427, 0.463, 0.529], 0.85, 0.3), // stone
    ColorEntry::new([0.518, 0.553, 0.604], 0.9, 0.2), // slate
    ColorEntry::new([0.824, 0.878, 0.871], 0.99, 0.0), // snow
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
