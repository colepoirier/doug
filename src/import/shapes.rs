use bevy::prelude::*;
use layout21::raw::{
    geom::{Path as VlsirPath, Polygon as VlsirPoly, Rect as VlsirRect},
    Point,
};

use crate::shapes::{InLayer, Net};

use super::poly_decomp::decompose_polygon;

#[derive(Debug, Clone)]
pub enum Shape {
    Rect(Rect),
    Poly(Vec<Rect>),
    Path(Vec<Rect>),
}

pub struct SpawnShape {
    pub shape: Shape,
    pub layer: InLayer,
    pub net: Net,
    pub color: Handle<ColorMaterial>,
}

#[must_use]
pub fn import_rect(
    rect: VlsirRect,
    net: &Option<String>,
    layer: u8,
    color: Handle<ColorMaterial>,
) -> SpawnShape {
    let rect = Rect::from_corners(
        (rect.p0.x as f32, rect.p0.y as f32).into(),
        (rect.p1.x as f32, rect.p1.y as f32).into(),
    );

    SpawnShape {
        shape: Shape::Rect(rect),
        layer: InLayer(layer),
        net: Net(net.clone()),
        color,
    }
}

#[must_use]
pub fn import_poly(
    poly: VlsirPoly,
    net: &Option<String>,
    layer: u8,
    color: Handle<ColorMaterial>,
) -> SpawnShape {
    let rects = decompose_polygon(&poly.points).unwrap();

    SpawnShape {
        shape: Shape::Poly(rects),
        layer: InLayer(layer),
        net: Net(net.clone()),
        color,
    }
}

#[must_use]
pub fn import_path(
    path: VlsirPath,
    net: &Option<String>,
    layer: u8,
    color: Handle<ColorMaterial>,
) -> SpawnShape {
    let num_points = path.points.len();
    let mut forward_poly_points = Vec::with_capacity(num_points);
    let mut backward_poly_points = Vec::with_capacity(num_points);
    // info!("width of path is {}", path.width);
    // info!("{:?}", path.points);
    assert_eq!(
        path.width % 2,
        0,
        "width must be even for our code's assumptions to hold!"
    );
    let half_width = (path.width / 2) as isize; // assuming that widths are even!
    for ix in 0..num_points {
        let p0 = path.points[ix];
        let p1 = path.points[(ix + 1) % num_points];
        // let corrected_point = p0.shift(offset);
        if p0.x == p1.x {
            // vertical
            forward_poly_points.push(Point {
                x: p0.x + half_width,
                y: p0.y,
            });
            backward_poly_points.push(Point {
                x: p0.x - half_width,
                y: p0.y,
            });
        } else {
            // horizontal
            forward_poly_points.push(Point {
                x: p0.x,
                y: p0.y - half_width,
            });
            backward_poly_points.push(Point {
                x: p0.x,
                y: p0.y + half_width,
            });
        }
    }
    let points = forward_poly_points
        .into_iter()
        .chain(backward_poly_points.into_iter().rev())
        .map(|Point { x, y }| IVec2::new(x as i32, y as i32))
        .collect::<Vec<IVec2>>();

    let rects = decompose_polygon(&points).unwrap();

    SpawnShape {
        shape: Shape::Path(rects),
        layer: InLayer(layer),
        net: Net(net.clone()),
        color,
    }
}
