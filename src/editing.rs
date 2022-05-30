use crate::{
    import::Net,
    screen_to_world_pos,
    shapes::{Path, Poly, Rect},
    CursorWorldPos, InLayer, ALPHA,
};
use bevy::prelude::*;
use bevy_prototype_lyon::plugin::ShapePlugin;
use bevy_prototype_lyon::prelude::{
    shapes as lyon_shapes, DrawMode, FillMode, FillOptions, FillRule, GeometryBuilder,
    Path as LyonPath, StrokeMode, StrokeOptions,
};

use lyon_algorithms::hit_test::hit_test_path;
use lyon_geom::Translation;

use sorted_vec::SortedVec;

pub struct EditingPlugin;

impl Plugin for EditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ShapePlugin)
            .insert_resource(ShapeStack::default())
            .insert_resource(PointerInitialPos::default())
            .add_event::<Interaction>()
            .add_stage_after(CoreStage::Update, "pointer_events", SystemStage::parallel())
            .add_stage_after("pointer_events", "set_hovered", SystemStage::parallel())
            .add_stage_after("set_hovered", "detect_clicked", SystemStage::parallel())
            .add_stage_after("detect_clicked", "highlight", SystemStage::parallel())
            .add_system_to_stage(CoreStage::Update, cursor_hover_detect_system)
            .add_system_set_to_stage(
                "pointer_events",
                SystemSet::new()
                    .with_system(initialize_pointer_event_determination)
                    .with_system(
                        resolve_pointer_event_determination
                            .after(initialize_pointer_event_determination),
                    ),
            )
            .add_system_to_stage("set_hovered", set_hovered_system)
            .add_system_to_stage("detect_clicked", select_clicked_system)
            .add_system_to_stage("highlight", highlight_hovered_system)
            .add_system_to_stage("highlight", highlight_selected_sytem)
            .add_system_to_stage("highlight", unhighlight_deselected_system)
            .add_system_set(
                SystemSet::new()
                    .with_system(spawn_despawn_selection_box_system)
                    .with_system(
                        draw_selection_box_system.before(spawn_despawn_selection_box_system),
                    ),
            )
            .add_system(cycle_shape_stack_hover_system)
            .add_system(print_hovered_info_system)
            .add_system(click_and_drag_shape_system)
            .add_system(print_selected_info_system);
    }
}

#[derive(Debug, Default, Deref)]
pub struct PointerInitialPos(Option<Vec2>);

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Interaction {
    Click,
    DragStart,
    DragEnd,
}

pub fn initialize_pointer_event_determination(
    windows: Res<Windows>,
    mut pointer_initial_pos: ResMut<PointerInitialPos>,
    input_mouse: Res<Input<MouseButton>>,
) {
    if input_mouse.just_pressed(MouseButton::Left) {
        let window = windows.get_primary().unwrap();

        if let Some(initial_pos) = window.cursor_position() {
            *pointer_initial_pos = PointerInitialPos(Some(initial_pos));
        }
    }
}

