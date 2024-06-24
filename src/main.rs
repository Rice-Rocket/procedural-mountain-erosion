use std::f32::consts::PI;

use bevy::{prelude::*, render::{mesh::{Indices, PrimitiveTopology}, render_asset::RenderAssetUsages}, window::PresentMode};
use bevy_inspector_egui::quick::{AssetInspectorPlugin, ResourceInspectorPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};
use compute::{uniforms::{MountainComputeSettings, MountainErosionTrigger, RegenerateMountain, RegenerateShadows}, MountainComputePlugin};
use material::{MountainMaterial, MountainMaterialPlugin};

mod material;
mod compute;
mod settings;

fn main() {
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
        ))

        .add_systems(Startup, setup)
        .add_systems(Update, keybinds)

        .run();
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

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 20_000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, 0.0, -0.1, 0.0)),
        ..default()
    });
}

fn keybinds(
    keys: Res<ButtonInput<KeyCode>>,
    mut gen_fbm_evw: EventWriter<RegenerateMountain>,
    mut gen_shadow_evw: EventWriter<RegenerateShadows>,
    mut erosion_evw: EventWriter<MountainErosionTrigger>,
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
