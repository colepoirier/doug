use crate::{
    import::Net,
    shapes::{Path, Poly, Rect},
    CursorWorldPos, InLayer, ALPHA,
};
use bevy::prelude::*;
use bevy_prototype_lyon::plugin::ShapePlugin;
use bevy_prototype_lyon::prelude::{DrawMode, FillRule, Path as LyonPath};

use lyon_algorithms::hit_test::hit_test_path;
use lyon_geom::Translation;

pub struct EditingPlugin;

impl Plugin for EditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ShapePlugin)
            .add_stage_after(CoreStage::Update, "detect_clicked", SystemStage::parallel())
            .add_stage_after(
                "detect_clicked",
                "highlight_selected",
                SystemStage::parallel(),
            )
            .add_stage_after(
                "highlight_selected",
                "unhighlight_selected",
                SystemStage::parallel(),
            )
            .add_system_to_stage(CoreStage::Update, cursor_hover_system)
            .add_system_to_stage(CoreStage::PostUpdate, highlight_hovered_system)
            .add_system_to_stage("detect_clicked", select_clicked_system)
            .add_system_to_stage("highlight_selected", highlight_selected_sytem)
            .add_system_to_stage("unhighlight_selected", unhighlight_deselected_system)
            .add_system(print_hovered_info_system)
            .add_system(print_selected_info_system);
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

/// Resource to calculate the shape the cursor interacted with by layer/z-order
/// Layer 0 is furthest from the camera/screen, Layer 999 is closest to the camera
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
    rect_q: Query<(Entity, &LyonPath, &Transform, &InLayer), With<Rect>>,
    poly_q: Query<(Entity, &LyonPath, &Transform, &InLayer), With<Poly>>,
    path_q: Query<(Entity, &LyonPath, &Transform, &InLayer), With<Path>>,
    hovered_q: Query<Entity, With<Hovered>>,
) {
    if cursor_pos.is_changed() {
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
            }
        }

        if let Some(e) = top_shape.shape {
            for hovered in hovered_q.iter() {
                if e != hovered {
                    commands.entity(hovered).remove::<Hovered>();
                }
            }
            commands.entity(e).insert(Hovered);
        } else {
            for hovered in hovered_q.iter() {
                commands.entity(hovered).remove::<Hovered>();
            }
        }
    }
}

/// Highlight a shape as Hovered by making it more opaque when the mouse hovers over it.
pub fn highlight_hovered_system(
    mut hovered_q: Query<&mut DrawMode, Added<Hovered>>,
    mut shape_q: Query<&mut DrawMode, Without<Hovered>>,
    removed_hovered: RemovedComponents<Hovered>,
) {
    for mut draw in hovered_q.iter_mut() {
        if let DrawMode::Outlined {
            ref mut fill_mode, ..
        } = *draw
        {
            fill_mode.color = *fill_mode.color.set_a(0.5);
        }
    }

    for entity in removed_hovered.iter() {
        if let Ok(mut draw) = shape_q.get_mut(entity) {
            if let DrawMode::Outlined {
                ref mut fill_mode, ..
            } = *draw
            {
                fill_mode.color = *fill_mode.color.set_a(ALPHA);
            }
        }
    }
}

pub fn select_clicked_system(
    mut commands: Commands,
    hovered_q: Query<Entity, With<Hovered>>,
    selected_q: Query<Entity, With<Selected>>,
    mouse_click: Res<Input<MouseButton>>,
) {
    if mouse_click.just_pressed(MouseButton::Left) {
        for selected in selected_q.iter() {
            commands.entity(selected).remove::<Selected>();
        }
        for hovered in hovered_q.iter() {
            if selected_q.get(hovered).is_ok() {
                commands.entity(hovered).remove::<Selected>();
            } else {
                commands.entity(hovered).insert(Selected);
            }
        }
    }
}

/// Highlight a shape as selected by making it more opaque than the Hovered opacity when it is clicked.
pub fn highlight_selected_sytem(mut curr_selected_q: Query<&mut DrawMode, With<Selected>>) {
    for mut draw in curr_selected_q.iter_mut() {
        if let DrawMode::Outlined {
            ref mut fill_mode, ..
        } = *draw
        {
            fill_mode.color = *fill_mode.color.set_a(0.75);
        }
    }
}

pub fn unhighlight_deselected_system(
    query: Query<Entity>,
    mut draw_q: Query<&mut DrawMode>,
    deselected: RemovedComponents<Selected>,
) {
    for entity in deselected.iter() {
        if let Ok(mut draw_mode) = draw_q.get_mut(entity) {
            if let DrawMode::Outlined {
                ref mut fill_mode, ..
            } = *draw_mode
            {
                if query.get_component::<Hovered>(entity).is_ok() {
                    fill_mode.color = *fill_mode.color.set_a(0.5);
                } else {
                    fill_mode.color = *fill_mode.color.set_a(ALPHA);
                }
            }
        }
    }
}

pub fn print_hovered_info_system(query: Query<(Entity, &Net, &InLayer), Added<Hovered>>) {
    for (e, net, layer) in query.iter() {
        info!("Hovered: entity: {e:?}, net: {net:?}, layer: {layer:?}.");
    }
}

pub fn print_selected_info_system(query: Query<(Entity, &Net, &InLayer), Added<Selected>>) {
    for (e, net, layer) in query.iter() {
        info!("Selected: entity: {e:?}, net: {net:?}, layer: {layer:?}.");
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
