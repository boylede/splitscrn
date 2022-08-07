use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::camera::{ScalingMode, Viewport},
    window::WindowResized,
};

use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_picking::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(set_camera_viewports)
        .add_system(bevy::window::close_on_esc)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(DebugCursorPickingPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    windows: Res<Windows>,
) {
    // add a number of spheres to the scene
    let half_width: isize = 2;
    let subdivisions: usize = 4;

    let mesh_handle = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.4,
        subdivisions,
    }));

    let matl_handle = materials.add(StandardMaterial {
        perceptual_roughness: 0.5,
        metallic: 0.6,
        base_color: Color::hsla(0.0, 0.0, 0.3, 1.0),
        ..Default::default()
    });

    for x in -half_width..half_width {
        for y in -half_width..half_width {
            for z in -half_width..half_width {
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: mesh_handle.clone(),
                        material: matl_handle.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            x as f32 + 0.35,
                            y as f32 - 1.0,
                            z as f32,
                        )),
                        ..Default::default()
                    })
                    .insert_bundle(PickableBundle::default());
            }
        }
    }

    // add a big cube to the screen

    // let mesh_handle = meshes.add(Mesh::from(shape::Cube {
    //     size: 5.0,
    // }));

    // let matl_handle = materials.add(StandardMaterial {
    //     perceptual_roughness: 0.5,
    //     metallic: 0.6,
    //     base_color: Color::hsla(0.3, 0.5, 0.3, 1.0),
    //     ..Default::default()
    // });

    // commands
    //     .spawn_bundle(PbrBundle {
    //         mesh: mesh_handle.clone(),
    //         material: matl_handle.clone(),
    //         // transform: Transform::from_translation(Vec3::new(
    //         //     x as f32 + 0.35,
    //         //     y as f32 - 1.0,
    //         //     z as f32,
    //         // )),
    //         ..Default::default()
    //     })
    //     .insert_bundle(PickableBundle::default());

    // Light
    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            1.0,
            -std::f32::consts::FRAC_PI_4,
        )),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    // set up a wall of cameras
    let (width, height) = {
        let window = windows.get_primary().unwrap();
        (window.physical_width(), window.physical_height())
    };
    let rows = 2;
    let cols = 2;

    commands.insert_resource(SplitscreenConfig { rows, cols });

    let vp_w = width / cols;
    let vp_h = height / rows;

    for i in 0..(rows * cols) {
        let row = i / cols;
        let col = i % cols;
        let ox = col * vp_w;
        let oy = row * vp_h;
        let ccc = if i == 0 {
            ClearColorConfig::Default
        } else {
            ClearColorConfig::None
        };

        let x = row as f32 * 4.0;
        let y = col as f32 * 4.0;
        let z = 2 as f32 * 4.0;

        let mut camera = Camera3dBundle {
            transform: Transform::from_xyz(x, y, z)
                .looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                viewport: Some(Viewport {
                    physical_position: UVec2::new(ox, oy),
                    physical_size: UVec2::new(vp_w, vp_h),
                    ..default()
                }),
                priority: i as isize,
                ..default()
            },
            camera_3d: Camera3d {
                clear_color: ccc,
                ..default()
            },
            projection:
            PerspectiveProjection {
                fov: 0.75 * (i as f32 / 8.0).min(1.0),
                ..default()
            }
            //  OrthographicProjection {
            //     scale: (i as f32 * 0.25) + 1.0,
            //     scaling_mode: ScalingMode::FixedVertical(3.0),
            //     ..default()
            // }
            .into(),
            ..default()
        };

        camera.transform = Transform::from_xyz(0.0, 20.0, 0.0).looking_at(Vec3::ZERO, -Vec3::Z);

        commands
            .spawn_bundle(camera)
            .insert_bundle(PickingCameraBundle::default())
            .insert(SplitscreenCamera { row, col });
    }
}

/// specifies the row and column a camera belongs to,
/// for regenerating the viewport on window resize
#[derive(Component)]
struct SplitscreenCamera {
    row: u32,
    col: u32,
}

/// a resource specifying the number of rows and columns
/// for use in regenerating the viewports during window
/// resize events
struct SplitscreenConfig {
    rows: u32,
    cols: u32,
}

/// resize the viewports when window is resized
fn set_camera_viewports(
    windows: Res<Windows>,
    mut resize_events: EventReader<WindowResized>,
    mut cameras: Query<(&mut Camera, &SplitscreenCamera)>,
    config: Res<SplitscreenConfig>,
) {
    for resize_event in resize_events.iter() {
        let (width, height) = {
            let window = windows.get(resize_event.id).unwrap();
            (window.physical_width(), window.physical_height())
        };
        let SplitscreenConfig { rows, cols } = *config;

        let vp_w = width / cols;
        let vp_h = height / rows;

        for (mut camera, splitscreen) in cameras.iter_mut() {
            let SplitscreenCamera { row, col } = splitscreen;
            let ox = col * vp_w;
            let oy = row * vp_h;
            camera.viewport = Some(Viewport {
                physical_position: UVec2::new(ox, oy),
                physical_size: UVec2::new(vp_w, vp_h),
                ..default()
            });
        }
    }
}
