use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_asset::RenderAssets,
        render_graph::{self, RenderLabel},
        render_resource::{BindGroupEntry, BindingResource, ComputePassDescriptor, PipelineCache},
    },
};

use super::{pipeline::MountainComputePipeline, uniforms::{MountainBrushStorage, MountainComputeTextures, MountainComputeUniforms}, NUM_EROSIONS, TEXTURE_SIZE, WORKGROUP_SIZE};

#[derive(Resource, ExtractResource, Default, Clone, Copy)]
pub enum MountainGenerateFBMStatus {
    #[default]
    Update,
    Wait
}

#[derive(Resource, ExtractResource, Default, Clone, Copy, PartialEq, Eq)]
pub enum MountainErosionStatus {
    #[default]
    Update,
    Wait
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct MountainRenderLabel;

#[derive(Default)]
pub struct MountainComputeNode {
    generate_fbm: bool,
    enable_erosion: bool,
}

impl render_graph::Node for MountainComputeNode {
    fn update(&mut self, world: &mut World) {
        let erosion_status = world.resource::<MountainErosionStatus>();
        self.enable_erosion = *erosion_status == MountainErosionStatus::Update;

        let mut fbm_status = world.resource_mut::<MountainGenerateFBMStatus>();

        let fbm_trigger = match *fbm_status {
            MountainGenerateFBMStatus::Update => true,
            MountainGenerateFBMStatus::Wait => false,
        };

        if fbm_trigger && self.generate_fbm {
            *fbm_status = MountainGenerateFBMStatus::Wait;
            self.generate_fbm = false;
            return;
        }

        if fbm_trigger && !self.generate_fbm {
            self.generate_fbm = true;
            return;
        }

        self.generate_fbm = false;
    }

    fn run<'w>(
        &self,
        graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let compute_pipelines = world.resource::<MountainComputePipeline>();

        let gpu_images = world.resource::<RenderAssets<Image>>();
        let mountain_textures = world.resource::<MountainComputeTextures>();

        let uniforms = world.resource::<MountainComputeUniforms>();
        let brush_storage = world.resource::<MountainBrushStorage>();
        
        let map = &gpu_images.get(&mountain_textures.map).unwrap();

        let bind_group = render_context
            .render_device()
            .create_bind_group(
                Some("mountain_compute_pass_bind_group"),
                &compute_pipelines.layout,
                &[
                    BindGroupEntry {
                        binding: 0,
                        resource: uniforms.buf.binding().unwrap(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&map.texture_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: brush_storage.indices.binding().unwrap(),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: brush_storage.weights.binding().unwrap(),
                    },
                ]
            );

        let encoder = render_context.command_encoder();

        if self.generate_fbm {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, &bind_group, &[]);

            let Some(pipeline) = pipeline_cache.get_compute_pipeline(compute_pipelines.fbm_pipeline) else {
                return Ok(());
            };

            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(TEXTURE_SIZE / WORKGROUP_SIZE, TEXTURE_SIZE / WORKGROUP_SIZE, 1);
        }

        if self.enable_erosion {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, &bind_group, &[]);

            let Some(pipeline) = pipeline_cache.get_compute_pipeline(compute_pipelines.erosion_pipeline) else {
                return Ok(());
            };

            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(NUM_EROSIONS, 1, 1);
        }

        Ok(())
    }
}
