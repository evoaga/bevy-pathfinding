use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl Eq for Point {}

impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // We can use the bit representation of the float for hashing
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.z.to_bits().hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct Polygon {
    pub vertices: Vec<Point>,
}

impl Polygon {
    pub fn new() -> Self {
        Polygon {
            vertices: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, x: f32, y: f32, z: f32) {
        self.vertices.push(Point { x, y, z });
    }
}

pub fn do_lines_intersect(p1: &Point, p2: &Point, q1: &Point, q2: &Point) -> bool {
    let d1 = direction(q1, q2, p1);
    let d2 = direction(q1, q2, p2);
    let d3 = direction(p1, p2, q1);
    let d4 = direction(p1, p2, q2);

    if d1 * d2 < 0.0 && d3 * d4 < 0.0 {
        return true;
    }

    if d1 == 0.0 && on_segment(q1, q2, p1) {
        return true;
    }
    if d2 == 0.0 && on_segment(q1, q2, p2) {
        return true;
    }
    if d3 == 0.0 && on_segment(p1, p2, q1) {
        return true;
    }
    if d4 == 0.0 && on_segment(p1, p2, q2) {
        return true;
    }

    false
}

pub fn direction(p: &Point, q: &Point, r: &Point) -> f32 {
    (q.x - p.x) * (r.z - p.z) - (q.z - p.z) * (r.x - p.x)
}

pub fn on_segment(p: &Point, q: &Point, r: &Point) -> bool {
    r.x >= p.x.min(q.x) && r.x <= p.x.max(q.x) && r.z >= p.z.min(q.z) && r.z <= p.z.max(q.z)
}

pub fn does_line_intersect_polygon(
    line_start: &Point,
    line_end: &Point,
    polygon: &Polygon,
) -> bool {
    let n = polygon.vertices.len();
    for i in 0..n {
        let next_i = (i + 1) % n;
        if do_lines_intersect(
            line_start,
            line_end,
            &polygon.vertices[i],
            &polygon.vertices[next_i],
        ) {
            return true;
        }
    }
    false
}

pub fn line_intersects_polygon_with_vertex_check(
    line_start: &Point,
    line_end: &Point,
    polygon: &Polygon,
) -> bool {
    let n = polygon.vertices.len();
    let mut line_start_on_edge = false;
    let mut line_start_edge_index = None;
    let mut line_start_index = None;
    let mut line_end_index = None;

    // Determine if line_start and line_end are vertices of the polygon
    for (i, vertex) in polygon.vertices.iter().enumerate() {
        if line_start == vertex {
            line_start_index = Some(i);
        }
        if line_end == vertex {
            line_end_index = Some(i);
        }
    }

    // Check if both points are vertices and are not neighbors
    if let (Some(start_idx), Some(end_idx)) = (line_start_index, line_end_index) {
        // Calculate if they are neighbors (i.e., consecutive in the vertex list)
        if (start_idx + 1) % n != end_idx && start_idx != (end_idx + 1) % n {
            return true;
        }
    }

    // Check if line_start is on an edge of the polygon (but not on a vertex)
    for i in 0..n {
        let next_i = (i + 1) % n;
        let v1 = &polygon.vertices[i];
        let v2 = &polygon.vertices[next_i];

        if on_segment(v1, v2, line_start) && line_start != v1 && line_start != v2 {
            line_start_on_edge = true;
            line_start_edge_index = Some(i);
            break;
        }
    }

    if line_start_on_edge {
        // Check if line_end is on a vertex of the polygon
        for (i, vertex) in polygon.vertices.iter().enumerate() {
            if line_end == vertex {
                // If line_end is on a vertex, check if it's a vertex of the edge line_start is on
                if let Some(start_edge_index) = line_start_edge_index {
                    let next_index = (start_edge_index + 1) % n;
                    if i == start_edge_index || i == next_index {
                        return false;
                    }
                }
                return true;
            }
        }
    }

    // Check if the line intersects with any of the polygon's edges
    for i in 0..n {
        let next_i = (i + 1) % n;
        let v1 = &polygon.vertices[i];
        let v2 = &polygon.vertices[next_i];

        // Skip this edge if line_start or line_end is exactly on one of the vertices of the polygon
        if line_start == v1 || line_start == v2 || line_end == v1 || line_end == v2 {
            continue;
        }

        if do_lines_intersect(line_start, line_end, v1, v2) {
            return true;
        }
    }
    false
}
