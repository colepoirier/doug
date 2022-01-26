use crate::{
    shapes::{Path, Poly, Rect},
    ALPHA,
};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{DrawMode, FillMode};
use derive_more::{Deref, DerefMut};

/// Marker component to indicate that the mouse
/// currently hovers over the given entity.
#[derive(Component)]
pub struct Hover;

pub fn hover_rect_system(
    mut commands: Commands,
    mut rect_q: Query<(Entity, &Rect, &mut DrawMode)>,
    mut cursor_pos: EventReader<CursorMoved>,
    windows: Res<Windows>,
    camera_q: Query<(&Transform, &Camera)>,
) {
    let (cam_t, cam) = camera_q.single();

    let window = windows.get(cam.window).unwrap();
    let window_size = Vec2::new(window.width(), window.height());

    // Convert screen position [0..resolution] to ndc [-1..1]
    let ndc_to_world = cam_t.compute_matrix() * cam.projection_matrix.inverse();

    for cursor in cursor_pos.iter() {
        let (x, y) = cursor.position.into();
        let ndc = (Vec2::new(x, y) / window_size) * 2.0 - Vec2::ONE;
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        let (x, y) = world_pos.truncate().into();
        info!("hover_rect_system cursor pos x {} y {}", x, y);
        for (
            entity,
            &Rect {
                width,
                height,
                origin,
            },
            mut draw,
        ) in rect_q.iter_mut()
        {
            let x_min = origin.x;
            let x_max = origin.x + (width as i32);
            let y_min = origin.y;
            let y_max = origin.y + (height as i32);
            // info!(
            //     "hover_rect_system rect dims x_min {} x_max {}  y_min {} y_max {}",
            //     x_min, x_max, y_min, y_max
            // );
            if (x_min < (x.round() as i32) && (x.round() as i32) < x_max)
                && (y_min < (y.round() as i32) && (y.round() as i32) < y_max)
            {
                commands.entity(entity).insert(Hover);
                info!("hover_rect_system is_hovered id: {:?}, dims: x_min {} x_max {}  y_min {} y_max {}", entity, x_min, x_max, y_min, y_max);
            } else {
                commands.entity(entity).remove::<Hover>();
                info!("hover_rect_system no longer is_hovered id: {:?}, dims: x_min {} x_max {}  y_min {} y_max {}", entity, x_min, x_max, y_min, y_max);
            }
        }
    }
}

/// Highlight a connector by increasing its radius when the mouse
/// hovers over it.
pub fn highlight_shape_system(
    // We need all connectors the mouse hovers over.
    mut q_hover: Query<(Entity, &mut DrawMode, &mut Transform), With<Hover>>,
    mut q2_hover: Query<(Entity, &mut DrawMode, &mut Transform), Without<Hover>>,
) {
    for (entity, mut draw, mut transform) in q_hover.iter_mut() {
        if let DrawMode::Outlined {
            ref mut fill_mode, ..
        } = *draw
        {
            fill_mode.color = Color::rgba(0.0, 0.0, 0.0, 1.0);
        }
        // transform.scale.x = 1.2;
        // transform.scale.y = 1.2;
        info!("highlight_shape_system is_hovered id: {:?}", entity);
    }

    for (entity, mut draw, mut transform) in q2_hover.iter_mut() {
        if let DrawMode::Outlined {
            ref mut fill_mode, ..
        } = *draw
        {
            fill_mode.color = *fill_mode.color.set_a(ALPHA);
        }
        // transform.scale.x = 1.0;
        // transform.scale.y = 1.0;
        info!("highlight_shape_system not is_hovered id: {:?}", entity);
    }
}

pub fn hover_poly_system(rect_q: Query<&Poly>, cursor_pos: Res<CursorMoved>) {}

pub fn hover_path_system(rect_q: Query<&Path>, cursor_pos: Res<CursorMoved>) {}

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

// impl Selectable for Rect {
//     type Selection = RectSelection;
//     fn select(
//         &self,
//         target: Entity,
//         mouse_pos: Vec2,
//         click_event: MouseButtonInput,
//         dimensions: impl Dimensions,
//     ) -> Option<Box<Self::Selection>> {
//         None
//     }
// }

// trait Dimensions: Component {}

// pub trait Selectable: Component + 'static {
//     type Selection: SelectionType;
//     fn select(
//         &self,
//         target: Entity,
//         mouse_pos: &Vec2,
//         click_event: &MouseButtonInput,
//         dimensions: Dimensions,
//     ) -> Option<Box<Self::Selection>>;
// }

// struct CurrentlySelected {
//     entity: Entity,
//     selection_type: Box<dyn SelectionType>,
// }

// trait SelectionType: Send + Sync {}

// impl SelectionType for RectSelection {}

// fn get_selection_system<S: Selectable, D: Dimensions>(
//     click_events: EventReader<MouseButtonInput>,
//     cursor_pos: EventReader<CursorMoved>,
//     query: Query<(Entity, &S, &D)>,
//     currently_selected: ResMut<CurrentlySelected>,
// ) {
//     // Look up the actual event strategy for detecting clicks
//     for click_event in click_events.iter() {
//         for (entity, selectable, dimensions) in query.iter() {
//             let maybe_selected = selectable.select(
//                 entity,
//                 cursor_pos.iter().nth(0).unwrap().position,
//                 click_event,
//                 dimensions,
//             );
//             if maybe_selected.is_some() {
//                 currently_selected.entity = entity;
//                 currently_selected.selection_type = maybe_selected.unwrap();
//             }
//         }
//     }
// }

// app.add_system(get_selection_system::<Rectangle>);
