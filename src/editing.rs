use crate::{
    shapes,
    shapes::{Poly, Rect},
    CursorWorldPos, InLayer, ALPHA,
};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{DrawMode, FillRule, Path};

use lyon_algorithms::hit_test::hit_test_path;
use lyon_geom::Translation;

/// Marker component to indicate that the mouse
/// currently hovers over the given entity.
#[derive(Component)]
pub struct Hover;

#[derive(Copy, Clone, Debug, Default)]
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

pub fn hover_shape_system(
    mut commands: Commands,
    cursor_pos: Res<CursorWorldPos>,
    rect_q: Query<(Entity, &Path, &Transform, &InLayer), With<Rect>>,
    poly_q: Query<(Entity, &Path, &Transform, &InLayer), With<Poly>>,
    path_q: Query<(Entity, &Path, &Transform, &InLayer), With<shapes::Path>>,
    hover_q: Query<Entity, With<Hover>>,
) {
    let mut top_shape = TopShape::default();

    let point = lyon_geom::point(cursor_pos.x as f32, cursor_pos.y as f32);

    for (entity, path, transform, layer) in rect_q.iter() {
        let layer = **layer;

        let path = path.0.clone().transformed(&Translation::new(
            transform.translation.x,
            transform.translation.y,
        ));
        let shape = TopShape {
            layer,
            shape: Some(entity),
        };

        if hit_test_path(&point, path.iter(), FillRule::NonZero, 0.0) && shape > top_shape {
            top_shape = shape;
        } else {
            commands.entity(entity).remove::<Hover>();
        }
    }

    for (entity, path, transform, layer) in poly_q.iter() {
        let layer = **layer;

        let path = path.0.clone().transformed(&Translation::new(
            transform.translation.x,
            transform.translation.y,
        ));

        let shape = TopShape {
            layer,
            shape: Some(entity),
        };

        if hit_test_path(&point, path.iter(), FillRule::NonZero, 0.0) && shape > top_shape {
            top_shape = shape;
        } else {
            commands.entity(entity).remove::<Hover>();
        }
    }

    for (entity, path, transform, layer) in path_q.iter() {
        let layer = **layer;

        let path = path.0.clone().transformed(&Translation::new(
            transform.translation.x,
            transform.translation.y,
        ));

        let shape = TopShape {
            layer,
            shape: Some(entity),
        };

        if hit_test_path(&point, path.iter(), FillRule::NonZero, 0.0) && shape > top_shape {
            top_shape = shape;
        } else {
            commands.entity(entity).remove::<Hover>();
        }
    }

    if let Some(e) = top_shape.shape {
        commands.entity(e).insert(Hover);
    } else {
        for e in hover_q.iter() {
            commands.entity(e).remove::<Hover>();
        }
    }
}

/// Highlight a shape by making it more opaque when the mouse hovers over it.
pub fn highlight_hovered_system(
    // We need all shapes the mouse hovers over.
    mut hover_q: Query<&mut DrawMode, Changed<Hover>>,
    mut no_hover_q: Query<&mut DrawMode, Without<Hover>>,
) {
    for mut draw in hover_q.iter_mut() {
        if let DrawMode::Outlined {
            ref mut fill_mode, ..
        } = *draw
        {
            fill_mode.color = *fill_mode.color.set_a(1.0);
        }
    }

    for mut draw in no_hover_q.iter_mut() {
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
