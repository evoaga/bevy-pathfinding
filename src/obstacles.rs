use crate::utils::Polygon;
use bevy::prelude::*;
use rand::Rng;

#[derive(Debug, Clone, Resource)]
pub struct ObstaclePolygons {
    pub polygons: Vec<Polygon>,
}

impl ObstaclePolygons {
    pub fn new() -> Self {
        ObstaclePolygons {
            polygons: Vec::new(),
        }
    }

    pub fn add_polygon(&mut self, polygon: Polygon) {
        self.polygons.push(polygon);
    }
}

pub fn generate_cuboids(obstacle_polygons: &mut ObstaclePolygons) -> Vec<(Transform, Vec3)> {
    let mut rng = rand::thread_rng();
    let mut transforms_and_scales = Vec::new();

    for _ in 0..120 {
        let scale_x = rng.gen_range(0.5..2.0);
        let scale_y = 1.0;
        let scale_z = rng.gen_range(0.5..2.0);

        let x = rng.gen_range(-40.0..40.0);
        let y = 0.5;
        let z = rng.gen_range(-40.0..40.0);

        let rotation_x = 0.0;
        let rotation_y = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
        let rotation_z = 0.0;

        let transform = Transform::from_xyz(x, y, z)
            .with_scale(Vec3::new(scale_x, scale_y, scale_z))
            .with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                rotation_x,
                rotation_y,
                rotation_z,
            ));

        transforms_and_scales.push((transform, Vec3::new(scale_x, scale_y, scale_z)));

        let polygon = generate_cuboid_polygon(transform, scale_x, scale_y, scale_z);
        obstacle_polygons.add_polygon(polygon);
    }

    transforms_and_scales
}

pub fn generate_cuboid_polygon(
    transform: Transform,
    scale_x: f32,
    scale_y: f32,
    scale_z: f32,
) -> Polygon {
    let mut polygon = Polygon::new();
    let buffer = 0.5;

    // Vertices are ordered counterclockwise when viewed from above
    let vertices = vec![
        Vec3::new(
            -scale_x / 2.0 - buffer,
            -scale_y / 2.0,
            -scale_z / 2.0 - buffer,
        ), // Bottom-left corner
        Vec3::new(
            -scale_x / 2.0 - buffer,
            -scale_y / 2.0,
            scale_z / 2.0 + buffer,
        ), // Top-left corner
        Vec3::new(
            scale_x / 2.0 + buffer,
            -scale_y / 2.0,
            scale_z / 2.0 + buffer,
        ), // Top-right corner
        Vec3::new(
            scale_x / 2.0 + buffer,
            -scale_y / 2.0,
            -scale_z / 2.0 - buffer,
        ), // Bottom-right corner
    ];

    for vertex in vertices {
        let transformed_vertex = transform.transform_point(vertex);
        polygon.add_vertex(
            transformed_vertex.x,
            transformed_vertex.y,
            transformed_vertex.z,
        );
    }

    polygon
}

pub fn render_cuboids(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    transforms_and_scales: Vec<(Transform, Vec3)>,
) {
    for (transform, scale) in transforms_and_scales {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(scale.x, scale.y, scale.z)),
            material: materials.add(Color::BLACK),
            transform,
            ..default()
        });
    }
}
