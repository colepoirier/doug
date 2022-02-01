pub mod editing;
pub mod import;
pub mod shapes;

use bevy::ecs::archetype::Archetypes;
use bevy::ecs::component::ComponentId;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::render::camera::Camera;
use bevy::{prelude::*, render::camera::ScalingMode};

use derive_more::{Deref, DerefMut};

use bevy_framepace::{FramepacePlugin, FramerateLimit};

use editing::EditingPlugin;
use import::Layout21ImportPlugin;

// Set a default alpha-value for most shapes
pub const ALPHA: f32 = 0.1;
pub const WIDTH: f32 = 10.0;

pub const DEFAULT_SCALE: f32 = 10e-2;
pub const DEFAULT_UNITS: f32 = 10e-9;

#[derive(Component, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nom(String);

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViewportDimensions {
    pub x_min: i64,
    pub x_max: i64,
    pub y_min: i64,
    pub y_max: i64,
}

impl ViewportDimensions {
    pub fn update(&mut self, other: &Self) {
        self.x_min = self.x_min.min(other.x_min);
        self.x_max = self.x_max.max(other.x_max);

        self.y_min = self.y_min.min(other.y_min);
        self.y_max = self.y_max.max(other.y_max);
    }
}

#[derive(Debug, Default, Clone, Copy, Deref, DerefMut)]
pub struct CursorWorldPos(pub IVec2);

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

#[derive(Component, Debug, Clone, Deref, DerefMut)]
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

#[derive(Debug, Default, Component)]
pub struct UpdateViewportEvent;

fn main() {
    App::new()
        .add_event::<UpdateViewportEvent>()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(WindowDescriptor {
            title: "Doug CAD".to_string(),
            width: 1920.0,
            height: 1080.0,
            vsync: true,
            ..Default::default()
        })
        .insert_resource(ViewportDimensions::default())
        .insert_resource(CursorWorldPos::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(Layout21ImportPlugin)
        .add_plugin(EditingPlugin)
        .add_plugin(FramepacePlugin {
            enabled: true,
            framerate_limit: FramerateLimit::Manual(30),
            warn_on_frame_drop: true,
            ..Default::default()
        })
        .add_startup_system(setup_system)
        .add_system(update_camera_viewport_system)
        .add_system(camera_changed_system)
        .add_system(pan_zoom_camera_system)
        .add_system(cursor_world_pos_system)
        .run();
}

fn setup_system(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.scaling_mode = ScalingMode::WindowSize;
    commands.spawn_bundle(camera);
}

pub fn pan_zoom_camera_system(
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    mut q_camera: Query<&mut Transform, With<Camera>>,
) {
    // change input mapping for panning here.
    let pan_button = MouseButton::Left;

    let mut pan = Vec2::ZERO;
    let mut scroll = 0.0;

    if input_mouse.pressed(pan_button) {
        for ev in ev_motion.iter() {
            pan += ev.delta;
        }
    }

    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }

    // assuming there is exacly one main camera entity, so this is ok.
    if let Ok(mut transform) = q_camera.get_single_mut() {
        if pan.length_squared() > 0.0 {
            let scale = transform.scale.x;
            transform.translation.x -= pan.x;
            transform.translation.y += pan.y;
        } else if scroll.abs() > 0.0 {
            let scale = (transform.scale.x - scroll).clamp(1.0, 10.0);
            transform.scale = Vec3::new(scale, scale, scale);
        }
    }
}

fn camera_changed_system(camera_q: Query<&Transform, (Changed<Transform>, With<Camera>)>) {
    for c in camera_q.iter() {
        info!("Camera new transform {:?}", c);
    }
}

pub fn update_camera_viewport_system(
    mut load_complete_event_reader: EventReader<UpdateViewportEvent>,
    viewport: Res<ViewportDimensions>,
    mut camera_q: Query<&mut Transform, With<Camera>>,
) {
    for _ in load_complete_event_reader.iter() {
        let mut camera_transform = camera_q.single_mut();

        let ViewportDimensions {
            x_min,
            x_max,
            y_min,
            y_max,
        } = *viewport;

        info!(
            "[x] min: {}, max: {} [y] min: {}, max: {}",
            x_min, x_max, y_min, y_max
        );

        let x = (x_max - x_min) as f32;
        let y = (y_max - y_min) as f32;

        info!("x {} y {}", x, y);

        let s = x.max(y) as f32 / 1800.0;

        camera_transform.scale.x = s;
        camera_transform.scale.y = s;

        camera_transform.translation.x = (x - 1920.0) / 1.8;
        camera_transform.translation.y = (y - 1080.0) / 1.8;
    }
}

pub fn cursor_world_pos_system(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_world_pos: ResMut<CursorWorldPos>,
    windows: Res<Windows>,
    camera_q: Query<(&Transform, &Camera)>,
) {
    let (cam_t, cam) = camera_q.single();

    let window = windows.get(cam.window).unwrap();
    let window_size = Vec2::new(window.width(), window.height());

    // Convert screen position [0..resolution] to ndc [-1..1]
    let ndc_to_world = cam_t.compute_matrix() * cam.projection_matrix.inverse();

    if let Some(&CursorMoved { position, .. }) = cursor_moved_events.iter().last() {
        let ndc = (Vec2::new(position.x, position.y) / window_size) * 2.0 - Vec2::ONE;
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
        world_pos.truncate();

        cursor_world_pos.x = world_pos.x.round() as i32;
        cursor_world_pos.y = world_pos.y.round() as i32;
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
