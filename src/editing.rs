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

#[derive(Component, Copy, Clone, Debug, Default)]
pub struct TopShape {
    pub layer: u16,
    pub shape: Option<Entity>,
}

impl PartialEq for TopShape {
    fn eq(&self, other: &Self) -> bool {
        if self.layer == other.layer {
            true
        } else {
            false
        }
    }
}

impl PartialOrd for TopShape {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.layer.partial_cmp(&other.layer)
    }
}

pub fn hover_rect_system(
    mut commands: Commands,
    mut top_shape: ResMut<TopShape>,
    rect_q: Query<(Entity, &Path, &InLayer), With<Rect>>,
    cursor_pos: Res<CursorWorldPos>,
) {
    let x = cursor_pos.x;
    let y = cursor_pos.y;

    for (entity, path, layer) in rect_q.iter() {
        let layer = **layer;

        if hit_test_path(
            &(x as f32, y as f32).into(),
            path.0.into_iter(),
            FillRule::NonZero,
            10.0,
        ) && layer >= top_shape.layer
        {
            *top_shape = TopShape {
                layer,
                shape: Some(entity),
            };
        } else {
            commands.entity(entity).remove::<Hover>();
        }
    }
    if let Some(e) = top_shape.shape {
        commands.entity(e).insert(Hover);
    }
}

pub fn hover_poly_system(
    mut commands: Commands,
    mut top_shape: ResMut<TopShape>,
    poly_q: Query<(Entity, &Path, &InLayer), With<Poly>>,
    cursor_pos: Res<CursorWorldPos>,
) {
    let x = cursor_pos.x;
    let y = cursor_pos.y;

    for (entity, path, layer) in poly_q.iter() {
        let layer = **layer;

        if hit_test_path(
            &(x as f32, y as f32).into(),
            path.0.into_iter(),
            FillRule::NonZero,
            10.0,
        ) && layer >= top_shape.layer
        {
            *top_shape = TopShape {
                layer,
                shape: Some(entity),
            };
        } else {
            commands.entity(entity).remove::<Hover>();
        }
        if let Some(e) = top_shape.shape {
            commands.entity(e).insert(Hover);
        }
    }
}

pub fn hover_path_system(
    mut commands: Commands,
    mut top_shape: ResMut<TopShape>,
    path_q: Query<(Entity, &Path, &InLayer), With<shapes::Path>>,
    cursor_pos: Res<CursorWorldPos>,
) {
    let x = cursor_pos.x;
    let y = cursor_pos.y;

    for (entity, path, layer) in path_q.iter() {
        let layer = **layer;

        if hit_test_path(
            &(x as f32, y as f32).into(),
            path.0.into_iter(),
            FillRule::NonZero,
            10.0,
        ) && layer >= top_shape.layer
        {
            *top_shape = TopShape {
                layer,
                shape: Some(entity),
            };
        } else {
            commands.entity(entity).remove::<Hover>();
        }
        if let Some(e) = top_shape.shape {
            commands.entity(e).insert(Hover);
        }
    }
}

/// Highlight a shape by making it more opaque when the mouse hovers over it.
pub fn highlight_shape_system(
    // We need all shapes the mouse hovers over.
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
