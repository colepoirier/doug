use bevy::{input::mouse::MouseButtonInput, prelude::*};

use bevy::input::mouse::MouseMotion;

// use crate::import::{Height, Origin, Rect, Width};

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
