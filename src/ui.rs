use crate::import::{LoadCellCompleteEvent, LoadLibEvent, ProtoGdsLib};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use layout21::protos::LayerShapes;
use rfd::FileDialog;

pub struct UIPlugin;

#[derive(Debug, Default, Copy, Clone)]
pub struct LibInfoUIState {
    selected: usize,
}

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .insert_resource(LibInfoUIState::default())
            .add_system(file_menu_system)
            // .add_system(debug_cursor_ui_or_world_system)
            .add_system(lib_info_cell_picker_system);
    }
}

pub fn file_menu_system(
    mut egui_ctx: ResMut<EguiContext>,
    mut load_proto_event_writer: EventWriter<LoadLibEvent>,
) {
    egui::TopBottomPanel::top("top_panel").show(egui_ctx.ctx_mut(), |ui| {
        // The top panel is often a good place for a menu bar:
        egui::menu::bar(ui, |ui| {
            egui::menu::menu_button(ui, "File", |ui| {
                ui.style_mut().body_text_style = egui::TextStyle::Heading;
                if ui.button("Load").clicked() {
                    ui.close_menu();
                    let proto = FileDialog::new()
                        .add_filter("protos", &["proto"])
                        .pick_file();
                    // handle file picking cancellation by only sending event if a file was selected
                    if let Some(path) = proto {
                        load_proto_event_writer.send(LoadLibEvent {
                            lib: String::from(path.to_str().unwrap()),
                        });
                    }
                }
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });
        });
    });
}

pub fn lib_info_cell_picker_system(
    mut egui_ctx: ResMut<EguiContext>,
    proto_gds_lib: Res<ProtoGdsLib>,
    mut state: ResMut<LibInfoUIState>,
) {
    let lib_name = if let Some(lib) = proto_gds_lib.lib.as_ref() {
        lib.domain.clone()
    } else {
        String::new()
    };
    egui::Window::new("Library Info")
        .resizable(true)
        .default_pos([5.0, 32.0])
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.label(format!("Current Library: {lib_name}"));

            if proto_gds_lib.cells.len() > 0 {
                let len = proto_gds_lib.cells.len();
                ui.add_enabled_ui(true, |ui| {
                    egui::ComboBox::from_label("Cells").width(100.0).show_index(
                        ui,
                        &mut state.selected,
                        len,
                        |i| proto_gds_lib.cells[i].to_owned(),
                    )
                });
            } else {
                ui.add_enabled_ui(false, |ui| {
                    egui::ComboBox::from_label("Cells")
                        .width(100.0)
                        .show_ui(ui, |_| {})
                        .response
                });
            }
            if let Some(lib) = proto_gds_lib.lib.as_ref() {
                ui.label(format!(
                    "No. shapes: {}",
                    lib.cells[state.selected]
                        .layout
                        .as_ref()
                        .unwrap()
                        .shapes
                        .iter()
                        .fold(0, |len, layer| {
                            let mut tot = 0;
                            tot += layer.rectangles.len();
                            tot += layer.polygons.len();
                            tot += layer.paths.len();
                            len + tot
                        })
                ));
            }
        });
}

// figure out if cursor is hovering over UI or over bevy 'app world'
pub fn debug_cursor_ui_or_world_system(mut egui_ctx: ResMut<EguiContext>) {
    info!(
        "is_pointer_over_area: {}, want_pointer_input: {}, is_using_pointer: {}",
        // simply detects if the cursor is over/in an egui element.
        // If a click and hold and mouse drag starts in the bevy app world,
        // the camera will not pan at this point, but if the click and hold/drag
        // continues and the cursor ends up over/in an egui element, the camera
        // will start to pan :( So we use `.wants_pointer_input` instead :)
        egui_ctx.ctx_mut().is_pointer_over_area(),
        // **desired behaviour** to ensure camera does not pan when moving egui windows,
        // or other clicking and dragging in/over egui elements,
        // and does not zoom when scrolling egui dropdown menus
        // does not scroll camera if a click and hold and mouse drag starts in bevy app world,
        // and at some point the cursor hovers over an egui element
        egui_ctx.ctx_mut().wants_pointer_input(),
        // active interactions with egui like clicking, holding mouse button,
        // for detecting slider dragging
        egui_ctx.ctx_mut().is_using_pointer(),
    );
}
