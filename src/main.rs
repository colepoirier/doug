pub mod draw;
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

#[derive(Component, Debug)]
pub struct LayerColors {
    colors: std::iter::Cycle<std::vec::IntoIter<Color>>,
}

impl Default for LayerColors {
    fn default() -> Self {
        Self {
            colors: vec!["648FFF", "785EF0", "DC267F", "FE6100", "FFB000"]
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

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct ViewPortDimensions {
    pub x_min: i64,
    pub x_max: i64,
    pub y_min: i64,
    pub y_max: i64,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LoadProtoEvent;
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LoadCompleteEvent;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Layer;

#[derive(Component, Debug, Default, Bundle, Clone, Copy)]
pub struct LayerBundle {
    pub layer: Layer,
    pub num: LayerNum,
    pub color: LayerColor,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LayerColor(pub Color);

#[derive(Component, Debug, Clone)]
pub struct InLayer(pub u16);

impl Default for InLayer {
    fn default() -> Self {
        InLayer(0)
    }
}

#[derive(
    Component, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deref, DerefMut,
)]
pub struct LayerNum(pub u16);

#[derive(Component, Debug, Default)]
pub struct CursorColliderDebug;

#[derive(Component, Default, Bundle)]
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

fn main() {
    App::new()
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
        .insert_resource(LayerColors::default())
        .init_resource::<EventTriggerState>()
        .insert_resource(ViewPortDimensions::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_system(event_trigger_system)
        .add_startup_system(setup_system)
        .add_system(load_proto_event_listener_system)
        .add_system(cursor_instersect_system)
        .add_system(cursor_collider_debug_sync_system)
        .add_system(camera_changed_system)
        .run();
}

fn setup_system(mut commands: Commands, windows: Res<Windows>) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.scaling_mode = ScalingMode::WindowSize;

    let window = windows.get_primary().unwrap();
    let width = window.width();
    let height = window.height();

    camera.transform.translation.x = width + 5000.0;
    camera.transform.translation.y = height;
    camera.transform.scale.x = 8.0;
    camera.transform.scale.y = 8.0;

    info!("Camera {:?}", camera.transform);
    commands.spawn_bundle(camera);

    let rect = shapes::Circle {
        radius: 20.0,
        center: [0.0, 0.0].into(),
    };

    let shape_lyon = GeometryBuilder::build_as(
        &rect,
        DrawMode::Outlined {
            fill_mode: FillMode {
                color: Color::hex("39FF14").unwrap(),
                options: FillOptions::default(),
            },
            outline_mode: StrokeMode {
                options: StrokeOptions::default().with_line_width(5.0),
                color: Color::hex("FFFFFF").unwrap(),
            },
        },
        Transform::from_translation(Vec3::new(width / 1.0, height / 1.0, 998.0)),
    );

    info!("Initial cursor pos: {:?}", shape_lyon.transform);

    let cursor_collider = CursorColliderBundle {
        shape_lyon,
        ..Default::default()
    };
    commands.spawn_bundle(cursor_collider);
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

pub fn cursor_collider_debug_sync_system(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_q: Query<&mut Transform, With<CursorColliderDebug>>,
    windows: Res<Windows>,
    camera_q: Query<&Transform, (With<Camera>, Without<CursorColliderDebug>)>,
) {
    let mut shape_pos = cursor_q.single_mut();
    let scale = camera_q.single().scale.x;

    let window = windows.get_primary().unwrap();

    let width = window.width();
    let height = window.height();

    if let Some(cursor_pos) = cursor_moved_events.iter().last() {
        let x = cursor_pos.position.x;
        let y = cursor_pos.position.y;

        let off_x = width + 15.0 * scale;
        let off_y = height - 1.0 * scale;

        let new_x = x * scale - off_x;
        let new_y = y * scale - off_y;

        shape_pos.translation.x = new_x;
        shape_pos.translation.y = new_y;
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
    entity_shape_query: Query<(&InLayer, &import::Rect)>,
    windows: Res<Windows>,
    camera_q: Query<(&GlobalTransform, &Camera), Without<CursorColliderDebug>>,
) {
    let (cam_t, cam) = camera_q.single();

    let window = windows.get(cam.window).unwrap();
    let window_size = Vec2::new(window.width(), window.height());

    // Convert screen position [0..resolution] to ndc [-1..1]
    let ndc_to_world = cam_t.compute_matrix() * cam.projection_matrix.inverse();

    let screen_pos = cursor_collider_q.single().translation.truncate();

    let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
    world_pos.truncate();

    let collider_t = cursor_collider_q.single();
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

fn load_proto_event_listener_system(
    mut events: EventReader<LoadProtoEvent>,
    mut commands: Commands,
    mut layer_colors: ResMut<LayerColors>,
    mut load_complete_event_writer: EventWriter<LoadCompleteEvent>,
    mut query: Query<&mut Transform, With<OrthographicProjection>>,
) {
    for _ in events.iter() {
        let t = std::time::Instant::now();
        import::load_proto_lib(
            &mut commands,
            &mut layer_colors,
            &mut load_complete_event_writer,
            &mut query,
        );
        let d = t.elapsed();
        info!("{:?}", d);
    }
}
