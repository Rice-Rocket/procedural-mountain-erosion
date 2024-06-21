use bevy::{prelude::*, render::{
    render_resource::{
        BindGroupLayout, BindGroupLayoutEntry, BindingType, BufferBindingType,
        CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache, ShaderStages,
        ShaderType as _, StorageTextureAccess, TextureFormat, TextureViewDimension,
    },
    renderer::RenderDevice,
}};

use super::uniforms::{MountainBrushIndices, MountainBrushWeights, MountainComputeSettings};

#[derive(Resource)]
pub struct MountainComputePipeline {
    pub layout: BindGroupLayout,

    pub fbm_pipeline: CachedComputePipelineId,
    pub shadow_pipeline: CachedComputePipelineId,
    pub erosion_pipeline: CachedComputePipelineId,
}

impl FromWorld for MountainComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            None,
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(MountainComputeSettings::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(MountainBrushIndices::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(MountainBrushWeights::min_size()),
                    },
                    count: None,
                },
            ]
        );

        let asset_server = world.resource::<AssetServer>();
        let height_shader = asset_server.load("shaders/height.wgsl");
        let erosion_shader = asset_server.load("shaders/erosion.wgsl");

        let pipeline_cache = world.resource::<PipelineCache>();

        let fbm_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: height_shader.clone(),
            shader_defs: vec![],
            entry_point: "height".into(),
        });

        let shadow_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: height_shader.clone(),
            shader_defs: vec![],
            entry_point: "shadow".into(),
        });
        
        let erosion_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: erosion_shader.clone(),
            shader_defs: vec![],
            entry_point: "erode".into(),
        });

        MountainComputePipeline {
            layout,
            fbm_pipeline,
            shadow_pipeline,
            erosion_pipeline,
        }
    }
}
