use bevy::prelude::*;
use noise::NoiseGeneratorSettings;

use crate::{material::UpdateMountainMaterial, settings::MountainShadowSettings, textures::{MountainTextures, MountainTexturesRaw}};

pub mod erode;
pub mod noise;

#[derive(Reflect, Resource, Clone)]
#[reflect(Resource)]
pub enum GenerationStrategy {
    Noise(NoiseGeneratorSettings),
}

impl Default for GenerationStrategy {
    fn default() -> Self {
        Self::Noise(NoiseGeneratorSettings::default())
    }
}

pub fn generate_maps(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    strategy: Res<GenerationStrategy>,
    mut update_material_evw: EventWriter<UpdateMountainMaterial>,
    shadow_settings: Res<MountainShadowSettings>,
    mut textures: ResMut<MountainTextures>,
) {
    match *strategy {
        GenerationStrategy::Noise(ref settings) => noise::generate_heights(&mut textures.heightmap, settings),
    }

    generate_gradients(textures.as_mut());
    generate_shadows(textures.as_mut(), shadow_settings.as_ref());

    commands.insert_resource(MountainTexturesRaw { map: images.add(textures.as_raw()) });
    update_material_evw.send(UpdateMountainMaterial);
}

pub fn generate_gradients(
    textures: &mut MountainTextures,
) {
    let dx = 2.0 / textures.heightmap.width() as f32;
    let dy = 2.0 / textures.heightmap.height() as f32;

    for x in 0..textures.heightmap.width() {
        for y in 0..textures.heightmap.height() {
            let height = textures.heightmap[(x, y)];
            let north = if y + 1 < textures.heightmap.height() { textures.heightmap[(x, y + 1)] } else { height };
            let south = if y >= 1 { textures.heightmap[(x, y - 1)] } else { height };
            let east = if x + 1 < textures.heightmap.width() { textures.heightmap[(x + 1, y)] } else { height };
            let west = if x >= 1 { textures.heightmap[(x - 1, y)] } else { height };

            textures.gradients_x[(x, y)] = (east - west) / dx;
            textures.gradients_y[(x, y)] = (south - north) / dy;
        }
    }
}

pub fn generate_shadows(
    textures: &mut MountainTextures,
    shadow_settings: &MountainShadowSettings,
) {
    let (width, height) = (textures.heightmap.width(), textures.heightmap.height());
    let step_dir = shadow_settings.sun_direction.normalize();

    for x in 0..width {
        for y in 0..height {
            let mut res = 0f32;
            let mut pos = Vec3::new(x as f32 / width as f32, textures.heightmap[(x, y)], y as f32 / height as f32);
            let mut n = 0;

            for _ in 0..128 {
                n += 1;

                if pos.x >= 1.0 || pos.x < 0.0 || pos.z >= 1.0 || pos.z < 0.0 {
                    break;
                }

                let env = textures.heightmap[((pos.x * width as f32).floor() as usize, (pos.z * height as f32).floor() as usize)];
                if env > pos.y {
                    res = 1.0;
                    break;
                }

                if pos.y > 1.0 {
                    break;
                }

                pos += step_dir * ((pos.y - env) * 0.05).max(1.0 / width as f32);
            }

            if n == 128 {
                res = 1.0;
            }

            textures.shadowmap[(x, y)] = res.clamp(0.0, 1.0);
        }
    }
}
