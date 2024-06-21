use std::f32::consts::PI;

use bevy::{prelude::*, render::{mesh::{Indices, PrimitiveTopology}, render_asset::RenderAssetUsages}};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use compute::{uniforms::{MountainErosionTrigger, RegenerateMountain}, MountainComputePlugin};
use material::{MountainMaterial, MountainMaterialPlugin};
use settings::MountainShadowSettings;

mod material;
mod compute;
// mod textures;
// mod heights;
mod settings;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            MountainMaterialPlugin,
            MountainComputePlugin,
        ))

        .init_resource::<MountainShadowSettings>()
        // .register_type::<MountainShadowSettings>()

        // .init_resource::<Rng>()
        // .insert_resource(MountainTextures::new(2048, 2048))

        .add_systems(Startup, setup)
        .add_systems(Update, keybinds)

        // .add_systems(Update, erode_main.run_if(in_state(AppState::Eroding)))
        // .add_systems(OnEnter(AppState::Eroding), erode_init)

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
    mut erosion_evw: EventWriter<MountainErosionTrigger>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        gen_fbm_evw.send(RegenerateMountain);
    }

    if keys.just_pressed(KeyCode::KeyE) {
        erosion_evw.send(MountainErosionTrigger::Toggle);
    }
}

// #[allow(clippy::too_many_arguments)]
// fn regenerate_on_enter(
//     mut commands: Commands,
//     mut images: ResMut<Assets<Image>>,
//     strategy: Res<GenerationStrategy>,
//     mut update_material_evw: EventWriter<UpdateMountainMaterial>,
//     shadow_settings: Res<MountainShadowSettings>,
//     keys: Res<ButtonInput<KeyCode>>,
//     mut textures: ResMut<MountainTextures>,
//     state: Res<State<AppState>>,
//     mut next_state: ResMut<NextState<AppState>>,
//     time: Res<Time>,
//     mut last_time: Local<Option<f32>>,
// ) {
//     if keys.just_pressed(KeyCode::KeyU) || (last_time.is_none() || time.elapsed_seconds() - last_time.unwrap() > 30.0) {
//         generate_shadows(textures.as_mut(), shadow_settings.as_ref());
//         generate_gradients(textures.as_mut());
//
//         commands.insert_resource(MountainTexturesRaw { map: images.add(textures.as_raw()) });
//         update_material_evw.send(UpdateMountainMaterial);
//         *last_time = Some(time.elapsed_seconds());
//     }
//
//     if keys.just_pressed(KeyCode::KeyE) {
//         match **state {
//             AppState::Main => next_state.set(AppState::Eroding),
//             AppState::Eroding => next_state.set(AppState::Main),
//         }
//     }
//
//     if keys.just_pressed(KeyCode::KeyR) {
//         next_state.set(AppState::Main);
//
//         generate_maps(
//             commands,
//             images,
//             strategy,
//             update_material_evw,
//             shadow_settings,
//             textures,
//         );
//     }
// }


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
