pub mod editing;
pub mod import;

use bevy::ecs::archetype::Archetypes;
use bevy::ecs::component::{ComponentId, Components};
use bevy::input::mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::render::camera::{Camera, OrthographicProjection};
use bevy::{prelude::*, render::camera::ScalingMode};

use derive_more::{Deref, DerefMut};

use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::{entity, shapes};

// Set a default alpha-value for most shapes
pub const ALPHA: f32 = 0.1;
pub const WIDTH: f32 = 10.0;

pub const DEFAULT_SCALE: f32 = 10e-2;
pub const DEFAULT_UNITS: f32 = 10e-9;

#[derive(Debug)]
pub struct LayerColors {
    colors: std::iter::Cycle<std::vec::IntoIter<Color>>,
}

impl Default for LayerColors {
    fn default() -> Self {
        Self {
            colors: vec![
                // "ff0000", "00ff00", "0000ff", "ffff00", "00ffff", "ff00ff", "ffffff",
                "648FFF", "785EF0", "DC267F", "FE6100", "FFB000",
            ]
            .into_iter()
            .map(|c| Color::hex(c).unwrap())
            .collect::<Vec<Color>>()
            .into_iter()
            .cycle(),
        }
    }
}

impl LayerColors {
    pub fn get_color(&mut self) -> Color {
        self.colors.next().unwrap()
    }
}

// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// pub enum Shape {
//     Rect,
//     Poly,
// }

// impl Default for Shape {
//     fn default() -> Self {
//         Self::Rect
//     }
// }

// impl Shape {
//     pub fn as_str(&self) -> &'static str {
//         match self {
//             Self::Rect => "RECT",
//             Self::Poly => "POLY",
//         }
//     }
// }

// #[derive(Debug, Default, Clone, Copy)]
// pub struct DrawShapeEvent {
//     pub layer: LayerNum,
//     pub shape: Shape,
// }

#[derive(Debug, Default, Clone, Copy)]
pub struct LoadProtoEvent;
#[derive(Debug, Default, Clone, Copy)]
pub struct LoadCompleteEvent;

fn main() {
    App::build()
        .add_event::<LoadProtoEvent>()
        .add_event::<LoadCompleteEvent>()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(WindowDescriptor {
            title: "Doug CAD".to_string(),
            width: 1920.0,
            height: 1080.0,
            vsync: true,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 8 })
        .insert_resource(LayerColors::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .init_resource::<EventTriggerState>()
        .add_system(event_trigger_system.system())
        .add_startup_system(setup.system())
        .add_system(load_proto_event_listener_system.system())
        .add_system(cursor_instersect_system.system())
        .add_system(cursor_collider_debug_sync.system())
        .add_system(camera_changed_system.system())
        .run();
}

fn camera_changed_system(camera_q: Query<&Transform, (Changed<Transform>, With<Camera>)>) {
    for c in camera_q.iter() {
        info!("Camera new transform {:?}", c);
    }
}

fn print_mouse_events_system(
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    for event in mouse_button_input_events.iter() {
        info!("{:?}", event);
    }

    for event in mouse_motion_events.iter() {
        info!("{:?}", event);
    }

    for event in cursor_moved_events.iter() {
        info!("{:?}", event);
    }

    for event in mouse_wheel_events.iter() {
        info!("{:?}", event);
    }
}

pub fn cursor_collider_debug_sync(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_q: Query<&mut Transform, With<CursorColliderDebug>>,
    windows: Res<Windows>,
    camera_q: Query<&Transform, (With<Camera>, Without<CursorColliderDebug>)>,
) {
    let mut shape_pos = cursor_q.single_mut().unwrap();
    // info!("CursorCollider is entity {}", e.id());
    let scale = camera_q.single().unwrap().scale.x;
    // cursor_pos.scale.x = 200.0;
    // cursor_pos.scale.y = 200.0;

    // let Transform {
    //     translation, scale, ..
    // } = camera_q.single().unwrap();

    let window = windows.get_primary().unwrap();

    let width = window.width();
    let height = window.height();

    // info!(
    //     "Window width: {} height: {}",
    //     window.width(),
    //     window.height()
    // );

    if let Some(cursor_pos) = cursor_moved_events.iter().last() {
        let x = cursor_pos.position.x;
        let y = cursor_pos.position.y;

        let off_x = width + 15.0 * scale;
        let off_y = height - 1.0 * scale;

        let new_x = x * scale - off_x;
        let new_y = y * scale - off_y;

        info!("x: {} [{}], y: {} [{}]", new_x, off_x, new_y, off_y);

        shape_pos.translation.x = new_x;
        shape_pos.translation.y = new_y;

        // collider_pos.translation = point![x, y].into();

        // info!(
        //     "CursorCollider(unique) entity {:?} shape_pos {:?} cursor_pos {:?} scale {:?}",
        //     e.id(),
        //     shape_pos.translation,
        //     cursor_pos.position,
        //     scale
        // );
    }
}

pub fn get_components_for_entity<'a>(
    entity: Entity,
    archetypes: &'a Archetypes,
) -> Option<impl Iterator<Item = ComponentId> + 'a> {
    for archetype in archetypes.iter() {
        if archetype.entities().contains(&entity) {
            return Some(archetype.components());
        }
    }
    None
}

