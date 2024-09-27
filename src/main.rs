mod camera;
mod cursor;
mod obstacles;
mod pathfinding;
mod player;
mod player_action;
mod player_gizmos;
mod player_movement;
mod player_stats;

pub use player::*;
pub use player_action::*;
pub use player_gizmos::*;
pub use player_movement::*;
pub use player_stats::*;
mod utils;

use crate::pathfinding::NavMesh;
use bevy::{
    color::palettes::css::GOLD,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::PresentMode,
};
use obstacles::*;
#[derive(Component)]
struct FpsText;

#[derive(Component)]
pub struct Ground;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin,
        ))
        .insert_resource(player::TargetPosition(None))
        .insert_resource(player::GizmoPath(None))
        .insert_resource(camera::CameraFollowToggle(true))
        .insert_resource(camera::CameraZoom(10.0))
        .insert_resource(cursor::CursorPosition::default())
        .insert_resource(player::LastTargetPosition(None))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                cursor::draw_cursor,
                player::handle_right_click_set_target_position,
                player::move_player_towards_target,
                camera::camera_follow,
                camera::toggle_camera_follow,
                camera::camera_edge_pan,
                camera::camera_zoom,
                player_movement::move_player_with_wasd,
                player_action::cast_q_spell,
                draw_path_gizmos,
                text_update_system,
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
        PlayerStats::new(5.0, 100.0, 1.0),
        PlayerAction::new(),
    ));

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(16.875, 16.875, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(120., 120.)),
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
