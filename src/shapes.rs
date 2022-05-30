use crate::{import::Net, InLayer};
use bevy::prelude::{Bundle, Component, Deref, DerefMut};
use bevy_prototype_lyon::entity;
use layout21::raw;

#[derive(Default, Bundle)]
pub struct ShapeBundle {
    pub net: Net,
    pub layer: InLayer,
    #[bundle]
    pub shape_lyon: entity::ShapeBundle,
}

#[derive(Component, Clone, Default, Debug, Deref, DerefMut)]
pub struct Rect(pub raw::Rect);

#[derive(Default, Bundle)]
pub struct RectBundle {
    pub rect: Rect,
    #[bundle]
    pub shape: ShapeBundle,
}

#[derive(Component, Clone, Default, Debug, Deref, DerefMut)]
pub struct Poly(pub raw::Polygon);

#[derive(Default, Bundle)]
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
