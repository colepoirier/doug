use crate::ViewPortDimensions;
use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct DrawRectEvent {}

pub fn draw_rect(
    commands: &mut Commands,
    events: EventReader<DrawRectEvent>,
    dims: ResMut<ViewPortDimensions>,
) {
}

#[derive(Debug, Component)]
pub struct DrawPolyEvent {}

pub fn draw_poly(
    commands: &mut Commands,
    events: EventReader<DrawPolyEvent>,
    dims: ResMut<ViewPortDimensions>,
) {
}

#[derive(Debug, Component)]
pub struct DrawPathEvent {}

pub fn draw_path(
    commands: &mut Commands,
    events: EventReader<DrawPathEvent>,
    dims: ResMut<ViewPortDimensions>,
) {
}
