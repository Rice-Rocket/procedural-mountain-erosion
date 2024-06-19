use bevy::prelude::*;

use crate::{material::UpdateMountainMaterial, settings::MountainShadowSlope, textures::{MountainTexture, MountainTextures}};

pub mod noise;

#[derive(Resource, Clone, Default)]
pub enum GenerationStrategy {
    #[default]
    Noise,
}

pub fn generate_maps(
    mut commands: Commands,
    images: ResMut<Assets<Image>>,
    strategy: Res<GenerationStrategy>,
    mut update_material_evw: EventWriter<UpdateMountainMaterial>,
    shadow_slope: Res<MountainShadowSlope>,
) {
    let mut textures = MountainTextures::new(256, 256);

    match *strategy {
        GenerationStrategy::Noise => noise::generate_heights(&mut textures.heightmap)
    }

    generate_gradients(&textures.heightmap, &mut textures.gradients_x, &mut textures.gradients_y);
    generate_shadows(&textures.heightmap, &mut textures.shadowmap, shadow_slope.0);

    commands.insert_resource(textures.into_raw(images));
    update_material_evw.send(UpdateMountainMaterial);
}

fn generate_gradients(
    heights: &MountainTexture,
    gradients_x: &mut MountainTexture,
    gradients_y: &mut MountainTexture,
) {
    for ((x, y, gx), gy) in gradients_x.enumerate_pixels_mut().zip(gradients_y.pixels_mut()) {
        let height = heights[(x, y)];
        let north = if y + 1 < heights.height() { heights[(x, y + 1)] } else { height };
        let south = if y >= 1 { heights[(x, y - 1)] } else { height };
        let east = if x + 1 < heights.width() { heights[(x + 1, y)] } else { height };
        let west = if x >= 1 { heights[(x - 1, y)] } else { height };

        *gx = (east - west) / 2.0;
        *gy = (south - north) / 2.0;
    }
}

fn generate_shadows(
    heights: &MountainTexture,
    shadows: &mut MountainTexture,
    shadow_slope: f32,
) {
    for y in 0..heights.height() {
        shadows[(0, y)] = heights[(0, y)] - shadow_slope;
    }

    for x in 1..heights.width() {
        for y in 0..heights.height() {
            let left_height = heights[(x - 1, y)];
            let left_shadow = shadows[(x - 1, y)];
            shadows[(x, y)] = left_height.max(left_shadow) - shadow_slope;
        }
    }
}
