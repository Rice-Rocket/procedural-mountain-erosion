use bevy::prelude::*;

use crate::{material::UpdateMountainMaterial, textures::{MountainTexture, MountainTextures}};

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
) {
    let mut textures = MountainTextures::new(256, 256);

    match *strategy {
        GenerationStrategy::Noise => noise::generate_heights(&mut textures.heightmap)
    }

    generate_shadows(&textures.heightmap, &mut textures.shadowmap);

    commands.insert_resource(textures.into_raw(images));
    update_material_evw.send(UpdateMountainMaterial);
}


const SHADOW_SLOPE: f32 = 0.1;

fn generate_shadows(
    heights: &MountainTexture,
    shadows: &mut MountainTexture,
) {
    for y in 0..heights.height() as usize {
        shadows[(0, y)] = heights[(0, y)] - SHADOW_SLOPE;
    }

    for x in 1..heights.width() as usize {
        for y in 0..heights.height() as usize {
            let left_height = heights[(x - 1, y)];
            let left_shadow = shadows[(x - 1, y)];
            shadows[(x, y)] = left_height.max(left_shadow) - SHADOW_SLOPE;
        }
    }
}
