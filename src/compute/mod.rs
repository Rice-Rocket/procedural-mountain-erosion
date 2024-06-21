use bevy::{prelude::*, render::{
    extract_resource::ExtractResourcePlugin, render_graph::RenderGraph,
    Render, RenderApp, RenderSet,
}};
use node::{MountainComputeNode, MountainErosionStatus, MountainGenerateFBMStatus, MountainRenderLabel};
use pipeline::MountainComputePipeline;
use uniforms::{
    prepare_storage, prepare_uniforms, setup_storage, setup_textures, update_erosion_status, update_generate_fbm_status, MountainBrushIndices, MountainBrushStorage, MountainBrushWeights, MountainComputeSettings, MountainComputeTextures, MountainComputeUniforms, MountainErosionTrigger, RegenerateMountain
};

pub const TEXTURE_SIZE: u32 = 256;
pub const WORKGROUP_SIZE: u32 = 8;
pub const NUM_EROSIONS: u32 = 1024;

pub mod node;
pub mod pipeline;
pub mod uniforms;

pub struct MountainComputePlugin;

impl Plugin for MountainComputePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MountainComputeSettings>()
            .init_resource::<MountainBrushWeights>()
            .init_resource::<MountainBrushIndices>()
            .init_resource::<MountainGenerateFBMStatus>()
            .init_resource::<MountainErosionStatus>()
            .add_event::<RegenerateMountain>()
            .add_event::<MountainErosionTrigger>()
            .add_systems(Startup, setup_textures)
            .add_systems(Update, (update_generate_fbm_status, update_erosion_status))
            .add_plugins((
                ExtractResourcePlugin::<MountainComputeSettings>::default(),
                ExtractResourcePlugin::<MountainBrushWeights>::default(),
                ExtractResourcePlugin::<MountainBrushIndices>::default(),
                ExtractResourcePlugin::<MountainComputeTextures>::default(),
                ExtractResourcePlugin::<MountainGenerateFBMStatus>::default(),
                ExtractResourcePlugin::<MountainErosionStatus>::default(),
            ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<MountainComputeUniforms>()
            .init_resource::<MountainBrushStorage>()
            .add_systems(Startup, setup_textures)
            .add_systems(Render, (prepare_uniforms, prepare_storage).in_set(RenderSet::Prepare));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(MountainRenderLabel, MountainComputeNode::default());
        render_graph.add_node_edges((
            MountainRenderLabel,
            bevy::render::graph::CameraDriverLabel,
        ));
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<MountainComputePipeline>();
    }
}
