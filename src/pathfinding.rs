use bevy::prelude::Resource;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::f32;

use crate::utils::{line_intersects_polygon_with_vertex_check, Point, Polygon};

struct Node {
    point: Point,
    g_score: f32,
    f_score: f32,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.f_score == other.f_score
    }
}

impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f_score
            .partial_cmp(&self.f_score)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Resource)]
pub struct NavMesh {
    pub vertices: Vec<Point>,
}

impl NavMesh {
    pub fn new() -> Self {
        NavMesh {
            vertices: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, point: Point) {
        self.vertices.push(point);
    }

    pub fn remove_vertex(&mut self, point: &Point) {
        self.vertices.retain(|v| v != point);
    }
}

fn heuristic(p1: &Point, p2: &Point) -> f32 {
    ((p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2) + (p1.z - p2.z).powi(2)).sqrt()
}

fn line_of_sight(s: &Point, s_prime: &Point, polygons: &[Polygon]) -> bool {
    for polygon in polygons {
        if line_intersects_polygon_with_vertex_check(s, s_prime, polygon) {
            return false;
        }
    }
    true
}

pub fn theta_star(
    mesh: &mut NavMesh,
    start: Point,
    goal: Point,
    obstacle_polygons: &[Polygon],
) -> Vec<Point> {
    mesh.add_vertex(start.clone());
    mesh.add_vertex(goal.clone());

    let mut open_list = BinaryHeap::new();
    let mut came_from: HashMap<Point, Point> = HashMap::new();
    let mut g_score: HashMap<Point, f32> = HashMap::new();
    let mut f_score: HashMap<Point, f32> = HashMap::new();

    let inf = f32::INFINITY;

    for vertex in &mesh.vertices {
        g_score.insert(vertex.clone(), inf);
        f_score.insert(vertex.clone(), inf);
    }

    g_score.insert(start.clone(), 0.0);
    f_score.insert(start.clone(), heuristic(&start, &goal));

    open_list.push(Node {
        point: start.clone(),
        g_score: 0.0,
        f_score: heuristic(&start, &goal),
    });

    came_from.insert(start.clone(), start.clone());

    while let Some(Node {
        point: current,
        g_score: current_g_score,
        ..
    }) = open_list.pop()
    {
        if current == goal {
            let mut path = Vec::new();
            let mut current = current;
            while let Some(prev) = came_from.get(&current) {
                if &current == prev {
                    break;
                }
                path.push(current.clone());
                current = prev.clone();
            }
            path.push(start.clone());
            path.reverse();

            mesh.remove_vertex(&start);
            mesh.remove_vertex(&goal);

            return path;
        }

        for neighbor in &mesh.vertices {
            if neighbor != &current && line_of_sight(&current, neighbor, obstacle_polygons) {
                let parent = came_from.get(&current).unwrap_or(&current).clone();

                if line_of_sight(&parent, neighbor, obstacle_polygons) {
                    let tentative_g_score = g_score[&parent] + heuristic(&parent, neighbor);
                    if tentative_g_score < g_score[neighbor] {
                        came_from.insert(neighbor.clone(), parent.clone());
                        g_score.insert(neighbor.clone(), tentative_g_score);
                        let new_f_score = tentative_g_score + heuristic(neighbor, &goal);
                        f_score.insert(neighbor.clone(), new_f_score);
                        open_list.push(Node {
                            point: neighbor.clone(),
                            g_score: tentative_g_score,
                            f_score: new_f_score,
                        });
                    }
                } else {
                    let tentative_g_score = current_g_score + heuristic(&current, neighbor);
                    if tentative_g_score < g_score[neighbor] {
                        came_from.insert(neighbor.clone(), current.clone());
                        g_score.insert(neighbor.clone(), tentative_g_score);
                        let new_f_score = tentative_g_score + heuristic(neighbor, &goal);
                        f_score.insert(neighbor.clone(), new_f_score);
                        open_list.push(Node {
                            point: neighbor.clone(),
                            g_score: tentative_g_score,
                            f_score: new_f_score,
                        });
                    }
                }
            }
        }
    }

    mesh.remove_vertex(&start);
    mesh.remove_vertex(&goal);

    Vec::new()
}