/* Project a point inside of a system. */
fn cursor_instersect_system(
    archetypes: &Archetypes,
    components: &Components,
    cursor_collider_q: Query<&Transform, With<CursorColliderDebug>>,
    entity_shape_query: Query<(&Visible, &InLayer, &import::Rect)>,
    windows: Res<Windows>,
    camera_q: Query<(&GlobalTransform, &Camera), Without<CursorColliderDebug>>,
) {
    let (cam_t, cam) = camera_q.single().unwrap();

    let window = windows.get(cam.window).unwrap();
    let window_size = Vec2::new(window.width(), window.height());

    // Convert screen position [0..resolution] to ndc [-1..1]
    let ndc_to_world = cam_t.compute_matrix() * cam.projection_matrix.inverse();

    let screen_pos = cursor_collider_q.single().unwrap().translation.truncate();

    let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
    world_pos.truncate();

    let collider_t = cursor_collider_q.single().unwrap();
}

#[derive(Debug, Default)]
pub struct CursorColliderDebug;

#[derive(Default, Bundle)]
struct CursorColliderBundle {
    pub cursor: CursorColliderDebug,
    #[bundle]
    pub shape_lyon: entity::ShapeBundle,
}

struct EventTriggerState {
    event_timer: Timer,
}

impl Default for EventTriggerState {
    fn default() -> Self {
        EventTriggerState {
            event_timer: Timer::from_seconds(0.001, true),
        }
    }
}

// sends event after 1 second
fn event_trigger_system(
    time: Res<Time>,
    mut state: ResMut<EventTriggerState>,
    mut my_events: EventWriter<LoadProtoEvent>,
) {
    state.event_timer.tick(time.delta());
    let timer = &mut state.event_timer;
    if timer.finished() && !timer.paused() {
        my_events.send(LoadProtoEvent);
        timer.pause()
    }
}

// // prints events as they come in
// fn draw_shape_event_listener_system(
//     mut events: EventReader<LoadCompleteEvent>,
//     mut commands: Commands,
//     // color_query: Query<(&LayerNum, &Color), With<Layer>>,
// ) {
//     for load_complete_event in events.iter() {
//         // test_spawn_path(&mut commands, &color_query);
//         // info!(
//         //     "Added {:?} to {:?}",
//         //     draw_shape_event.shape, draw_shape_event.layer
//         // );
//     }
// }

// prints events as they come in
fn load_proto_event_listener_system(
    mut events: EventReader<LoadProtoEvent>,
    mut commands: Commands,
    mut layer_colors: ResMut<LayerColors>,
    mut load_complete_event_writer: EventWriter<LoadCompleteEvent>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection)>,
) {
    for _ in events.iter() {
        let t = std::time::Instant::now();
        import::test_load_proto_lib(
            &mut commands,
            &mut layer_colors,
            &mut load_complete_event_writer,
            &mut query,
        );
        let d = t.elapsed();
        info!("{:?}", d);
    }
}

fn setup(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();

    camera.orthographic_projection.scaling_mode = ScalingMode::WindowSize;

    commands.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(1000.0, 10.0, 2000.0)),
        light: Light {
            intensity: 100_000_000_.0,
            range: 6000.0,
            ..Default::default()
        },
        ..Default::default()
    });

    info!("{:?}", camera.transform);
    commands.spawn_bundle(camera);

    let rect = shapes::Circle {
        radius: 5.0,
        center: [0.0, 0.0].into(),
    };

    let transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));

    let shape_lyon = GeometryBuilder::build_as(
        &rect,
        ShapeColors {
            main: Color::hex("FFFFFF").unwrap(),
            outline: Color::hex("FFFFFF").unwrap(),
        },
        DrawMode::Outlined {
            fill_options: FillOptions::default(),
            outline_options: StrokeOptions::default(),
        },
        transform,
    );

    let cursor_collider = CursorColliderBundle {
        // collider: ColliderBundle {
        //     shape: ColliderShape::ball(5.0),
        //     flags: (ActiveEvents::INTERSECTION_EVENTS | ActiveEvents::CONTACT_EVENTS).into(),
        //     ..Default::default()
        // },
        shape_lyon,
        ..Default::default()
    };
    commands.spawn_bundle(cursor_collider);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Layer;

#[derive(Debug, Default, Bundle, Clone, Copy)]
pub struct LayerBundle {
    pub layer: Layer,
    pub num: LayerNum,
    pub color: LayerColor,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LayerColor(pub Color);

#[derive(Debug, Clone)]
pub struct InLayer(pub u16);

impl Default for InLayer {
    fn default() -> Self {
        InLayer(0)
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deref, DerefMut)]
pub struct LayerNum(pub u16);
