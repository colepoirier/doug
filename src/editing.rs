use crate::{
    get_component_names_for_entity,
    import::Net,
    screen_to_world_pos,
    shapes::{GeoRect, Path, Poly, Rect},
    CursorWorldPos, InLayer, ALPHA,
};
use bevy::{
    ecs::{archetype::Archetypes, component::Components},
    prelude::*,
};
use bevy_egui::EguiContext;
use bevy_prototype_lyon::plugin::ShapePlugin;
use bevy_prototype_lyon::prelude::{
    shapes as lyon_shapes, DrawMode, FillMode, FillOptions, FillRule, GeometryBuilder,
    Path as LyonPath, StrokeMode, StrokeOptions,
};

use geo::{coord, intersects::Intersects, translate::Translate};
use lyon_algorithms::hit_test::hit_test_path;
use lyon_geom::Translation;

use sorted_vec::SortedVec;

pub struct EditingPlugin;

impl Plugin for EditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ShapePlugin)
            .insert_resource(ShapeStack::default())
            .insert_resource(UndoRedoHistory::default())
            .insert_resource(PointerInitialPos::default())
            .add_event::<Interaction>()
            .add_event::<UndoRedoEvent>()
            .add_event::<PreDragPosEvent>()
            .add_stage_after(CoreStage::Update, "pointer_events", SystemStage::parallel())
            .add_stage_after("pointer_events", "set_hovered", SystemStage::parallel())
            .add_stage_after("set_hovered", "detect_clicked", SystemStage::parallel())
            .add_stage_after(
                "detect_clicked",
                "transform_at_drag_start",
                SystemStage::parallel(),
            )
            .add_stage_after(
                "transform_at_drag_start",
                "click_and_drag",
                SystemStage::parallel(),
            )
            .add_stage_after("click_and_drag", "highlight", SystemStage::parallel())
            .add_stage_after("click_and_drag", "undo_redo_track", SystemStage::parallel())
            .add_stage_after(
                "undo_redo_track",
                "undo_redo_debug",
                SystemStage::parallel(),
            )
            .add_system_to_stage(CoreStage::Update, cursor_hover_detect_system)
            .add_system_to_stage("transform_at_drag_start", dragged_shape_initial_pos_system)
            .add_system_to_stage("undo_redo_track", undo_redo_tracking_system)
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
            .add_system(print_selected_info_system)
            .add_system(undo_redo_key_combo_system)
            .add_system(undo_redo_system)
            .add_system_to_stage("undo_redo_debug", debug_undo_redo_system)
            // .add_system(debug_selection_box_components)
            .add_system_to_stage("click_and_drag", click_and_drag_shape_system)
            .add_system_to_stage("click_and_drag", selection_box_selection_system);
    }
}

