use crate::ViewPortDimensions;
use bevy::prelude::*;

pub struct DrawRectEvent {}

pub fn draw_rect(
    commands: &mut Commands,
    events: EventReader<DrawRectEvent>,
    dims: ResMut<ViewPortDimensions>,
) {
}

pub struct DrawPolyEvent {}

pub fn draw_poly(
    commands: &mut Commands,
    events: EventReader<DrawPolyEvent>,
    dims: ResMut<ViewPortDimensions>,
) {
}

pub struct DrawPathEvent {}

pub fn draw_path(
    commands: &mut Commands,
    events: EventReader<DrawPathEvent>,
    dims: ResMut<ViewPortDimensions>,
) {
}
