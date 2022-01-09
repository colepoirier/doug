use crate::{InLayer, Nom};
use bevy::prelude::{Bundle, Component, IVec2};
use bevy_prototype_lyon::entity;
use derive_more::{Deref, DerefMut};
use layout21::protos::Point;

#[derive(Component, Default, Bundle)]
pub struct ShapeBundle {
    pub name: Nom,
    pub layer: InLayer,
    #[bundle]
    pub shape_lyon: entity::ShapeBundle,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Rect {
    pub width: u32,
    pub height: u32,
    pub origin: IVec2,
}

#[derive(Component, Default, Bundle)]
pub struct RectBundle {
    pub rect: Rect,
    #[bundle]
    pub shape: ShapeBundle,
}

#[derive(Component, Debug, Default, Clone)]
pub struct Poly {
    pub verts: Vec<Point>,
}

#[derive(Component, Default, Bundle)]
pub struct PolyBundle {
    pub poly: Poly,
    #[bundle]
    pub shape: ShapeBundle,
}

#[derive(Component, Debug, Clone, Default)]
pub struct Path {
    pub verts: Vec<Point>,
}

#[derive(Component, Default, Bundle)]
pub struct PathBundle {
    pub path: Path,
    #[bundle]
    pub shape: ShapeBundle,
}