fn debug_selection_box_components(
    selection_box_q: Query<Entity, With<SelectionBox>>,
    world: &World,
) {
    if let Ok(entity) = selection_box_q.get_single() {
        info!(
            "SelectionBox components: {:?}",
            get_component_names_for_entity(entity, &world.archetypes(), &world.components())
        );
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
    mut egui_ctx: ResMut<EguiContext>,
) {
    if input_mouse.just_pressed(MouseButton::Left) && !egui_ctx.ctx_mut().wants_pointer_input() {
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
                    if input_mouse.just_released(MouseButton::Left) {
                        interaction_ev.send(Interaction::DragEnd);
                        *drag_started = false;
                        *pointer_initial_pos = PointerInitialPos(None);
                    }
                    return;
                }
            }
            None => {
                if input_mouse.just_released(MouseButton::Left) {
                    interaction_ev.send(Interaction::DragEnd);
                    *drag_started = false;
                    *pointer_initial_pos = PointerInitialPos(None);
                }
                return;
            }
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
    selection_box_q: Query<Entity, With<SelectionBox>>,
) {
    // TODO: add delta so shape stack does not reset if mouse moves a tiny bit while
    // changing the active shape in the shape stack
    if cursor_pos.is_changed()
        && !input_mouse.pressed(MouseButton::Left)
        && selection_box_q.get_single().is_err()
    {
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
        info!("EVENT: {ev:?}");
        if hovered_q.is_empty() {
            for selected in selected_q.iter() {
                info!("Nothing Hovered, removing Selected from: {selected:?}");
                commands.entity(selected).remove::<Selected>();
            }
        }

        if ev == DragEnd {
            for dragging_e in dragging_q.iter() {
                info!("Drag end: removing Dragging from: {dragging_e:?}");
                commands.entity(dragging_e).remove::<Dragging>();
            }
        }

        for hovered in hovered_q.iter() {
            // logic if the user is holding the LAlt key
            if keyboard.pressed(KeyCode::LAlt) {
                if ev == Click {
                    // if the hovered shape that was clicked is already selected, deselect it
                    if selected_q.get(hovered).is_ok() {
                        info!("LAlt held and Hovered shape already selected, and clicked, Removing Selected from: {hovered:?}");
                        commands.entity(hovered).remove::<Selected>();
                    }
                    // if the hoverered shape that was clicked is not already selected, select it
                    else {
                        // mark the shape that was hovered when the click happened as selected
                        info!("LAlt held and Hovered shape clicked, Inserting Selected on: {hovered:?}");
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
                            info!("    multiple shapes were selected and one of them was clicked");
                            // remove the Selected marker component from all shapes except for the clicked shape
                            if hovered_q.get(selected).is_err() {
                                info!("        Removing Selected from: {selected:?}");
                                commands.entity(selected).remove::<Selected>();
                            }
                        }
                    } else if ev == DragStart {
                        info!("multiple shapes and drag");
                        if let Ok(_) = selected_q.get(hovered) {
                            info!("    one of the shapes that was selected was the shape that was hovered when the drag started");
                            for selected in selected_q.iter() {
                                info!("        Inserting Dragging on: {selected:?}");
                                commands.entity(selected).insert(Dragging);
                            }
                        } else {
                            info!("    a shape that was not selected was hovered when the drag started");
                            for selected in selected_q.iter() {
                                info!("        Removing Selected on: {selected:?}");
                                commands.entity(selected).remove::<Selected>();
                            }
                            info!("    Inserting Dragging on: {hovered:?}");
                            commands.entity(hovered).insert(Dragging);
                            info!("    Inserting Selected on: {hovered:?}");
                            commands.entity(hovered).insert(Selected);
                        }
                    } else if ev == DragEnd {
                        for dragging in dragging_q.iter() {
                            info!("    Removing Dragging on: {dragging:?}");
                            commands.entity(dragging).remove::<Dragging>();
                        }
                    }
                }
                // if there is exactly one shape currently selected when the click/drag happened
                else if selected_q.get_single().is_ok() {
                    info!("exactly one shape is selected");
                    if selected_q.get(hovered).is_ok() {
                        // if the hovered shape that was clicked is already selected, deselect it
                        if ev == Click {
                            info!("    exactly one shape is selected and hovered, and click");
                            info!("    Removing Selected from: {hovered:?}");
                            commands.entity(hovered).remove::<Selected>();
                        }
                        if ev == DragStart {
                            info!("    exactly one shape is selected and was teh shape hovered when drag start");
                            info!("    Inserting Dragging on: {hovered:?}");
                            commands.entity(hovered).insert(Dragging);
                        }
                    }
                    // if the shape that is hovered is not selected, then regardless of whether
                    // click/drag run this
                    else if selected_q.get(hovered).is_err() {
                        info!("    selected shape is not hovered");
                        // deselect all previously selected shapes before marking the
                        // shape that was hovered when the click happened as selected
                        for selected in selected_q.iter() {
                            info!("        Removing Selected from: {selected:?}");
                            commands.entity(selected).remove::<Selected>();
                        }
                        if ev == DragStart {
                            info!("    Inserting Dragging on: {hovered:?}");
                            commands.entity(hovered).insert(Dragging);
                        }
                        // mark the shape that was hovered when the click happened as selected
                        info!("    Inserting Selected on: {hovered:?}");
                        commands.entity(hovered).insert(Selected);
                    }
                } else if selected_q.get_single().is_err() && ev == DragStart {
                    info!("no shape is currently selected");
                    // mark the shape that was hovered when the click happened as selected
                    info!("    Inserting Dragging on: {hovered:?}");
                    commands.entity(hovered).insert(Dragging);
                    info!("    Inserting Selected on: {hovered:?}");
                    commands.entity(hovered).insert(Selected);
                } else if selected_q.get_single().is_err() && ev == Click {
                    info!("no shape is currently selected");
                    // mark the shape that was hovered when the click happened as selected
                    info!("    Inserting Dragging on: {hovered:?}");
                    commands.entity(hovered).insert(Selected);
                }
            }
        }
    }
}

#[derive(Component)]
pub struct SelectionBox;

#[derive(Component, Deref, DerefMut, Debug)]
pub struct DeltaWidthHeight(pub IVec2);

#[derive(Bundle)]
pub struct SelectionBoxBundle {
    pub rect: Rect,
    pub delta_rect: DeltaWidthHeight,
    marker: SelectionBox,
}

