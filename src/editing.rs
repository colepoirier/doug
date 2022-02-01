use crate::{
    shapes,
    shapes::{Poly, Rect},
    CursorWorldPos, InLayer, ALPHA,
};
use bevy::prelude::*;
use bevy_prototype_lyon::plugin::ShapePlugin;
use bevy_prototype_lyon::prelude::{DrawMode, FillRule, Path};

use lyon_algorithms::hit_test::hit_test_path;
use lyon_geom::Translation;

pub struct EditingPlugin;

impl Plugin for EditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ShapePlugin)
            .add_stage("detect_hover", SystemStage::parallel())
            .add_stage_after("detect_hover", "highlight_hovered", SystemStage::parallel())
            .add_stage_after(
                "highlight_hovered",
                "detect_clicked",
                SystemStage::parallel(),
            )
            .add_stage_after(
                "detect_clicked",
                "highlight_selected",
                SystemStage::parallel(),
            )
            .add_system_to_stage("detect_hover", cursor_hover_system)
            .add_system_to_stage("highlight_hovered", highlight_hovered_system)
            .add_system_to_stage("detect_clicked", select_clicked_system)
            .add_system_to_stage("highlight_selected", highlight_selected_sytem);
    }
}

/// Marker component to indicate that the mouse
/// currently hovers over the given entity.
#[derive(Component)]
pub struct Hovered;

/// Marker component to indicate that the given
/// is currently selected entity.
#[derive(Component)]
pub struct Selected;

/// Resource to calculate the interacted shape by layer/z-order
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

pub fn cursor_hover_system(
    mut commands: Commands,
    cursor_pos: Res<CursorWorldPos>,
    rect_q: Query<(Entity, &Path, &Transform, &InLayer), With<Rect>>,
    poly_q: Query<(Entity, &Path, &Transform, &InLayer), With<Poly>>,
    path_q: Query<(Entity, &Path, &Transform, &InLayer), With<shapes::Path>>,
    hover_q: Query<Entity, With<Hovered>>,
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
            commands.entity(entity).remove::<Hovered>();
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
            commands.entity(entity).remove::<Hovered>();
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
            commands.entity(entity).remove::<Hovered>();
        }
    }

    if let Some(e) = top_shape.shape {
        commands.entity(e).insert(Hovered);
    } else {
        for e in hover_q.iter() {
            commands.entity(e).remove::<Hovered>();
        }
    }
}

/// Highlight a shape by making it more opaque when the mouse hovers over it.
pub fn highlight_hovered_system(
    // We need all shapes the mouse hovers over.
    mut hover_q: Query<&mut DrawMode, Changed<Hovered>>,
    mut no_hover_q: Query<&mut DrawMode, Without<Hovered>>,
) {
    for mut draw in hover_q.iter_mut() {
        if let DrawMode::Outlined {
            ref mut fill_mode, ..
        } = *draw
        {
            fill_mode.color = *fill_mode.color.set_a(0.5);
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

pub fn select_clicked_system(
    mut commands: Commands,
    hover_q: Query<Entity, With<Hovered>>,
    selected_q: Query<Entity, With<Selected>>,
    mouse_click: Res<Input<MouseButton>>,
) {
    for hovered in hover_q.iter() {
        if mouse_click.pressed(MouseButton::Left) {
            if let Ok(clicked_selected) = selected_q.get(hovered) {
                info!("{:?} was clicked while already selected", hovered);
                commands.entity(clicked_selected).remove::<Selected>();
            } else {
                commands.entity(hovered).insert(Selected);
                info!("{:?} was clicked while not yet selected", hovered);
            }
        }
    }
}

/// Highlight a shape by making it more opaque when the mouse hovers over it.
pub fn highlight_selected_sytem(
    // We need all shapes the mouse hovers over.
    mut selected_q: Query<&mut DrawMode, With<Selected>>,
    mut shape_q: Query<&mut DrawMode, Without<Selected>>,
    deselected: RemovedComponents<Selected>,
) {
    for mut draw in selected_q.iter_mut() {
        if let DrawMode::Outlined {
            ref mut fill_mode, ..
        } = *draw
        {
            fill_mode.color = *fill_mode.color.set_a(0.75);
        }
    }

    for entity in deselected.iter() {
        let mut draw = shape_q.get_mut(entity).unwrap();
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
