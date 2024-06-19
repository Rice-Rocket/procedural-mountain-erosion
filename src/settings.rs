use bevy::{math::Vec3, prelude::{ReflectResource, Resource}, reflect::Reflect, render::render_resource::ShaderType};

#[derive(Debug, Clone, Reflect, ShaderType)]
pub struct MountainRenderSettings {
    pub sun_direction: Vec3,
    pub terrain_height: f32,
}

impl Default for MountainRenderSettings {
    fn default() -> Self {
        Self {
            sun_direction: Vec3::ZERO,
            terrain_height: 50.0,
        }
    }
}

#[derive(Reflect, Resource, Clone)]
#[reflect(Resource)]
pub struct MountainShadowSettings{
    pub sun_direction: Vec3,
}

impl Default for MountainShadowSettings {
    fn default() -> Self {
        Self {
            sun_direction: Vec3::new(1.0, 0.3, 1.0).normalize(),
        }
    }
}