fn spawn_despawn_selection_box_system(
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    mut interaction_ev: EventReader<Interaction>,
    selection_box_q: Query<Entity, With<SelectionBox>>,
    selected_q: Query<Entity, With<Selected>>,
) {
    use crate::editing::Interaction::*;

    for &ev in interaction_ev.iter() {
        match ev {
            DragStart => {
                if keyboard.pressed(KeyCode::LAlt) {
                    info!("Spawn SelectionBox");
                    commands.spawn().insert(SelectionBox);
                    // Remove selected from all currently selected entities when a SelectionBox starts
                    for selected_e in selected_q.iter() {
                        commands.entity(selected_e).remove::<Selected>();
                    }
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
    mut selection_box_q: Query<
        (Entity, &mut Rect, &mut DeltaWidthHeight),
        (With<SelectionBox>, With<LyonPath>),
    >,
) {
    let draw_mode = DrawMode::Outlined {
        fill_mode: FillMode {
            options: FillOptions::default(),
            color: Color::rgba(1.0, 1.0, 1.0, 0.25),
        },
        outline_mode: StrokeMode {
            options: StrokeOptions::default().with_line_width(3.0),
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
        },
    };

    if let Ok(e) = new_selection_box_q.get_single() {
        let lyon_rect = lyon_shapes::Rectangle {
            origin: lyon_shapes::RectangleOrigin::BottomLeft,
            extents: (0.0, 0.0).into(),
        };

        *initial_world_pos = screen_to_world_pos(&windows, &camera_q, pointer_initial_pos.unwrap());
        let transform =
            Transform::from_translation(Vec3::new(initial_world_pos.x, initial_world_pos.y, 800.0));

        let selection_box = GeometryBuilder::build_as(&lyon_rect, draw_mode, transform);
        commands
            .entity(e)
            .insert_bundle(selection_box)
            .insert(Rect(GeoRect::new(
                (initial_world_pos.x as i32, initial_world_pos.y as i32),
                (initial_world_pos.x as i32, initial_world_pos.y as i32),
            )))
            .insert(DeltaWidthHeight((0, 0).into()));
    }

    if let Ok((sb_e, mut rect, mut delta_wh)) = selection_box_q.get_single_mut() {
        let delta = **cursor_world_pos - *initial_world_pos;
        let new_rect = Rect(GeoRect::new(
            (cursor_world_pos.x as i32, cursor_world_pos.y as i32),
            (initial_world_pos.x as i32, initial_world_pos.y as i32),
        ));
        delta_wh.x = new_rect.width() - rect.width();
        delta_wh.y = new_rect.height() - rect.height();
        *rect = new_rect;

        let lyon_rect = lyon_shapes::Rectangle {
            origin: lyon_shapes::RectangleOrigin::BottomLeft,
            extents: (delta.x, delta.y).into(),
        };

        let transform =
            Transform::from_translation(Vec3::new(initial_world_pos.x, initial_world_pos.y, 800.0));

        let selection_box = GeometryBuilder::build_as(&lyon_rect, draw_mode, transform);
        commands.entity(sb_e).insert_bundle(selection_box);
    }
}

pub fn selection_box_selection_system(
    mut commands: Commands,
    sb_q: Query<(&Rect, &DeltaWidthHeight), (With<SelectionBox>, Changed<Rect>)>,
    rect_q: Query<(Entity, &Rect, &Transform), (Without<Selected>, Without<SelectionBox>)>,
    poly_q: Query<(Entity, &Poly, &Transform), Without<Selected>>,
    path_q: Query<(Entity, &Path, &Transform), Without<Selected>>,
    selected_rect_q: Query<(Entity, &Rect, &Transform), With<Selected>>,
    selected_poly_q: Query<(Entity, &Poly, &Transform), With<Selected>>,
    selected_path_q: Query<(Entity, &Path, &Transform), With<Selected>>,
) {
    for (selection_r, delta_wh) in sb_q.iter() {
        if delta_wh.x < 0 || delta_wh.y < 0 {
            // do deselection
            for (e, r, t) in selected_rect_q.iter() {
                if !selection_r
                    .intersects(&r.translate(t.translation.x as i32, t.translation.y as i32))
                {
                    commands.entity(e).remove::<Selected>();
                }
            }

            for (e, p, t) in selected_poly_q.iter() {
                if !selection_r
                    .intersects(&p.translate(t.translation.x as i32, t.translation.y as i32))
                {
                    commands.entity(e).remove::<Selected>();
                }
            }
        } else {
            // do selection
            for (e, r, t) in rect_q.iter() {
                let transformed_coords =
                    r.translate(t.translation.x as i32, t.translation.y as i32);
                // info!(
                //     "selection_r: {selection_r:?}, rect: {r:?}, translation: ({}, {}), transformed: {transformed_coords:?}",
                //     t.translation.x,
                //     t.translation.y
                // );
                if selection_r.intersects(&transformed_coords) {
                    info!("Inserting Selected on {e:?}!");
                    commands.entity(e).insert(Selected);
                }
            }

            for (e, p, t) in poly_q.iter() {
                // info!("selection_r: {selection_r:?}, poly: {p:?}");
                if selection_r
                    .intersects(&p.translate(t.translation.x as i32, t.translation.y as i32))
                {
                    commands.entity(e).insert(Selected);
                }
            }
        }
        // info!("SelectionBox: {selection_r:?}!");
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

#[derive(Debug, Clone, Copy)]
pub struct PreDragPosEvent {
    pub entity: Entity,
    pub pos: Vec2,
}

fn dragged_shape_initial_pos_system(
    transform_q: Query<(Entity, &Transform), Added<Dragging>>,
    mut pre_drag_pos_ev: EventWriter<PreDragPosEvent>,
) {
    if let Some((entity, t)) = transform_q.iter().nth(0) {
        let pos = t.translation.truncate();
        pre_drag_pos_ev.send(PreDragPosEvent { entity, pos });
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

#[derive(Debug, Default)]
pub struct UndoRedoHistory {
    pub offset: usize,
    pub actions: Vec<AtomicAction>,
}

#[derive(Debug, Default)]
pub struct AtomicAction {
    pub action: TranslateAction,
    pub entities: Vec<Entity>,
}

#[derive(Deref, DerefMut, Debug, Default)]
pub struct TranslateAction(pub Vec2);

pub fn undo_redo_tracking_system(
    mut history: ResMut<UndoRedoHistory>,
    dragging_q: Query<Entity, With<Dragging>>,
    transform_q: Query<&Transform, Without<Dragging>>,
    mut interaction_ev: EventReader<Interaction>,
    mut pre_drag_pos_ev: EventReader<PreDragPosEvent>,
    mut initial_shape_pos: Local<(Vec<Entity>, Vec2)>,
    mut dragging_entities: Local<Vec<Entity>>,
) {
    for ev in pre_drag_pos_ev.iter() {
        *initial_shape_pos = (vec![ev.entity], ev.pos);
    }
    for interaction in interaction_ev.iter() {
        match interaction {
            Interaction::DragStart => {
                *dragging_entities = dragging_q.iter().collect::<Vec<Entity>>();
            }
            Interaction::DragEnd => {
                if !dragging_entities.is_empty() {
                    let entity = initial_shape_pos.0[0];
                    let pos = initial_shape_pos.1;
                    let new_t = transform_q.get(entity).unwrap();
                    let dt = new_t.translation.truncate() - pos;
                    history.actions.push(AtomicAction {
                        action: TranslateAction(dt),
                        entities: (*dragging_entities).clone(),
                    });
                    history.offset += 1;
                    *dragging_entities = vec![];
                }
            }
            _ => (),
        }
    }
}

pub fn debug_undo_redo_system(history: Res<UndoRedoHistory>) {
    if history.is_changed() {
        info!("{history:?}");
    }
}

pub enum UndoRedoEvent {
    Undo,
    Redo,
}

pub fn undo_redo_key_combo_system(
    keyboard: Res<Input<KeyCode>>,
    mut undo_redo_ev: EventWriter<UndoRedoEvent>,
) {
    if keyboard.pressed(KeyCode::LControl) && keyboard.just_pressed(KeyCode::Z) {
        if keyboard.pressed(KeyCode::LShift) {
            info!("shift-ctrl-z!");
            undo_redo_ev.send(UndoRedoEvent::Redo);
        } else {
            info!("ctrl-z!");
            undo_redo_ev.send(UndoRedoEvent::Undo);
        }
    }
}

impl UndoRedoHistory {
    fn undo_action(&mut self, transform_q: &mut Query<&mut Transform>) {
        let AtomicAction { action, entities } = &self.actions[self.offset - 1];
        for e in entities {
            if let Ok(mut t) = transform_q.get_mut(*e) {
                t.translation -= (*action).extend(0.0);
            }
        }
        self.offset -= 1;
    }

    fn redo_action(&mut self, transform_q: &mut Query<&mut Transform>) {
        let AtomicAction { action, entities } = &self.actions[self.offset];
        for e in entities {
            if let Ok(mut t) = transform_q.get_mut(*e) {
                t.translation += (*action).extend(0.0);
            }
        }
        self.offset += 1;
    }
}

pub fn undo_redo_system(
    mut undo_redo_ev: EventReader<UndoRedoEvent>,
    mut undo_redo_history: ResMut<UndoRedoHistory>,
    mut transform_q: Query<&mut Transform>,
) {
    for ev in undo_redo_ev.iter() {
        use UndoRedoEvent::*;
        match ev {
            Undo => {
                if undo_redo_history.offset > 0 && undo_redo_history.actions.len() > 0 {
                    undo_redo_history.undo_action(&mut transform_q);
                }
            }
            Redo => {
                if undo_redo_history.actions.len() > 0
                    && undo_redo_history.offset < undo_redo_history.actions.len()
                {
                    undo_redo_history.redo_action(&mut transform_q);
                }
            }
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
