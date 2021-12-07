pub mod editing;
pub mod import;

use std::borrow::Borrow;
use std::ops::Mul;

use bevy::input::mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::render::camera::{Camera, OrthographicProjection};
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_inspector_egui::{
    Inspectable, InspectableRegistry, InspectorPlugin, WorldInspectorParams, WorldInspectorPlugin,
};
use bevy_prototype_lyon::{entity, shapes};

use bevy_rapier2d::rapier::parry::bounding_volume::AABB;
use derive_more::{Deref, DerefMut};

use bevy_prototype_lyon::prelude::*;

use bevy_rapier2d::prelude::*;
use import::Nom;

// use bevy_config_cam::ConfigCam;

// Set a default alpha-value for most shapes
pub const ALPHA: f32 = 0.10;
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

#[derive(Inspectable, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Shape {
    Rect,
    Poly,
}

impl Default for Shape {
    fn default() -> Self {
        Self::Rect
    }
}

impl Shape {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rect => "RECT",
            Self::Poly => "POLY",
        }
    }
}

#[derive(Inspectable, Debug, Default, Clone, Copy)]
pub struct DrawShapeEvent {
    pub layer: LayerNum,
    pub shape: Shape,
}

#[derive(Inspectable, Debug, Default, Clone, Copy)]
pub struct LoadProtoEvent;
#[derive(Inspectable, Debug, Default, Clone, Copy)]
pub struct LoadCompleteEvent;

fn main() {
    App::build()
        .add_event::<LoadProtoEvent>()
        .add_event::<LoadCompleteEvent>()
        .insert_resource(Msaa { samples: 8 })
        .insert_resource(LayerColors::default())
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        // .insert_resource(Vec::<Path>::default())
        .insert_resource(WindowDescriptor {
            title: "Doug CAD".to_string(),
            width: 1920.0,
            height: 1080.0,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        // .add_plugin(InspectorPlugin::<Resources>::new())
        .add_plugin(ShapePlugin)
        // .add_plugin(ConfigCam)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .init_resource::<EventTriggerState>()
        // .add_plugin(NoCameraPlayerPlugin)
        .add_system(event_trigger_system.system())
        .add_startup_system(setup.system())
        .add_system(load_proto_event_listener_system.system())
        .add_system(cursor_instersect_system.system())
        .insert_resource(QueryPipeline::default())
        // .add_system(print_mouse_events_system.system())
        .add_system(cursor_collider_sync.system())
        .add_system_to_stage(CoreStage::PostUpdate, cursor_collider_debug.system())
        // .add_plugin(InspectorPlugin::<CursorColliderBundle>::new())
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

fn cursor_collider_debug(
    mut intersection_events: EventReader<IntersectionEvent>,
    mut contact_events: EventReader<ContactEvent>,
) {
    for intersection_event in intersection_events.iter() {
        info!("Received intersection event: {:?}", intersection_event);
    }

    for contact_event in contact_events.iter() {
        info!("Received contact event: {:?}", contact_event);
    }
}

pub fn cursor_collider_sync(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_q: Query<&mut Transform, With<CursorCollider>>,
    windows: Res<Windows>,
    camera_q: Query<&Transform, (With<Camera>, Without<CursorCollider>)>,
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

/* Project a point inside of a system. */
fn cursor_instersect_system(
    cursor_collider: Query<&Transform, With<CursorCollider>>,
    query_pipeline: Res<QueryPipeline>,
    collider_query: QueryPipelineColliderComponentsQuery,
    mut entity_shape_query: Query<(&mut Visible, &InLayer, &ColliderPosition, &ColliderShape)>,
) {
    for t in cursor_collider.iter() {
        // Wrap the bevy query so it can be used by the query pipeline.
        let collider_set = QueryPipelineColliderComponentsSet(&collider_query);

        let point: Point<Real> = [t.translation.x, t.translation.y].into();
        // info!("Testing for intersections with point {}", point.coords);
        // let solid = true;
        let groups = InteractionGroups::all();
        let filter = None;

        // if let Some((handle, projection)) =
        //     query_pipeline.project_point(&collider_set, &point, solid, groups, filter)
        // {
        //     // The collider closest to the point has this `handle`.
        //     info!(
        //         "Projected point on entity {:?}. Point projection: {}",
        //         handle.entity(),
        //         projection.point
        //     );
        //     info!(
        //         "Point was inside of the collider shape: {}",
        //         projection.is_inside
        //     );
        // }

        query_pipeline.intersections_with_point(&collider_set, &point, groups, filter, |handle| {
            // Callback called on each collider with a shape containing the point.
            // info!("The entity {:?} contains the point.", handle.entity());
            // let mut colors = entity_shape_query.get_mut(handle.entity()).unwrap();

            // colors.outline = Color::rgb(1.0, 1.0, 1.0);

            let (mut visible, layer, pt, shape) =
                entity_shape_query.get_mut(handle.entity()).unwrap();

            match shape.as_typed_shape() {
                TypedShape::Cuboid(c) => {
                    let b = c.local_aabb();
                    // info!(
                    //     "layer: {:?} extents: {:?} center: {:?} verticies: {:?} he: {:?} we: {:?} pt: {:?}",
                    //     // "mins: {:?}, maxs: {:?}, min,cursor,max x: {}, {}, {} y: {}, {}, {}",
                    //     layer,
                    //     // mins, maxs, mins[0], point[0], maxs[0], mins[1], point[1], maxs[1],
                    //     b.extents(),
                    //     b.center(),
                    //     b.vertices(),
                    //     c.half_extents,
                    //     b.split_at_center(),
                    //     pt,
                    // )
                }
                _ => (),
            };

            // visible.is_transparent = false;

            // Return `false` instead if we want to stop searching for other colliders containing this point.
            true
        });
    }
}

#[derive(Debug, Default)]
pub struct CursorCollider;

#[derive(Default, Bundle)]
struct CursorColliderBundle {
    pub cursor: CursorCollider,
    // #[bundle]
    // pub collider: ColliderBundle,
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

#[derive(Inspectable, Debug, Default, Clone, Copy)]
pub struct Layer;

#[derive(Inspectable, Debug, Default, Bundle, Clone, Copy)]
pub struct LayerBundle {
    pub layer: Layer,
    pub num: LayerNum,
    pub color: LayerColor,
}

#[derive(Inspectable, Debug, Default, Clone, Copy)]
pub struct LayerColor(pub Color);

#[derive(Inspectable, Debug, Clone)]
pub struct InLayer(pub u16);

impl Default for InLayer {
    fn default() -> Self {
        InLayer(0)
    }
}

#[derive(
    Inspectable, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deref, DerefMut,
)]
pub struct LayerNum(pub u16);
