pub mod editing;
pub mod import;
pub mod shapes;
pub mod ui;

use bevy::ecs::archetype::Archetypes;
use bevy::ecs::component::{ComponentId, Components};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::render::camera::Camera;
use bevy::{prelude::*, render::camera::ScalingMode, window::PresentMode, winit::WinitSettings};

use bevy_egui::EguiContext;

use layout21::raw;

// use bevy_framepace::{FramepacePlugin, FramerateLimit};
// use bevy_inspector_egui::WorldInspectorPlugin;

use editing::EditingPlugin;
use import::Layout21ImportPlugin;
use ui::UIPlugin;

// Set a default alpha-value for most shapes
pub const ALPHA: f32 = 0.1;
pub const WIDTH: f32 = 10.0;

pub const DEFAULT_SCALE: f32 = 10e-2;
pub const DEFAULT_UNITS: f32 = 10e-9;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViewportDimensions {
    pub x_min: i64,
    pub x_max: i64,
    pub y_min: i64,
    pub y_max: i64,
    pub center: raw::Point,
}

#[derive(Debug, Default, Clone, Copy, Deref, DerefMut)]
pub struct CursorWorldPos(pub Vec2);

#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct InLayer(pub u8);

impl Default for InLayer {
    fn default() -> Self {
        InLayer(0)
    }
}

#[derive(
    Component, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deref, DerefMut,
)]
pub struct LayerNum(pub u8);

#[derive(Debug, Default, Clone, Copy)]
pub struct UpdateViewportEvent {
    pub viewport: ViewportDimensions,
}

