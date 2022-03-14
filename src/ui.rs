use crate::import::{ImportLibCompleteEvent, LoadCellEvent, OpenVlsirLibEvent, VlsirLib};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use rfd::FileDialog;

pub struct UIPlugin;

#[derive(Debug, Default, Copy, Clone)]
pub struct LibInfoUIDropdownState {
    selected: usize,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct LibInfoUILoadingState {
    loading: bool,
}

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .insert_resource(LibInfoUIDropdownState::default())
            .insert_resource(LibInfoUILoadingState::default())
            .add_system(file_menu_system)
            // .add_system(debug_cursor_ui_or_world_system)
            .add_system(lib_info_cell_picker_system)
            .add_system(load_dropdown_selected_cell_system);
    }
}

pub fn file_menu_system(
    mut egui_ctx: ResMut<EguiContext>,
    mut vlsir_lib: ResMut<VlsirLib>,
    mut open_vlsir_lib_event_writer: EventWriter<OpenVlsirLibEvent>,
) {
    egui::TopBottomPanel::top("top_panel").show(egui_ctx.ctx_mut(), |ui| {
        // The top panel is often a good place for a menu bar:
        egui::menu::bar(ui, |ui| {
            egui::menu::menu_button(ui, "File", |ui| {
                ui.style_mut().body_text_style = egui::TextStyle::Heading;
                if ui.button("Load").clicked() {
                    ui.close_menu();
                    let path = FileDialog::new()
                        .add_filter("protos", &["proto"])
                        .pick_file();
                    // handle file picking cancellation by only sending event if a file was selected
                    if let Some(path) = path {
                        vlsir_lib.path = Some(path.to_str().unwrap().to_owned());
                        vlsir_lib.name = Some(
                            path.to_str()
                                .unwrap()
                                .split("/")
                                .last()
                                .unwrap()
                                .strip_suffix(".proto")
                                .unwrap()
                                .to_owned(),
                        );
                        open_vlsir_lib_event_writer.send(OpenVlsirLibEvent);
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
    vlsir_lib: Res<VlsirLib>,
    mut open_vlsir_lib_event_reader: EventReader<OpenVlsirLibEvent>,
    mut import_lib_complete_event_reader: EventReader<ImportLibCompleteEvent>,
    mut dropdown_state: ResMut<LibInfoUIDropdownState>,
    mut loading_state: ResMut<LibInfoUILoadingState>,
) {
    let mut temp = dropdown_state.selected;

    for _ in open_vlsir_lib_event_reader.iter() {
        loading_state.loading = true;
    }

    for _ in import_lib_complete_event_reader.iter() {
        loading_state.loading = false;
    }

    egui::Window::new("Library Info")
        .resizable(true)
        .default_pos([5.0, 32.0])
        .show(egui_ctx.ctx_mut(), |ui| {
            if vlsir_lib.path.is_none() {
                ui.label(format!("Current Library:"));
            } else if loading_state.loading {
                ui.label(format!("Current Library:"));
                // ui.add_space(4.0);
                ui.add(
                    egui::ProgressBar::new(0.99)
                        .text(format!("Loading {}...", vlsir_lib.name.as_ref().unwrap()))
                        .animate(true),
                );
            } else if vlsir_lib.path.is_some() && vlsir_lib.lib.is_some() {
                ui.label(format!(
                    "Current Library: {}",
                    vlsir_lib.name.as_ref().unwrap()
                ));
            }

            ui.add_space(5.0);

            if vlsir_lib.cells.len() > 0 {
                let len = vlsir_lib.cells.len();
                ui.horizontal(|ui| {
                    ui.spacing_mut().slider_width = 300.0;
                    ui.add(egui::Label::new("Cells:"));
                    // ui.add_space(1.0);
                    ui.add_enabled_ui(true, |ui| {
                        egui::ComboBox::from_label("")
                            .show_index(ui, &mut temp, len, |i| vlsir_lib.cells[i].to_owned())
                    });
                });
            } else {
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Cells:"));
                    // ui.add_space(1.0);
                    ui.add_enabled_ui(false, |ui| {
                        egui::ComboBox::from_label("")
                            .width(100.0)
                            .show_ui(ui, |_| {})
                            .response
                    });
                });
            }

            if dropdown_state.selected != temp {
                dropdown_state.selected = temp;
            }

            if let Some(lib) = vlsir_lib.lib.as_ref() {
                ui.add_space(5.0);
                ui.label(format!(
                    "No. shapes: {}",
                    lib.cells[dropdown_state.selected]
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

pub fn load_dropdown_selected_cell_system(
    state: Res<LibInfoUIDropdownState>,
    mut load_cell_event_writer: EventWriter<LoadCellEvent>,
) {
    if state.is_changed() {
        info!("{state:?}");
        load_cell_event_writer.send(LoadCellEvent(state.selected));
    }
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
