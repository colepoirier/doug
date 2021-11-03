pub mod import;

use bevy::reflect::erased_serde::deserialize;
use bevy::render::camera::OrthographicProjection;
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_config_cam::{CameraState, ConfigCam, NoCameraPlayerPlugin, PlayerPlugin};
use bevy_inspector_egui::RegisterInspectable;
use bevy_inspector_egui::{widgets::ResourceInspector, Inspectable, InspectorPlugin};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_prototype_lyon::prelude::ShapePlugin;
use import::Path;

use derive_more::{Deref, DerefMut};

// use bevy_config_cam::ConfigCam;

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
            .map(|c| *Color::hex(c).unwrap().set_a(ALPHA))
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Inspectable)]
pub enum Shape {
    Rect,
    Poly,
}

#[derive(Debug, Default, Inspectable)]
pub struct Paths {
    pub paths: Vec<Path>,
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

#[derive(Debug, Default, Clone, Copy)]
pub struct DrawShapeEvent {
    pub layer: LayerNum,
    pub shape: Shape,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LoadProtoEvent;
#[derive(Debug, Default, Clone, Copy)]
pub struct LoadCompleteEvent;

fn main() {
    App::new()
        .add_event::<LoadProtoEvent>()
        .add_event::<LoadCompleteEvent>()
        .insert_resource(Msaa { samples: 8 })
        .insert_resource(LayerColors::default())
        .insert_resource(Paths::default())
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
        .insert_resource(WorldInspectorParams {
            despawnable_entities: true,
            ..Default::default()
        })
        .add_plugin(WorldInspectorPlugin::new())
        // .add_plugin(InspectorPlugin::<Resources>::new())
        .add_plugin(ShapePlugin)
        // .add_plugin(ConfigCam)
        .register_inspectable::<LayerColor>()
        .register_inspectable::<InLayer>()
        .register_inspectable::<Path>()
        .register_inspectable::<LayerNum>()
        .init_resource::<EventTriggerState>()
        .add_plugin(NoCameraPlayerPlugin)
        .add_system(event_trigger_system.system())
        .add_startup_system(setup.system())
        .add_system(load_proto_event_listener_system.system())
        .run();
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

// prints events as they come in
fn draw_shape_event_listener_system(
    mut events: EventReader<LoadCompleteEvent>,
    mut commands: Commands,
    // color_query: Query<(&LayerNum, &Color), With<Layer>>,
) {
    for load_complete_event in events.iter() {
        // test_spawn_path(&mut commands, &color_query);
        // info!(
        //     "Added {:?} to {:?}",
        //     draw_shape_event.shape, draw_shape_event.layer
        // );
    }
}

// prints events as they come in
fn load_proto_event_listener_system(
    mut events: EventReader<LoadProtoEvent>,
    mut commands: Commands,
    mut layer_colors: ResMut<LayerColors>,
    mut paths: ResMut<Paths>,
    mut load_complete_event_writer: EventWriter<LoadCompleteEvent>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection)>,
) {
    for _ in events.iter() {
        let t = std::time::Instant::now();
        import::test_load_proto_lib(
            &mut commands,
            &mut layer_colors,
            &mut paths,
            &mut load_complete_event_writer,
            &mut query,
        );
        let d = t.elapsed();
        println!("{:?}", d);
    }
}

fn setup(mut commands: Commands) {
    // let mut transform = Transform::from_xyz(0.0, 0.0, 1_000.0).looking_at(Vec3::default(), Vec3::Y);
    // transform.apply_non_uniform_scale(Vec3::new(8.0, 8.0, 1_000.0));

    let mut camera = OrthographicCameraBundle::new_2d();

    camera.orthographic_projection.scale = 1_000_000.0;
    camera.orthographic_projection.scaling_mode = ScalingMode::FixedVertical;

    camera.transform = Transform::from_xyz(0.0, 0.0, 1_000.0);
    camera.transform.translation.x = -40000.0;
    camera.transform.translation.y = 300000.0;

    println!("{:?}", camera.transform);
    commands.spawn_bundle(camera);
}

// Set a default alpha-value for most shapes
pub const ALPHA: f32 = 0.25;
pub const WIDTH: f32 = 1000.0;

#[derive(Debug, Component, Default, Clone, Copy, Inspectable)]
pub struct Layer;

#[derive(Debug, Component, Default, Bundle, Clone, Copy, Inspectable)]
pub struct LayerBundle {
    pub layer: Layer,
    pub num: LayerNum,
    pub color: LayerColor,
}

#[derive(Debug, Component, Default, Clone, Copy, Inspectable)]
pub struct LayerColor(pub Color);

#[derive(Debug, Component, Clone, Inspectable)]
pub struct InLayer(pub Entity);

impl Default for InLayer {
    fn default() -> Self {
        InLayer(Entity::new(0))
    }
}

#[derive(
    Debug,
    Component,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Deref,
    DerefMut,
    Inspectable,
)]
pub struct LayerNum(pub u16);
