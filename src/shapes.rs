use bevy::prelude::{Component, Deref, DerefMut};

#[derive(Component, Debug, Default, Clone, PartialEq, PartialOrd, Deref, DerefMut)]
pub struct Net(pub Option<String>);

#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct InLayer(pub u8);

#[derive(Component)]
pub struct Rect;

#[derive(Component)]
pub struct Poly;

#[derive(Component)]
pub struct Path;
