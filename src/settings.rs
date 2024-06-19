use bevy::{prelude::{ReflectResource, Resource}, reflect::Reflect, render::render_resource::ShaderType};

#[derive(Debug, Clone, Reflect, ShaderType)]
pub struct MountainRenderSettings {
    pub shadow_attenuation: f32,
}

impl Default for MountainRenderSettings {
    fn default() -> Self {
        Self {
            shadow_attenuation: 0.125,
        }
    }
}

#[derive(Reflect, Resource, Clone)]
#[reflect(Resource)]
pub struct MountainShadowSlope(pub f32);

impl Default for MountainShadowSlope {
    fn default() -> Self {
        MountainShadowSlope(0.2)
    }
}
