mod camera;
mod obstacles;
mod pathfinding;
mod player;
mod utils;

use crate::pathfinding::NavMesh;
use bevy::{
    color::palettes::css::GOLD,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::PresentMode,
};
use obstacles::*;
use player::*;

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct Ground;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Mailbox,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin,
        ))
        .insert_resource(camera::CameraFollowToggle(true))
        .insert_resource(TargetPosition(None))
        .insert_resource(player::GizmoPath(None))
        .insert_resource(camera::CameraZoom(10.0))
        .insert_resource(CursorPosition::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                draw_path_gizmos,
                text_update_system,
                move_player,
                camera::camera_follow,
                camera::toggle_camera_follow,
                camera::camera_edge_pan,
                camera::camera_zoom,
                move_player_with_wasd,
                update_cursor_position,
                draw_cursor,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut obstacle_polygons = ObstaclePolygons::new();
    let transforms_and_scales = generate_cuboids(&mut obstacle_polygons);
    render_cuboids(
        &mut commands,
        &mut meshes,
        &mut materials,
        transforms_and_scales,
    );

    let cloned_polygons = obstacle_polygons.clone();
    commands.insert_resource(cloned_polygons);

    // Build the nav mesh based on the generated obstacles
    let mut nav_mesh = NavMesh::new();
    for polygon in &obstacle_polygons.polygons {
        for vertex in &polygon.vertices {
            nav_mesh.add_vertex(vertex.clone());
        }
    }
    commands.insert_resource(nav_mesh);

    for polygon in &obstacle_polygons.polygons {
        for vertex in &polygon.vertices {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.1)),
                material: materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    ..default()
                }),
                transform: Transform::from_xyz(vertex.x, vertex.y, vertex.z),
                ..default()
            });
        }
    }

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Player,
    ));

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(16.875, 16.875, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(80., 80.)),
            material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
            ..default()
        },
        Ground,
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    ..default()
                },
            ),
            TextSection::new(
                "",
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 60.0,
                    color: GOLD.into(),
                },
            ),
        ]),
        FpsText,
    ));
}

fn text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

#[derive(Resource, Default)]
struct CursorPosition {
    position: Option<Vec3>,
}

fn update_cursor_position(
    mut cursor_position: ResMut<CursorPosition>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    windows: Query<&Window>,
) {
    if mouse_button_input.just_pressed(MouseButton::Right) {
        let (camera, camera_transform) = camera_query.single();
        let ground = ground_query.single();

        if let Some(cursor_position_2d) = windows.single().cursor_position() {
            // Calculate a ray pointing from the camera into the world based on the cursor's position.
            if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position_2d) {
                // Calculate if and where the ray is hitting the ground plane.
                if let Some(distance) =
                    ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
                {
                    let point = ray.get_point(distance);
                    cursor_position.position = Some(point);
                }
            }
        }
    }
}

fn draw_cursor(
    cursor_position: Res<CursorPosition>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    mut gizmos: Gizmos,
) {
    if let Some(position) = cursor_position.position {
        let ground = ground_query.single();
        // Draw a circle just above the ground plane at the stored position.
        gizmos.circle(
            position + ground.up() * 0.01,
            ground.up(),
            0.3,
            Color::srgb(0.0, 0.0, 0.5),
        );
    }
}
