use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_asset::RenderAssetUsages,
        render_resource::{
            Extent3d, FilterMode, SamplerDescriptor, ShaderType, StorageBuffer,
            TextureDimension, TextureFormat, TextureUsages, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::ImageSampler,
    },
};

use super::{node::{MountainErosionStatus, MountainGenerateFBMStatus, MountainGenerateShadowStatus, MountainPrepareWriteStatus}, TEXTURE_SIZE};

pub const EROSION_RADIUS: i32 = 3;
// NOTE: Make sure to change value in shader if this is changed.
pub const BRUSH_STORAGE_LENGTH: u32 = 64; // Actually 49 (2 * EROSION_RADIUS + 1) ^ 2

#[derive(Clone, Resource, ExtractResource, Reflect, ShaderType)]
#[reflect(Resource)]
pub struct MountainComputeSettings {
    pub map_size: u32,

    pub num_octaves: u32,
    pub roughness: f32,
    pub lacunarity: f32,
    pub persistence: f32,
    pub sharpness: f32,
    pub offset: f32,
    pub strength: f32,
    pub center: Vec2,

    time: f32,
    brush_length: u32,

    pub sun_direction: Vec3,

    pub max_lifetime: u32,
    pub erosion_radius: i32,
    pub inertia: f32,
    pub sediment_capacity_factor: f32,
    pub min_sediment_capacity: f32,
    pub erode_speed: f32,
    pub deposit_speed: f32,
    pub evaporation_speed: f32,
    pub gravity: f32,
    pub start_speed: f32,
    pub start_water: f32,
    
    _padding: Vec2,
}

impl Default for  MountainComputeSettings {
    fn default() -> Self {
        Self {
            map_size: TEXTURE_SIZE,

            // num_octaves: 4,
            // roughness: 1.4,
            // lacunarity: 5.0,
            // persistence: 0.2,
            // sharpness: 0.0,
            // offset: 0.0,
            // strength: 1.0,
            // center: Vec2::new(0.0, 0.0),

            num_octaves: 4,
            roughness: 1.3,
            lacunarity: 3.0,
            persistence: 0.15,
            sharpness: 0.0,
            offset: 0.0,
            strength: 1.0,
            center: Vec2::new(0.5, -0.5),

            time: 0.0,
            brush_length: BRUSH_STORAGE_LENGTH,

            sun_direction: Vec3::new(1.0, 4.0, 0.5).normalize(),

            max_lifetime: 30,
            erosion_radius: EROSION_RADIUS,
            inertia: 0.3,
            sediment_capacity_factor: 3.0,
            min_sediment_capacity: 0.01,
            erode_speed: 0.3,
            deposit_speed: 0.3,
            evaporation_speed: 0.01,
            gravity: 4.0,
            start_speed: 1.0,
            start_water: 1.0,

            _padding: Vec2::ZERO,
        }
    }
}

#[derive(Resource, Default)]
pub struct MountainComputeUniforms {
    pub buf: UniformBuffer<MountainComputeSettings>,
}

pub fn prepare_uniforms(
    mut uniforms: ResMut<MountainComputeUniforms>,
    general_settings: Res<MountainComputeSettings>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    time: Res<Time>,
) {
    let general = uniforms.buf.get_mut();
    *general = general_settings.clone();

    general.time = time.elapsed_seconds();
    if !general.sun_direction.is_normalized() {
        general.sun_direction = general.sun_direction.normalize();
    }

    uniforms.buf.write_buffer(&render_device, &render_queue);
}

#[derive(Event)]
pub struct RegenerateMountain;

pub fn update_generate_fbm_status(
    mut evr: EventReader<RegenerateMountain>,
    mut status: ResMut<MountainGenerateFBMStatus>,
) {
    for _ev in evr.read() {
        *status = MountainGenerateFBMStatus::Update;
    }
}

#[derive(Event)]
pub struct RegenerateShadows;

pub fn update_generate_shadow_status(
    mut evr: EventReader<RegenerateShadows>,
    mut status: ResMut<MountainGenerateShadowStatus>,
) {
    for _ev in evr.read() {
        *status = MountainGenerateShadowStatus::Update;
    }
}

#[derive(Event)]
pub struct PrepareWriteCompute;

pub fn update_prepare_write_status(
    mut evr: EventReader<PrepareWriteCompute>,
    mut status: ResMut<MountainPrepareWriteStatus>,
) {
    for _ev in evr.read() {
        *status = MountainPrepareWriteStatus::Update;
    }
}

