pub mod geom;
pub mod import;

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_prototype_lyon::prelude::*;

use geom::{Layer, LayerBundle, LayerNum};

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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

#[derive(Debug, Default, Clone, Copy)]
pub struct DrawShapeEvent {
    pub layer: LayerNum,
    pub shape: Shape,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LoadProtoEvent;

fn main() {
    App::new()
        .add_event::<LoadProtoEvent>()
        .insert_resource(Msaa { samples: 8 })
        .insert_resource(LayerColors::default())
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(WindowDescriptor {
            title: "Doug CAD".to_string(),
            width: 1920.,
            height: 1080.,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .init_resource::<EventTriggerState>()
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
    mut events: EventReader<DrawShapeEvent>,
    mut commands: Commands,
    // color_query: Query<(&LayerNum, &Color), With<Layer>>,
) {
    for draw_shape_event in events.iter() {
        // test_spawn_path(&mut commands, &color_query);
        info!(
            "Added {:?} to {:?}",
            draw_shape_event.shape, draw_shape_event.layer
        );
    }
}

// prints events as they come in
fn load_proto_event_listener_system(
    mut events: EventReader<LoadProtoEvent>,
    mut commands: Commands,
    mut layer_colors: ResMut<LayerColors>,
    // color_query: Query<(&LayerNum, &Color), With<Layer>>,
) {
    for _ in events.iter() {
        let t = std::time::Instant::now();
        import::test_load_proto_lib(&mut commands, &mut layer_colors);
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
    camera.transform.translation.x = 0.0;
    camera.transform.translation.y = 1600.0;
    // .looking_at(Vec3::default(), Vec3::Y);

    // let direction = camera.transform.local_z();
    // camera.transform.translation = direction * 1.0;
    println!("{:?}", camera.transform);
    commands.spawn_bundle(camera);
}

// fn test_spawn_path(commands: &mut Commands, color_query: &Query<(&LayerNum, &Color), With<Layer>>) {
//     Path::spawn(
//         commands,
//         color_query,
//         LayerNum(0),
//         5.0,
//         &vec![
//             Vec2::new(0.0, 0.0),
//             Vec2::new(150.0, 300.0),
//             Vec2::new(300.0, 0.0),
//         ],
//     );

//     Path::spawn(
//         commands,
//         color_query,
//         LayerNum(1),
//         5.0,
//         &vec![
//             Vec2::new(-150.0, 0.0),
//             Vec2::new(0.0, 150.0),
//             Vec2::new(150.0, 0.0),
//         ],
//     );
// }
