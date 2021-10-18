mod geom;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use std::{collections::HashMap, fmt::Debug, sync::Arc};

use geom::{Layer, LayerMap, Path};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Shape {
    Rect,
    Polygon,
}

impl Default for Shape {
    fn default() -> Self {
        Self::Rect
    }
}

#[derive(Debug, Default, Clone)]
pub struct DrawShapeEvent {
    pub layer_name: String,
    pub shape: Shape,
}

fn main() {
    App::build()
        .add_event::<DrawShapeEvent>()
        .insert_resource(Msaa { samples: 8 })
        .insert_resource(LayerMap::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .init_resource::<EventTriggerState>()
        .add_system(event_trigger_system.system())
        .add_system(event_listener_system.system())
        .add_startup_system(setup.system())
        .run();
}

struct EventTriggerState {
    event_timer: Timer,
}

impl Default for EventTriggerState {
    fn default() -> Self {
        EventTriggerState {
            event_timer: Timer::from_seconds(1.0, true),
        }
    }
}

// sends MyEvent every second
fn event_trigger_system(
    time: Res<Time>,
    mut state: ResMut<EventTriggerState>,
    mut my_events: EventWriter<DrawShapeEvent>,
) {
    state.event_timer.tick(time.delta());
    let timer = &mut state.event_timer;
    if timer.finished() && !timer.paused() {
        my_events.send(DrawShapeEvent {
            layer_name: "LAYER_0".into(),
            shape: Shape::Polygon,
        });
        timer.pause()
    }
}

// prints events as they come in
fn event_listener_system(
    mut events: EventReader<DrawShapeEvent>,
    mut commands: Commands,
    color_query: Query<&Color, With<Layer>>,
    layers: Res<LayerMap>,
) {
    for draw_shape_event in events.iter() {
        test_spawn_path(&mut commands, &color_query, &layers);
        info!(
            "Added {:?} to {:?}",
            draw_shape_event.shape, draw_shape_event.layer_name
        );
    }
}

fn setup(mut commands: Commands, mut layers: ResMut<LayerMap>) {
    let layer = commands
        .spawn()
        .insert(Layer)
        .insert(Name::new("LAYER_0"))
        .insert(Color::CRIMSON)
        .id();
    layers.insert(Name::new("LAYER_0"), layer);
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn test_spawn_path(
    commands: &mut Commands,
    color_query: &Query<&Color, With<Layer>>,
    layers: &Res<LayerMap>,
) {
    let layer = layers.get(&Name::new("LAYER_0")).unwrap();

    Path::spawn(
        commands,
        color_query,
        *layer,
        10.0,
        &vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(150.0, 300.0),
            Vec2::new(300.0, 0.0),
        ],
    )
}
