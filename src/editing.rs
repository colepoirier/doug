use crate::{
    shapes,
    shapes::{Poly, Rect},
    CursorWorldPos, InLayer, ALPHA,
};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{DrawMode, FillRule, Path};

use lyon_algorithms::hit_test::hit_test_path;

/// Marker component to indicate that the mouse
/// currently hovers over the given entity.
#[derive(Component)]
pub struct Hover;

pub fn hover_rect_system(
    mut commands: Commands,
    rect_q: Query<(Entity, &Path, &InLayer), With<Rect>>,
    cursor_pos: Res<CursorWorldPos>,
) {
    let x = cursor_pos.x;
    let y = cursor_pos.y;

    let mut top_shape: (u16, Option<Entity>) = (0, None);

    for (entity, path, layer) in rect_q.iter() {
        if hit_test_path(
            &(x as f32, y as f32).into(),
            path.0.into_iter(),
            FillRule::NonZero,
            10.0,
        ) && **layer >= top_shape.0
        {
            top_shape = (**layer, Some(entity));
        } else {
            commands.entity(entity).remove::<Hover>();
        }
    }
    if let Some(e) = top_shape.1 {
        commands.entity(e).insert(Hover);
    }
}

pub fn hover_poly_system(
    mut commands: Commands,
    poly_q: Query<(Entity, &Path, &InLayer), With<Poly>>,
    cursor_pos: Res<CursorWorldPos>,
) {
    let x = cursor_pos.x;
    let y = cursor_pos.y;

    let mut top_shape: (u16, Option<Entity>) = (0, None);

    for (entity, path, layer) in poly_q.iter() {
        if hit_test_path(
            &(x as f32, y as f32).into(),
            path.0.into_iter(),
            FillRule::NonZero,
            10.0,
        ) && **layer >= top_shape.0
        {
            top_shape = (**layer, Some(entity));
        } else {
            commands.entity(entity).remove::<Hover>();
        }
        if let Some(e) = top_shape.1 {
            commands.entity(e).insert(Hover);
        }
    }
}

pub fn hover_path_system(
    mut commands: Commands,
    path_q: Query<(Entity, &Path, &InLayer), With<shapes::Path>>,
    cursor_pos: Res<CursorWorldPos>,
) {
    let x = cursor_pos.x;
    let y = cursor_pos.y;

    let mut top_shape: (u16, Option<Entity>) = (0, None);

    for (entity, path, layer) in path_q.iter() {
        if hit_test_path(
            &(x as f32, y as f32).into(),
            path.0.into_iter(),
            FillRule::NonZero,
            10.0,
        ) && **layer >= top_shape.0
        {
            top_shape = (**layer, Some(entity));
        } else {
            commands.entity(entity).remove::<Hover>();
        }
        if let Some(e) = top_shape.1 {
            commands.entity(e).insert(Hover);
        }
    }
}

/// Highlight a connector by increasing its radius when the mouse
/// hovers over it.
pub fn highlight_shape_system(
    // We need all connectors the mouse hovers over.
    mut q_hover: Query<&mut DrawMode, Changed<Hover>>,
    mut q2_hover: Query<&mut DrawMode, Without<Hover>>,
) {
    for mut draw in q_hover.iter_mut() {
        if let DrawMode::Outlined {
            ref mut fill_mode, ..
        } = *draw
        {
            fill_mode.color = *fill_mode.color.set_a(1.0);
        }
    }

    for mut draw in q2_hover.iter_mut() {
        if let DrawMode::Outlined {
            ref mut fill_mode, ..
        } = *draw
        {
            fill_mode.color = *fill_mode.color.set_a(ALPHA);
        }
    }
}

// #[derive(Component)]
// pub enum RectSelection {
//     BottomLeft,
//     Left,
//     TopLeft,
//     Top,
//     TopRight,
//     Right,
//     BottomRight,
//     Bottom,
//     Body,
// }
