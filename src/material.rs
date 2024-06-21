use bevy::{asset::load_internal_asset, prelude::*, render::render_resource::AsBindGroup};

use crate::{compute::uniforms::MountainComputeTextures, settings::{ColorEntry, MountainRenderSettings, MountainShadowSettings}};

pub const MOUNTAIN_MATERIAL_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x243e54999439800056177abc27c63000);


#[derive(AsBindGroup, Debug, Reflect, Clone, Asset)]
#[reflect(Debug, Default)]
pub struct MountainMaterial {
    #[uniform(0, visibility(vertex, fragment))]
    pub settings: MountainRenderSettings,

    #[texture(1, visibility(vertex, fragment), dimension = "2d")]
    #[sampler(2)]
    pub map: Option<Handle<Image>>,

    #[storage(3, visibility(fragment), read_only)]
    pub colors: [ColorEntry; 7],
}

impl Material for MountainMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/mountain.wgsl".into()
    }

    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/mountain.wgsl".into()
    }
}

#[derive(Event)]
pub struct UpdateMountainMaterial;

pub fn prepare_mountain_material(
    handles: Query<&Handle<MountainMaterial>>,
    mut materials: ResMut<Assets<MountainMaterial>>,
    mountain_textures: Res<MountainComputeTextures>,
    mut updates_evr: EventReader<UpdateMountainMaterial>,
    shadow_settings: Res<MountainShadowSettings>,
) {
    for handle in handles.iter() {
        let mat = materials.get_mut(handle).unwrap();

        if updates_evr.read().next().is_some() {
            mat.settings.sun_direction = shadow_settings.sun_direction.normalize();
        }

        if mat.map.is_none() {
            mat.map = Some(mountain_textures.map.clone())
        }
    }
}


pub struct MountainMaterialPlugin;

impl Plugin for MountainMaterialPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            MOUNTAIN_MATERIAL_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/mountain.wgsl"),
            Shader::from_wgsl
        );

        app
            .add_plugins(MaterialPlugin::<MountainMaterial>::default())
            .add_systems(Update, prepare_mountain_material)
            .add_event::<UpdateMountainMaterial>()
            .register_type::<MountainMaterial>()
            .register_asset_reflect::<MountainMaterial>()
            .register_type::<Handle<MountainMaterial>>();
    }
}