fn main() {
    App::new()
        .add_event::<UpdateViewportEvent>()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(WindowDescriptor {
            title: "Doug CAD".to_string(),
            width: 1920.0,
            height: 1080.0,
            ..Default::default()
        })
        .insert_resource(WinitSettings::desktop_app())
        // .insert_resource(WindowDescriptor {
        //     present_mode: PresentMode::Mailbox,
        //     ..Default::default()
        // })
        .insert_resource(ViewportDimensions::default())
        .insert_resource(CursorWorldPos::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(Layout21ImportPlugin)
        .add_plugin(EditingPlugin)
        .add_plugin(UIPlugin)
        // .add_plugin(FramepacePlugin::default())
        // .add_plugin(WorldInspectorPlugin::default())
        .add_stage("camera_change", SystemStage::parallel())
        .add_stage_after(
            "camera_change",
            "detect_camera_change",
            SystemStage::parallel(),
        )
        .add_startup_system(setup_system)
        .add_system_to_stage("camera_change", update_camera_viewport_system)
        .add_system_to_stage("camera_change", camera_zoom_system)
        .add_system_to_stage("camera_change", camera_pan_system)
        .add_system_to_stage("detect_camera_change", camera_changed_system)
        .add_system(cursor_world_pos_system)
        .run();
}

fn setup_system(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.scaling_mode = ScalingMode::WindowSize;
    commands.spawn_bundle(camera);
}

pub fn update_camera_viewport_system(
    windows: Res<Windows>,
    mut update_viewport_event_reader: EventReader<UpdateViewportEvent>,
    mut viewport_dimensions: ResMut<ViewportDimensions>,
    mut camera_q: Query<(&mut Transform, &mut OrthographicProjection, &Camera)>,
) {
    for UpdateViewportEvent { viewport } in update_viewport_event_reader.iter() {
        let (mut cam_t, mut proj, cam) = camera_q.single_mut();

        *viewport_dimensions = *viewport;

        let ViewportDimensions {
            x_min,
            x_max,
            y_min,
            y_max,
            center,
        } = *viewport;

        info!("[x] min: {x_min}, max: {x_max} [y] min: {y_min}, max: {y_max}",);

        let width = (x_max - x_min) as f32;
        let height = (y_max - y_min) as f32;

        info!("width: {width}, height: {height}");

        let padding = 100.0;

        let window = windows.primary();

        let screen_width = window.width() - (2.0 * padding);
        let screen_height = window.height() - (2.0 * padding);

        let width_ratio = width / screen_width;
        let height_ratio = height / screen_height;

        info!("width/viewport_width: {width_ratio}, height/viewport_height: {height_ratio}");

        let scale = width_ratio.max(height_ratio);

        let world_width = screen_width * scale;
        let world_height = screen_height * scale;

        info!("world_width: {world_width}, world_height: {world_height}");

        proj.scale = scale;

        cam_t.translation.x = center.x as f32;
        cam_t.translation.y = center.y as f32;
    }
}

fn camera_zoom_system(
    mut query: Query<&mut OrthographicProjection>,
    mut scroll_events: EventReader<MouseWheel>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    if !egui_ctx.ctx_mut().wants_pointer_input() {
        // code taken from the excellent bevy_pancam plugin
        // https://web.archive.org/web/20220402030829/https://github.com/johanhelsing/bevy_pancam
        let pixels_per_line = 100.0; // Maybe make configurable?
        let scroll = scroll_events
            .iter()
            .map(|ev| match ev.unit {
                MouseScrollUnit::Pixel => ev.y,
                MouseScrollUnit::Line => ev.y * pixels_per_line,
            })
            .sum::<f32>();

        if scroll == 0.0 {
            return;
        }

        for mut projection in query.iter_mut() {
            projection.scale = (projection.scale * (1. + -scroll * 0.001)).max(0.00001);
        }
    }
}

pub fn camera_pan_system(
    input_mouse: Res<Input<MouseButton>>,
    mut egui_ctx: ResMut<EguiContext>,
    mut cam_q: Query<(&mut Transform, &OrthographicProjection)>,
    windows: Res<Windows>,
    mut last_pos: Local<Option<Vec2>>,
) {
    // change input mapping for panning here.
    let pan_button = MouseButton::Right;

    if !egui_ctx.ctx_mut().wants_pointer_input() {
        // code taken from the excellent bevy_pancam plugin
        // https://web.archive.org/web/20220402030829/https://github.com/johanhelsing/bevy_pancam
        let window = windows.get_primary().unwrap();

        // Use position instead of MouseMotion, otherwise we don't get acceleration movement
        let current_pos = match window.cursor_position() {
            Some(current_pos) => current_pos,
            None => return,
        };
        let delta = current_pos - last_pos.unwrap_or(current_pos);

        for (mut transform, projection) in cam_q.iter_mut() {
            if input_mouse.pressed(pan_button) {
                let scaling = Vec2::new(
                    window.width() / (projection.right - projection.left),
                    window.height() / (projection.top - projection.bottom),
                ) * projection.scale;

                transform.translation -= (delta * scaling).extend(0.);
            }
        }
        *last_pos = Some(current_pos);
    }
}

fn camera_changed_system(camera_q: Query<&Transform, (Changed<Transform>, With<Camera>)>) {
    for c in camera_q.iter() {
        info!("Camera new transform {:?}", c);
    }
}

pub fn cursor_world_pos_system(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_world_pos: ResMut<CursorWorldPos>,
    windows: Res<Windows>,
    camera_q: Query<(&Transform, &Camera)>,
) {
    let (cam_t, cam) = camera_q.single();

    let window = windows.primary();
    let window_size = Vec2::new(window.width(), window.height());

    // Convert screen position [0..resolution] to ndc [-1..1]
    let ndc_to_world = cam_t.compute_matrix() * cam.projection_matrix.inverse();

    if let Some(&CursorMoved { position, .. }) = cursor_moved_events.iter().last() {
        let ndc = (Vec2::new(position.x, position.y) / window_size) * 2.0 - Vec2::ONE;
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
        world_pos.truncate();

        cursor_world_pos.x = world_pos.x;
        cursor_world_pos.y = world_pos.y;
    }
}

pub fn screen_to_world_pos(
    windows: &Windows,
    camera_q: &Query<(&Transform, &Camera)>,
    screen_pos: Vec2,
) -> Vec2 {
    let (cam_t, cam) = camera_q.single();

    let window = windows.primary();
    let window_size = Vec2::new(window.width(), window.height());

    // Convert screen position [0..resolution] to ndc [-1..1]
    let ndc_to_world = cam_t.compute_matrix() * cam.projection_matrix.inverse();
    let ndc = (Vec2::new(screen_pos.x, screen_pos.y) / window_size) * 2.0 - Vec2::ONE;
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
    world_pos.truncate()
}

pub fn get_component_names_for_entity(
    entity: Entity,
    archetypes: &Archetypes,
    components: &Components,
) -> Vec<String> {
    let mut comp_names = vec![];
    for archetype in archetypes.iter() {
        if archetype.entities().contains(&entity) {
            comp_names = archetype.components().collect::<Vec<ComponentId>>();
        }
    }
    comp_names
        .iter()
        .map(|c| components.get_info(*c).unwrap().name().to_string())
        .collect::<Vec<String>>()
}
