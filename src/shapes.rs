use crate::{import::Net, InLayer};
use bevy::prelude::{Bundle, Component, Deref, DerefMut};
use bevy_prototype_lyon::entity;
use geo;
use layout21::raw;

#[derive(Default, Bundle)]
pub struct ShapeBundle {
    pub net: Net,
    pub layer: InLayer,
    #[bundle]
    pub shape_lyon: entity::ShapeBundle,
}

pub type GeoRect = geo::Rect<i32>;

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct Rect(pub GeoRect);

#[derive(Bundle)]
pub struct RectBundle {
    pub rect: Rect,
    #[bundle]
    pub shape: ShapeBundle,
}

pub type GeoPolygon = geo::Polygon<i32>;

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct Poly(pub GeoPolygon);

#[derive(Bundle)]
pub struct PolyBundle {
    pub poly: Poly,
    #[bundle]
    pub shape: ShapeBundle,
}

#[derive(Component, Clone, Default, Debug, Deref, DerefMut)]
pub struct Path(pub raw::Path);

#[derive(Default, Bundle)]
pub struct PathBundle {
    pub path: Path,
    #[bundle]
    pub shape: ShapeBundle,
}
