use bevy::log::{Level, LogPlugin};
use bevy::render::camera::Camera;
use bevy::{prelude::*, render::camera::ScalingMode, window::PresentMode, winit::WinitSettings};
use bevy_mod_picking::{self, DefaultPickingPlugins};
use bevy_pancam::{PanCam, PanCamPlugin};

pub mod editing;
pub mod import;
pub mod shapes;
pub mod ui;

// use bevy_framepace::{FramepacePlugin, FramerateLimit};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use editing::EditingPlugin;
use import::Layout21ImportPlugin;
use ui::UIPlugin;

// Set a default alpha-value for most shapes
pub const ALPHA: f32 = 0.1;
pub const WIDTH: f32 = 10.0;

pub const DEFAULT_SCALE: f32 = 10e-2;
pub const DEFAULT_UNITS: f32 = 10e-9;

fn main() {
    // use bevy::log::LogPlugin;
    // use bevy_mod_debugdump::schedule_graph::Settings;

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(WinitSettings::desktop_app())
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Doug CAD".to_string(),
                        resolution: (1920.0, 1080.0).into(),
                        present_mode: PresentMode::AutoNoVsync,
                        // Tells wasm to resize the window according to the available canvas
                        fit_canvas_to_parent: true,
                        // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: "bevy_mod_picking=trace".into(),
                    level: Level::INFO,
                })
                .build(), // .disable::<LogPlugin>(),
        )
        .add_plugins((Layout21ImportPlugin, EditingPlugin, UIPlugin))
        .add_plugins(PanCamPlugin::default())
        .add_plugins(DefaultPickingPlugins)
        // .add_plugin(FramepacePlugin::default())
        .add_plugins(WorldInspectorPlugin::default())
        .add_systems(Update, camera_changed_system)
        .add_systems(Startup, setup_system)
        .run();

    // let settings = Settings::default().filter_in_crate("doug");
    // bevy_mod_debugdump::print_main_schedule(&mut app);
    // let dot = bevy_mod_debugdump::schedule_graph_dot(&mut app, Update, &settings);
    // println!("{dot}");
}

fn setup_system(
    mut commands: Commands,
    mut logging_next_state: ResMut<NextState<bevy_mod_picking::debug::DebugPickingMode>>,
) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::WindowSize(1.0);
    commands.spawn((camera, PanCam::default()));
    logging_next_state.set(bevy_mod_picking::debug::DebugPickingMode::Normal);
}

fn camera_changed_system(camera_q: Query<&Transform, (Changed<Transform>, With<Camera>)>) {
    for c in camera_q.iter() {
        info!("Camera new transform {:?}", c);
    }
}

// use bevy::ecs::archetype::Archetypes;

// pub fn get_component_names_for_entity(
//     entity: Entity,
//     archetypes: &Archetypes,
//     components: &Components,
// ) -> Vec<String> {
//     let mut comp_names = vec![];
//     for archetype in archetypes.iter() {
//         if archetype.entities().contains(&entity) {
//             comp_names = archetype.components().collect::<Vec<ComponentId>>();
//         }
//     }
//     comp_names
//         .iter()
//         .map(|c| components.get_info(*c).unwrap().name().to_string())
//         .collect::<Vec<String>>()
// }
