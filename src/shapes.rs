use crate::import::Net;
use bevy::prelude::{Bundle, Component, Deref, DerefMut};
use bevy_mod_picking::PickableBundle;
use bevy_prototype_lyon::{
    entity,
    prelude::{Fill, Stroke},
};
use geo;
use layout21::raw;

#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct InLayer(pub u8);

impl Default for InLayer {
    fn default() -> Self {
        InLayer(0)
    }
}

#[derive(
    Component, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deref, DerefMut,
)]
pub struct LayerNum(pub u8);

#[derive(Bundle)]
pub struct ShapeBundle {
    pub net: Net,
    pub layer: InLayer,
    pub shape_lyon: entity::ShapeBundle,
    pub fill: Fill,
    pub stroke: Stroke,
    pub pickable: PickableBundle,
}

pub type GeoRect = geo::Rect<i32>;

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct Rect(pub GeoRect);

#[derive(Bundle)]
pub struct RectBundle {
    pub rect: Rect,
    pub shape: ShapeBundle,
}

pub type GeoPolygon = geo::Polygon<i32>;

#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub struct Poly(pub GeoPolygon);

#[derive(Bundle)]
pub struct PolyBundle {
    pub poly: Poly,
    pub shape: ShapeBundle,
}

#[derive(Component, Clone, Default, Debug, Deref, DerefMut)]
pub struct Path(pub raw::Path);

#[derive(Bundle)]
pub struct PathBundle {
    pub path: Path,
    pub shape: ShapeBundle,
}
