use bevy::math::{IVec2, Rect, Vec2};
use layout21::raw::Point;

#[derive(Debug, Clone, Copy)]
struct Edge {
    x: i32,
    y_min: i32,
    y_max: i32,
}

pub trait IntoIVec2 {
    fn into_ivec2(&self) -> IVec2;
}

// Implement for your Point struct
impl IntoIVec2 for Point {
    fn into_ivec2(&self) -> IVec2 {
        IVec2::new(self.x as i32, self.y as i32)
    }
}

impl IntoIVec2 for IVec2 {
    fn into_ivec2(&self) -> IVec2 {
        *self // Just return a copy since it's already an IVec2
    }
}

pub fn decompose_polygon<T: IntoIVec2>(vertices: &[T]) -> Option<Vec<Rect>> {
    let mut edges = [(Edge {
        x: 0,
        y_min: 0,
        y_max: 0,
    }); 256];
    let mut y_coords = [0i32; 256];
    let mut xs = [0i32; 256];

    // Combined edge processing and y-coord collection, knowing every other edge is vertical
    let mut edge_count = 0;
    let mut y_count = 0;

    if vertices.len() > y_coords.len() {
        return None;
    }

    for i in (1..vertices.len()).step_by(2) {
        let v1 = vertices[i].into_ivec2();
        let v2 = vertices[(i + 1) % vertices.len()].into_ivec2();

        if edge_count >= edges.len() {
            return None;
        }

        edges[edge_count] = Edge {
            x: v1.x,
            y_min: v1.y.min(v2.y),
            y_max: v1.y.max(v2.y),
        };
        edge_count += 1;

        // Collect y coordinates during the same pass
        y_coords[y_count] = v1.y;
        y_count += 1;
        y_coords[y_count] = v2.y;
        y_count += 1;
    }

    y_coords[..y_count].sort_unstable();

    if y_count > 0 {
        let mut write = 1;
        for read in 1..y_count {
            if y_coords[read] != y_coords[write - 1] {
                y_coords[write] = y_coords[read];
                write += 1;
            }
        }
        y_count = write;
    }

    let mut result = Vec::new();
    for ys in y_coords[..y_count].windows(2) {
        let (y1, y2) = (ys[0], ys[1]);
        let mut x_count = 0;
        for edge in &edges[..edge_count] {
            if edge.y_min <= y1 && edge.y_max >= y2 {
                if x_count >= xs.len() {
                    return None;
                }
                xs[x_count] = edge.x;
                x_count += 1;
            }
        }
        xs[..x_count].sort_unstable();
        for chunk in xs[..x_count].chunks_exact(2) {
            result.push(Rect {
                min: Vec2::new(chunk[0] as f32, y1 as f32),
                max: Vec2::new(chunk[1] as f32, y2 as f32),
            });
        }
    }
    Some(result)
}
