use std::ops::{Index, IndexMut};

use bevy::{prelude::*, render::{extract_resource::ExtractResource, render_asset::RenderAssetUsages, render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages}}};


#[derive(Resource, ExtractResource, Clone)]
pub struct MountainTexturesRaw {
    pub map: Handle<Image>,
}

#[derive(Clone)]
pub struct MountainTextures {
    pub heightmap: MountainTexture,
    pub shadowmap: MountainTexture,
}

impl MountainTextures {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            heightmap: MountainTexture::new(width, height),
            shadowmap: MountainTexture::new(width, height),
        }
    }

    pub fn into_raw(
        self,
        mut images: ResMut<Assets<Image>>,
    ) -> MountainTexturesRaw {
        let mut im = Image::new(
            Extent3d {
                width: self.heightmap.width,
                height: self.heightmap.map.len() as u32 / self.heightmap.width,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            self.heightmap.map.iter().zip(self.shadowmap.map.iter()).flat_map(|(height, shadow)| {
                let height_bits = height.to_bits();
                let shadow_bits = shadow.to_bits();
                [
                    (height_bits & 0x000000FF) as u8,
                    ((height_bits & 0x0000FF00) >> 8) as u8,
                    ((height_bits & 0x00FF0000) >> 16) as u8,
                    ((height_bits & 0xFF000000) >> 24) as u8,
                    (shadow_bits & 0x000000FF) as u8,
                    ((shadow_bits & 0x0000FF00) >> 8) as u8,
                    ((shadow_bits & 0x00FF0000) >> 16) as u8,
                    ((shadow_bits & 0xFF000000) >> 24) as u8,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ].into_iter()
            }).collect(),
            TextureFormat::Rgba32Float,
            RenderAssetUsages::RENDER_WORLD,
        );

        im.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING;

        MountainTexturesRaw { map: images.add(im) }
    }
}


#[derive(Clone, Default)]
pub struct MountainTexture{
    map: Vec<f32>,
    width: u32,
}

impl MountainTexture {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            map: vec![0.0; (width * height) as usize],
            width,
        }
    }
}

impl Index<(usize, usize)> for MountainTexture {
    type Output = f32;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.map[index.1 * self.width as usize + index.0]
    }
}

impl IndexMut<(usize, usize)> for MountainTexture {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.map[index.1 * self.width as usize + index.0]
    }
}

impl From<MountainTexture> for Image {
    fn from(val: MountainTexture) -> Self {
        let mut im = Image::new(
            Extent3d {
                width: val.width,
                height: val.map.len() as u32 / val.width,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            val.map.iter().flat_map(|v| {
                let bits = v.to_bits();
                [
                    (bits & 0x000000FF) as u8,
                    ((bits & 0x0000FF00) >> 8) as u8,
                    ((bits & 0x00FF0000) >> 16) as u8,
                    ((bits & 0xFF000000) >> 24) as u8,
                ].into_iter()
            }).collect(),
            TextureFormat::R32Float,
            RenderAssetUsages::RENDER_WORLD,
        );

        im.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING;

        im
    }
}