#[allow(dead_code)]
#[derive(Event)]
pub enum MountainErosionTrigger {
    Start,
    Stop,
    Toggle
}

pub fn update_erosion_status(
    mut evr: EventReader<MountainErosionTrigger>,
    mut status: ResMut<MountainErosionStatus>,
) {
    for ev in evr.read() {
        match ev {
            MountainErosionTrigger::Start => *status = MountainErosionStatus::Update,
            MountainErosionTrigger::Stop => *status = MountainErosionStatus::Wait,
            MountainErosionTrigger::Toggle => if *status == MountainErosionStatus::Wait {
                *status = MountainErosionStatus::Update
            } else {
                *status = MountainErosionStatus::Wait
            }
        }
    }
}


#[derive(Resource, ExtractResource, Clone)]
pub struct MountainComputeTextures {
    pub map: Handle<Image>
}

pub fn setup_textures(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let extent = Extent3d {
        width: TEXTURE_SIZE,
        height: TEXTURE_SIZE,
        depth_or_array_layers: 1,
    };

    let mut im = Image::new_fill(
        extent,
        TextureDimension::D2,
        &[0; 16],
        TextureFormat::Rgba32Float,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );

    im.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING 
        | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC;
    im.sampler = ImageSampler::Descriptor(SamplerDescriptor {
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..default()
    }.into());

    commands.insert_resource(MountainComputeTextures {
        map: images.add(im),
    });
}

#[derive(Resource, ShaderType, ExtractResource, Reflect, Clone)]
#[reflect(Resource)]
pub struct MountainBrushWeights{ pub weights: [f32; BRUSH_STORAGE_LENGTH as usize] }

#[derive(Resource, ShaderType, ExtractResource, Reflect, Clone)]
#[reflect(Resource)]
pub struct MountainBrushIndices{ pub indices: [[i32; 2]; BRUSH_STORAGE_LENGTH as usize] }

impl Default for MountainBrushWeights { fn default() -> Self { Self { weights: [0.0; BRUSH_STORAGE_LENGTH as usize] } } }
impl Default for MountainBrushIndices { fn default() -> Self { Self { indices: [[0; 2]; BRUSH_STORAGE_LENGTH as usize] } } }

#[derive(Resource, Default)]
pub struct MountainBrushStorage {
    pub weights: StorageBuffer<MountainBrushWeights>,
    pub indices: StorageBuffer<MountainBrushIndices>,
}

pub fn setup_storage(
    mut weights_storage: ResMut<MountainBrushWeights>,
    mut indices_storage: ResMut<MountainBrushIndices>,
) {
    let mut weights = [0.0; BRUSH_STORAGE_LENGTH as usize];
    let mut indices = [[0; 2]; BRUSH_STORAGE_LENGTH as usize];

    let mut weight_sum = 0.0;
    let mut i = 0;

    for brush_y in -EROSION_RADIUS..=EROSION_RADIUS {
        for brush_x in -EROSION_RADIUS..=EROSION_RADIUS {
            let sqr_dst = brush_x * brush_x + brush_y * brush_y;
            if sqr_dst < EROSION_RADIUS * EROSION_RADIUS {
                indices[i] = [brush_x, brush_y];
                let brush_weight = 1.0 - (sqr_dst as f32).sqrt() / EROSION_RADIUS as f32;
                weight_sum += brush_weight;
                weights[i] = brush_weight;
                i += 1;
            }
        }
    }

    for w in weights.iter_mut() {
        *w /= weight_sum;
    }

    weights_storage.weights = weights;
    indices_storage.indices = indices;

    // let storage_weights = storage.weights.get_mut();
    // storage_weights.weights = weights;
    //
    // let storage_indices = storage.indices.get_mut();
    // storage_indices.indices = indices;
}

pub fn prepare_storage(
    mut storage: ResMut<MountainBrushStorage>,
    storage_weights: Res<MountainBrushWeights>,
    storage_indices: Res<MountainBrushIndices>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let weights = storage.weights.get_mut();
    *weights = storage_weights.clone();

    let indices = storage.indices.get_mut();
    *indices = storage_indices.clone();

    storage.weights.write_buffer(&render_device, &render_queue);
    storage.indices.write_buffer(&render_device, &render_queue);
}
