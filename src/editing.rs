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

use sorted_vec::SortedVec;

pub struct EditingPlugin;

impl Plugin for EditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ShapePlugin)
            .insert_resource(ShapeStack::default())
            .add_stage_after(CoreStage::Update, "set_hovered", SystemStage::parallel())
            .add_stage_after("set_hovered", "detect_clicked", SystemStage::parallel())
            .add_stage_after("detect_clicked", "highlight", SystemStage::parallel())
            .add_system_to_stage(CoreStage::Update, cursor_hover_detect_system)
            .add_system_to_stage("set_hovered", set_hovered_system)
            .add_system_to_stage("detect_clicked", select_clicked_system)
            .add_system_to_stage("highlight", highlight_hovered_system)
            .add_system_to_stage("highlight", highlight_selected_sytem)
            .add_system_to_stage("highlight", unhighlight_deselected_system)
            .add_system(cycle_shape_stack_hover_system)
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

#[derive(Debug, Clone, Copy, Eq, Ord)]
pub struct Shape {
    pub layer: u8,
    pub entity: Entity,
}

impl Default for Shape {
    fn default() -> Self {
        Self {
            layer: 0,
            entity: Entity::from_raw(0),
        }
    }
}

impl PartialEq for Shape {
    fn eq(&self, other: &Self) -> bool {
        if self.layer == other.layer {
            true
        } else {
            false
        }
    }
}

impl PartialOrd for Shape {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.layer.partial_cmp(&other.layer)
    }
}

/// Resource to calculate the shape the cursor interacted with by layer/z-order
/// Layer 0 is furthest from the camera/screen, Layer 999 is closest to the camera
#[derive(Clone, Debug, Default)]
pub struct ShapeStack {
    pub offset: isize,
    pub stack: SortedVec<Shape>,
}

pub fn cursor_hover_detect_system(
    cursor_pos: Res<CursorWorldPos>,
    mut shape_stack: ResMut<ShapeStack>,
    rect_q: Query<(Entity, &LyonPath, &Transform, &InLayer, &Visibility), With<Rect>>,
    poly_q: Query<(Entity, &LyonPath, &Transform, &InLayer, &Visibility), With<Poly>>,
    path_q: Query<(Entity, &LyonPath, &Transform, &InLayer, &Visibility), With<Path>>,
) {
    if cursor_pos.is_changed() {
        *shape_stack = ShapeStack::default();

        let point = lyon_geom::point(cursor_pos.x as f32, cursor_pos.y as f32);

        for (entity, path, transform, layer, vis) in rect_q.iter() {
            let layer = **layer;

            let path = path.0.clone().transformed(&Translation::new(
                transform.translation.x,
                transform.translation.y,
            ));

            if hit_test_path(&point, path.iter(), FillRule::NonZero, 0.0) && vis.is_visible {
                shape_stack.stack.insert(Shape { layer, entity });
            }
        }

        for (entity, path, transform, layer, vis) in poly_q.iter() {
            let layer = **layer;

            let path = path.0.clone().transformed(&Translation::new(
                transform.translation.x,
                transform.translation.y,
            ));

            if hit_test_path(&point, path.iter(), FillRule::NonZero, 0.0) && vis.is_visible {
                shape_stack.stack.insert(Shape { layer, entity });
            }
        }

        for (entity, path, transform, layer, vis) in path_q.iter() {
            let layer = **layer;

            let path = path.0.clone().transformed(&Translation::new(
                transform.translation.x,
                transform.translation.y,
            ));

            if hit_test_path(&point, path.iter(), FillRule::NonZero, 0.0) && vis.is_visible  {
                shape_stack.stack.insert(Shape { layer, entity });
            }
        }
    }
}


pub fn set_hovered_system(
    mut commands: Commands,
    shape_stack: Res<ShapeStack>,
    hovered_q: Query<Entity, With<Hovered>>,
) {
    if shape_stack.stack.len() > 0 {
        let offset = shape_stack.offset;
        let stack = shape_stack.stack.iter().rev().collect::<Vec<&Shape>>();

        let index = if offset < 0 {
            (stack.len() as isize + offset) as usize % stack.len()
        } else if offset > 0 {
            offset as usize % stack.len()
        } else {
            0
        };

        let entity = stack[index].entity;

        for hovered in hovered_q.iter() {
            if entity != hovered {
                commands.entity(hovered).remove::<Hovered>();
            }
        }
        commands.entity(entity).insert(Hovered);
    } else {
        for hovered in hovered_q.iter() {
            commands.entity(hovered).remove::<Hovered>();
        }
    }
}

pub fn cycle_shape_stack_hover_system(
    mut shape_stack: ResMut<ShapeStack>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::W) {
        shape_stack.offset += 1;
    } else if keyboard.just_pressed(KeyCode::Q) {
        shape_stack.offset -= 1;
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
    keyboard: Res<Input<KeyCode>>,
) {
    if mouse_click.just_pressed(MouseButton::Left) {
        if hovered_q.is_empty() {
            for selected in selected_q.iter() {
                commands.entity(selected).remove::<Selected>();
            }
        }

        for hovered in hovered_q.iter() {
            // logic if the user is holding the LAlt key
            if keyboard.pressed(KeyCode::LAlt) {
                // if the hovered shape that was clicked is already selected, deselect it
                if selected_q.get(hovered).is_ok() {
                    commands.entity(hovered).remove::<Selected>();
                }
                // if the hoverered shape that was clicked is not already selected, select it
                else {
                    // mark the shape that was hovered when the click happened as selected
                    commands.entity(hovered).insert(Selected);
                }
            }
            // logic if the user is not holding the LAlt key
            else {
                // if there are multiple shapes currently selected (from a previous LAlt held state)
                // deselect all except the the clicked shape
                if !selected_q.is_empty() && selected_q.get_single().is_err() {
                    // deselect all previously selected shapes before marking the
                    // shape that was hovered when the click happened as selected
                    for selected in selected_q.iter() {
                        // remove the Selected marker component from all shapes except for the clicked shape
                        if hovered_q.get(selected).is_err() {
                            commands.entity(selected).remove::<Selected>();
                        }
                    }
                }
                // if there is exactly one shape currently selected when the click happened
                else {
                    // if the hovered shape that was clicked is already selected, deselect it
                    if selected_q.get(hovered).is_ok() {
                        commands.entity(hovered).remove::<Selected>();
                    }
                    // if the hoverered shape that was clicked is not already selected, select it
                    else {
                        // deselect all previously selected shapes before marking the
                        // shape that was hovered when the click happened as selected
                        for selected in selected_q.iter() {
                            commands.entity(selected).remove::<Selected>();
                        }
                        // mark the shape that was hovered when the click happened as selected
                        commands.entity(hovered).insert(Selected);
                    }
                }
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

pub fn print_hovered_info_system(
    query: Query<(Entity, &Net, &InLayer), Added<Hovered>>,
    shape_stack: Res<ShapeStack>,
) {
    for (e, net, layer) in query.iter() {
        info!(
            "Hovered: entity: {e:?}, net: {net:?}, layer: {layer:?}, index: {}.",
            shape_stack.offset
        );
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
