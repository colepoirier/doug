use crate::{import::Net, InLayer};
use bevy::prelude::{Bundle, Component};
use bevy_prototype_lyon::entity;
use derive_more::{Deref, DerefMut};
use layout21raw;

#[derive(Component, Default, Bundle)]
pub struct ShapeBundle {
    pub net: Net,
    pub layer: InLayer,
    #[bundle]
    pub shape_lyon: entity::ShapeBundle,
}

#[derive(Component, Clone, Default, Debug, Deref, DerefMut)]
pub struct Rect(pub layout21raw::Rect);

#[derive(Default, Bundle)]
pub struct RectBundle {
    pub rect: Rect,
    #[bundle]
    pub shape: ShapeBundle,
}

#[derive(Component, Clone, Default, Debug, Deref, DerefMut)]
pub struct Poly(pub layout21raw::Polygon);

#[derive(Component, Default, Bundle)]
pub struct PolyBundle {
    pub poly: Poly,
    #[bundle]
    pub shape: ShapeBundle,
}

#[derive(Component, Clone, Default, Debug, Deref, DerefMut)]
pub struct Path(pub layout21raw::Path);

#[derive(Component, Default, Bundle)]
pub struct PathBundle {
    pub path: Path,
    #[bundle]
    pub shape: ShapeBundle,
}
