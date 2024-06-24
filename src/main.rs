use std::f32::consts::PI;

use bevy::{prelude::*, render::{mesh::{Indices, PrimitiveTopology}, render_asset::RenderAssetUsages}, window::PresentMode};
use bevy_image_export::{ImageExportBundle, ImageExportPlugin, ImageExportSettings, ImageExportSource};
use bevy_inspector_egui::quick::{AssetInspectorPlugin, ResourceInspectorPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};
use compute::{uniforms::{MountainComputeSettings, MountainComputeTextures, MountainErosionTrigger, PrepareWriteCompute, RegenerateMountain, RegenerateShadows}, MountainComputePlugin};
use material::{MountainMaterial, MountainMaterialPlugin};

mod material;
mod compute;
mod settings;

fn main() {
    let export_plugin = ImageExportPlugin::default();
    let export_threads = export_plugin.threads.clone();
    
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            PanOrbitCameraPlugin,
            ScreenDiagnosticsPlugin::default(),
            ScreenFrameDiagnosticsPlugin,
            MountainMaterialPlugin,
            MountainComputePlugin,
            ResourceInspectorPlugin::<MountainComputeSettings>::default(),
            AssetInspectorPlugin::<MountainMaterial>::default(),
            export_plugin,
        ))

        .add_systems(Startup, setup)
        .add_systems(Update, keybinds)

        .run();

    export_threads.finish();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<MountainMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            camera_3d: Camera3d {
                ..default()
            },
            transform: Transform::from_xyz(-10.0, 5.0, -100.0)
                .with_rotation(Quat::from_euler(EulerRot::YXZ, -PI, 0.0, PI)),
            ..default()
        },
        PanOrbitCamera::default(),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(create_mountain_plane()),
            material: materials.add(MountainMaterial::default()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
    ));

}

#[allow(clippy::too_many_arguments)]
fn keybinds(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut gen_fbm_evw: EventWriter<RegenerateMountain>,
    mut gen_shadow_evw: EventWriter<RegenerateShadows>,
    mut erosion_evw: EventWriter<MountainErosionTrigger>,
    mut prepare_write_evw: EventWriter<PrepareWriteCompute>,
    mut prepared_write: Local<bool>,
    mut export_sources: ResMut<Assets<ImageExportSource>>,
    compute_textures: Res<MountainComputeTextures>,
    image_exports: Query<Entity, With<ImageExportSettings>>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        gen_fbm_evw.send(RegenerateMountain);
    }

    if keys.just_pressed(KeyCode::KeyS) {
        gen_shadow_evw.send(RegenerateShadows);
    }

    if keys.just_pressed(KeyCode::KeyE) {
        erosion_evw.send(MountainErosionTrigger::Toggle);
    }

    for entity in image_exports.iter() {
        commands.entity(entity).despawn();
    }

    if *prepared_write {
        *prepared_write = false;
        commands.spawn(ImageExportBundle {
            source: export_sources.add(compute_textures.map.clone()),
            settings: ImageExportSettings {
                output_dir: "heightmaps".into(),
                extension: "exr".into(),
            }
        });
    }

    if keys.just_pressed(KeyCode::KeyW) {
        prepare_write_evw.send(PrepareWriteCompute);
        *prepared_write = true;
    }
}

const PLANE_LENGTH: f32 = 256.0;
const PLANE_RES: usize = 8;

fn create_mountain_plane() -> Mesh {
    let half_length = PLANE_LENGTH * 0.5;
    let side_vert_count = PLANE_LENGTH as usize * PLANE_RES;

    let mut positions = vec![];
    let mut uvs = vec![];

    for x in 0..=side_vert_count {
        for z in 0..=side_vert_count {
            positions.push(Vec3::new(
                (x as f32 / side_vert_count as f32 * PLANE_LENGTH) - half_length,
                0.0,
                (z as f32 / side_vert_count as f32 * PLANE_LENGTH) - half_length,
            ));
            uvs.push(Vec2::new(x as f32 / side_vert_count as f32, z as f32 / side_vert_count as f32));
        }
    }

    let mut indices = vec![0u32; side_vert_count * side_vert_count * 6];

    let mut vi = 0u32;
    for _x in 0..side_vert_count {
        for _z in 0..side_vert_count {
            indices.push(vi);
            indices.push(vi + 1);
            indices.push(vi + side_vert_count as u32 + 2);
            indices.push(vi);
            indices.push(vi + side_vert_count as u32 + 2);
            indices.push(vi + side_vert_count as u32 + 1);
            vi += 1;
        }
        vi += 1;
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}
