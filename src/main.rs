mod geom;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use std::collections::HashMap;

use geom::{Layer, LayerMap, Path};

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 8 })
        .insert_resource(LayerMap(HashMap::<Name, Entity>::new()))
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_startup_system(setup.system().chain(test_spawn_path.system()))
        .run();
}

fn setup(
    mut commands: Commands,
    // color_query: Query<&Color, With<Layer>>
) {
    let layer = commands.spawn().insert(Layer).insert(Color::CRIMSON).id();
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn test_spawn_path(
    commands: &mut Commands,
    color_query: &Query<&Color, With<Layer>>,
    layers: Res<LayerMap>,
    layer: Name,
) {
    Path::spawn(
        commands,
        color_query,
        layers,
        layer,
        10.0,
        &vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(150.0, 300.0),
            Vec2::new(300.0, 0.0),
        ],
    )
}