pub fn resolve_pointer_event_determination(
    mut pointer_initial_pos: ResMut<PointerInitialPos>,
    windows: Res<Windows>,
    input_mouse: Res<Input<MouseButton>>,
    mut interaction_ev: EventWriter<Interaction>,
    mut drag_started: Local<bool>,
) {
    if let Some(initial_pos) = **pointer_initial_pos {
        let window = windows.get_primary().unwrap();

        let current_pos = match window.cursor_position() {
            Some(pos) => {
                if (0.0 <= pos.x && pos.x <= window.width())
                    && (0.0 <= pos.y && pos.y <= window.height())
                {
                    pos
                } else {
                    return;
                }
            }
            None => return,
        };

        let delta = current_pos - initial_pos;

        if delta.length_squared() > 10.0
            && !input_mouse.just_released(MouseButton::Left)
            && !*drag_started
        {
            interaction_ev.send(Interaction::DragStart);
            *drag_started = true;
        }

        if input_mouse.just_released(MouseButton::Left) {
            if delta.length_squared() < 10.0 {
                interaction_ev.send(Interaction::Click);
            } else {
                interaction_ev.send(Interaction::DragEnd);
                *drag_started = false;
            }
            *pointer_initial_pos = PointerInitialPos(None);
        }
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

/// Marker component to indicate that the given
/// entity is being dragged.
#[derive(Component)]
pub struct Dragging;

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
    input_mouse: Res<Input<MouseButton>>,
) {
    // TODO: add delta so shape stack does not reset if mouse moves a tiny bit wile
    // changing the active shape in the shape stack
    if cursor_pos.is_changed() && !input_mouse.pressed(MouseButton::Left) {
        *shape_stack = ShapeStack::default();

        let point = lyon_geom::point(cursor_pos.x as f32, cursor_pos.y as f32);

        for (entity, path, transform, layer, vis) in rect_q.iter() {
            let layer = **layer;

            let path = path.0.clone().transformed(&Translation::new(
                transform.translation.x,
                transform.translation.y,
            ));

            if hit_test_path(&point, path.iter(), FillRule::NonZero, 0.00000001) && vis.is_visible {
                shape_stack.stack.insert(Shape { layer, entity });
            }
        }

        for (entity, path, transform, layer, vis) in poly_q.iter() {
            let layer = **layer;

            let path = path.0.clone().transformed(&Translation::new(
                transform.translation.x,
                transform.translation.y,
            ));

            if hit_test_path(&point, path.iter(), FillRule::NonZero, 0.00000001) && vis.is_visible {
                shape_stack.stack.insert(Shape { layer, entity });
            }
        }

        for (entity, path, transform, layer, vis) in path_q.iter() {
            let layer = **layer;

            let path = path.0.clone().transformed(&Translation::new(
                transform.translation.x,
                transform.translation.y,
            ));

            if hit_test_path(&point, path.iter(), FillRule::NonZero, 0.00000001) && vis.is_visible {
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
    mut hovered_q: Query<(Entity, &mut DrawMode), Added<Hovered>>,
    mut shape_q: Query<(Entity, &mut DrawMode), Without<Hovered>>,
    selected_q: Query<Entity, With<Selected>>,
    removed_hovered: RemovedComponents<Hovered>,
) {
    'outer_1: for (hovered_e, mut draw) in hovered_q.iter_mut() {
        if let DrawMode::Outlined {
            ref mut fill_mode, ..
        } = *draw
        {
            for selected_e in selected_q.iter() {
                if selected_e == hovered_e {
                    continue 'outer_1;
                }
            }
            fill_mode.color = *fill_mode.color.set_a(0.5);
        }
    }

    'outer_2: for entity in removed_hovered.iter() {
        if let Ok((shape_e, mut draw)) = shape_q.get_mut(entity) {
            if let DrawMode::Outlined {
                ref mut fill_mode, ..
            } = *draw
            {
                for selected_e in selected_q.iter() {
                    if selected_e == shape_e {
                        continue 'outer_2;
                    }
                }
                fill_mode.color = *fill_mode.color.set_a(ALPHA);
            }
        }
    }
}

pub fn select_clicked_system(
    mut commands: Commands,
    hovered_q: Query<Entity, With<Hovered>>,
    selected_q: Query<Entity, With<Selected>>,
    dragging_q: Query<Entity, With<Dragging>>,
    keyboard: Res<Input<KeyCode>>,
    mut interaction_ev: EventReader<Interaction>,
) {
    use crate::editing::Interaction::*;

    for &ev in interaction_ev.iter() {
        if hovered_q.is_empty() {
            for selected in selected_q.iter() {
                info!("Nothing Hovered, removing Selected from: {selected:?}");
                commands.entity(selected).remove::<Selected>();
            }
        }

        if ev == DragEnd {
            for dragging_e in dragging_q.iter() {
                info!("Removing Dragging from: {dragging_e:?}");
                commands.entity(dragging_e).remove::<Dragging>();
            }
        }

        for hovered in hovered_q.iter() {
            // logic if the user is holding the LAlt key
            if keyboard.pressed(KeyCode::LAlt) {
                if ev == Click {
                    // if the hovered shape that was clicked is already selected, deselect it
                    if selected_q.get(hovered).is_ok() {
                        info!("Removing Selected from: {hovered:?}");
                        commands.entity(hovered).remove::<Selected>();
                    }
                    // if the hoverered shape that was clicked is not already selected, select it
                    else {
                        // mark the shape that was hovered when the click happened as selected
                        info!("Inserting Selected on: {hovered:?}");
                        commands.entity(hovered).insert(Selected);
                    }
                }
            }
            // logic if the user is not holding the LAlt key
            else {
                // if there are multiple shapes currently selected (from a previous LAlt held state)
                // deselect all except the the clicked shape
                if !selected_q.is_empty() && selected_q.get_single().is_err() {
                    if ev == Click {
                        info!("multiple shapes and click");
                        // deselect all previously selected shapes before marking the
                        // shape that was hovered when the click happened as selected
                        for selected in selected_q.iter() {
                            info!("multiple shapes were selected and one of them was clicked");
                            // remove the Selected marker component from all shapes except for the clicked shape
                            if hovered_q.get(selected).is_err() {
                                info!("Removing Selected from: {selected:?}");
                                commands.entity(selected).remove::<Selected>();
                            }
                        }
                    } else if ev == DragStart {
                        if let Ok(_) = selected_q.get(hovered) {
                            for selected in selected_q.iter() {
                                info!("Inserting Dragging on: {selected:?}");
                                commands.entity(selected).insert(Dragging);
                            }
                        } else {
                            for selected in selected_q.iter() {
                                commands.entity(selected).remove::<Selected>();
                            }
                            commands.entity(hovered).insert(Dragging);
                            commands.entity(hovered).insert(Selected);
                        }
                    } else if ev == DragEnd {
                        for dragging in dragging_q.iter() {
                            info!("Removing Dragging on: {dragging:?}");
                            commands.entity(dragging).remove::<Dragging>();
                        }
                    }
                }
                // if there is exactly one shape currently selected when the click/drag happened
                else if selected_q.get_single().is_ok() {
                    info!("exactly one shape is selected");
                    // if the hovered shape that was clicked is already selected, deselect it
                    if selected_q.get(hovered).is_ok() {
                        if ev == Click {
                            info!("exactly one shape is selected and hovered, and click");
                            info!("Removing Selected from: {hovered:?}");
                            commands.entity(hovered).remove::<Selected>();
                        }
                        if ev == DragStart {
                            info!("exactly one shape is selected and hovered, and drag");
                            info!("Removing Selected from: {hovered:?}");
                            commands.entity(hovered).insert(Dragging);
                        }
                    }
                    // if the shape that is hovered is not selected, then regardless of whether
                    // click/drag run this
                    else if selected_q.get(hovered).is_err() {
                        info!("selected shape is not hovered");
                        // deselect all previously selected shapes before marking the
                        // shape that was hovered when the click happened as selected
                        for selected in selected_q.iter() {
                            info!("Removing Selected from: {selected:?}");
                            commands.entity(selected).remove::<Selected>();
                        }
                        // mark the shape that was hovered when the click happened as selected
                        info!("Inserting Selected on: {hovered:?}");
                        commands.entity(hovered).insert(Dragging);
                        commands.entity(hovered).insert(Selected);
                    }
                } else if selected_q.get_single().is_err() && ev == DragStart {
                    info!("no shape is currently selected");
                    // mark the shape that was hovered when the click happened as selected
                    info!("Inserting Dragging on: {hovered:?}");
                    commands.entity(hovered).insert(Dragging);
                    commands.entity(hovered).insert(Selected);
                } else if selected_q.get_single().is_err() && ev == Click {
                    info!("no shape is currently selected");
                    // mark the shape that was hovered when the click happened as selected
                    info!("Inserting Dragging on: {hovered:?}");
                    commands.entity(hovered).insert(Selected);
                }
            }
        }
    }
}

#[derive(Component)]
pub struct SelectionBox;

fn spawn_despawn_selection_box_system(
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    mut interaction_ev: EventReader<Interaction>,
    selection_box_q: Query<Entity, With<SelectionBox>>,
) {
    use crate::editing::Interaction::*;

    for &ev in interaction_ev.iter() {
        match ev {
            DragStart => {
                if keyboard.pressed(KeyCode::LAlt) {
                    info!("Spawn SelectionBox");
                    commands.spawn().insert(SelectionBox);
                }
            }
            DragEnd => {
                if let Ok(e) = selection_box_q.get_single() {
                    commands.entity(e).despawn();
                    info!("Despawn SelectionBox");
                    // now, send an event with the lyon shape to the selection system
                }
            }
            _ => continue,
        }
    }
}

pub fn draw_selection_box_system(
    mut commands: Commands,
    windows: Res<Windows>,
    camera_q: Query<(&Transform, &Camera)>,
    pointer_initial_pos: Res<PointerInitialPos>,
    cursor_world_pos: Res<CursorWorldPos>,
    mut initial_world_pos: Local<Vec2>,
    new_selection_box_q: Query<Entity, Added<SelectionBox>>,
    selection_box_q: Query<Entity, (With<SelectionBox>, With<LyonPath>)>,
) {
    if let Ok(e) = new_selection_box_q.get_single() {
        let lyon_rect = lyon_shapes::Rectangle {
            origin: lyon_shapes::RectangleOrigin::BottomLeft,
            extents: (0.0, 0.0).into(),
        };

        *initial_world_pos = screen_to_world_pos(&windows, &camera_q, pointer_initial_pos.unwrap());
        let transform =
            Transform::from_translation(Vec3::new(initial_world_pos.x, initial_world_pos.y, 800.0));

        let selection_box = GeometryBuilder::build_as(
            &lyon_rect,
            DrawMode::Outlined {
                fill_mode: FillMode {
                    color: Color::rgba(1.0, 1.0, 1.0, 0.0),
                    options: FillOptions::default(),
                },
                outline_mode: StrokeMode {
                    options: StrokeOptions::default().with_line_width(3.0),
                    color: Color::rgba(1.0, 1.0, 1.0, 1.0),
                },
            },
            transform,
        );
        commands.entity(e).insert_bundle(selection_box);
    }

    if let Ok(e) = selection_box_q.get_single() {
        let delta = **cursor_world_pos - *initial_world_pos;

        let lyon_rect = lyon_shapes::Rectangle {
            origin: lyon_shapes::RectangleOrigin::BottomLeft,
            extents: (delta.x, delta.y).into(),
        };

        let transform = Transform::from_translation(Vec3::new(initial_world_pos.x, initial_world_pos.y, 800.0));

        let selection_box = GeometryBuilder::build_as(
            &lyon_rect,
            DrawMode::Outlined {
                fill_mode: FillMode {
                    color: Color::rgba(1.0, 1.0, 1.0, 0.0),
                    options: FillOptions::default(),
                },
                outline_mode: StrokeMode {
                    options: StrokeOptions::default().with_line_width(3.0),
                    color: Color::rgba(1.0, 1.0, 1.0, 1.0),
                },
            },
            transform,
        );
        commands.entity(e).insert_bundle(selection_box);
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

pub fn click_and_drag_shape_system(
    input_mouse: Res<Input<MouseButton>>,
    mut dragging_q: Query<&mut Transform, With<Dragging>>,
    cursor_world_pos: Res<CursorWorldPos>,
    mut last_pos: Local<Option<Vec2>>,
) {
    if input_mouse.pressed(MouseButton::Left) {
        let current_pos = **cursor_world_pos;
        let delta = (current_pos - last_pos.unwrap_or(current_pos)).extend(0.0);

        if dragging_q.is_empty() {
            *last_pos = None;
        }

        for mut transform in dragging_q.iter_mut() {
            transform.translation += delta;
        }

        *last_pos = Some(current_pos);
    } else {
        *last_pos = None;
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
