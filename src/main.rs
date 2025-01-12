use bevy::log::{Level, LogPlugin};
use bevy::{prelude::*, window::PresentMode, winit::WinitSettings};

pub mod import;
pub mod shapes;

use bevy_editor_cam::prelude::EditorCam;
use bevy_editor_cam::DefaultEditorCamPlugins;
use import::{library::LoadVlsirLib, Layout21ImportPlugin, RectMesh};

// Set a default alpha-value for most shapes
pub const ALPHA: f32 = 0.1;
pub const WIDTH: f32 = 1.0;

pub const DEFAULT_SCALE: f32 = 10e-2;
pub const DEFAULT_UNITS: f32 = 10e-9;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        // .insert_resource(WinitSettings::desktop_app())
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Doug CAD".to_string(),
                        resolution: (1920.0, 1080.0).into(),
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: "doug=trace".into(),
                    level: Level::WARN,
                    ..default()
                })
                .build(),
        )
        .add_plugins((MeshPickingPlugin, DefaultEditorCamPlugins))
        .add_plugins(Layout21ImportPlugin)
        .add_systems(Startup, setup_system)
        .run();
}

fn setup_system(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn((
        Camera2d::default(),
        Projection::from(OrthographicProjection::default_2d()),
        EditorCam::default(),
    ));
    let resource = LoadVlsirLib {
        // path: "/Users/colepoirier/Dropbox/rust_2020_onwards/doug/doug/libs/dff1_lib.proto".into(),
        path: "/Users/colepoirier/Dropbox/rust_2020_onwards/doug/doug/libs/oscibear.proto".into(),
    };
    commands.insert_resource(resource);
    println!("inserted resoure!");
    commands.insert_resource(RectMesh(meshes.add(Rectangle::default())));
}
