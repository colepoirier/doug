use crate::{
    shapes::{Path, Poly, Rect},
    CursorWorldPos, InLayer, ALPHA,
};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::DrawMode;

/// Marker component to indicate that the mouse
/// currently hovers over the given entity.
#[derive(Component)]
pub struct Hover;

pub fn hover_rect_system(
    mut commands: Commands,
    rect_q: Query<(Entity, &Rect, &InLayer)>,
    cursor_pos: Res<CursorWorldPos>,
) {
    let x = cursor_pos.x;
    let y = cursor_pos.y;

    let mut top_shape: (u16, Option<Entity>) = (0, None);

    for (
        entity,
        &Rect {
            width,
            height,
            origin,
        },
        layer,
    ) in rect_q.iter()
    {
        let x_min = origin.x;
        let x_max = origin.x + (width as i32);
        let y_min = origin.y;
        let y_max = origin.y + (height as i32);

        if (x_min <= x && x <= x_max) && (y_min <= y && y <= y_max) && **layer >= top_shape.0 {
            top_shape = (**layer, Some(entity));
        } else {
            commands.entity(entity).remove::<Hover>();
        }
    }
    if let Some(e) = top_shape.1 {
        commands.entity(e).insert(Hover);
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

pub fn hover_poly_system(poly_q: Query<&Poly>, cursor_pos: Res<CursorWorldPos>) {}

pub fn hover_path_system(path_q: Query<&Path>, cursor_pos: Res<CursorWorldPos>) {}

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
