use crate::{
    import::{
        ImportLibCompleteEvent, Layer, Layers, LoadCellEvent, OpenVlsirLibEvent, VlsirCell,
        VlsirLib,
    },
    InLayer,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use rfd::FileDialog;

pub struct UIPlugin;

/// Token to ensure a system runs on the main thread.
#[derive(Default)]
pub struct NonSendMarker;

#[derive(Debug, Default, Copy, Clone)]
pub struct LibInfoUIDropdownState {
    pub selected: usize,
}

#[derive(Debug, Default, Clone)]
pub struct LayersUIState {
    pub layers: Vec<(bool, u8, String)>,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct LibInfoUILoadingState {
    pub loading: bool,
}

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .insert_resource(LibInfoUIDropdownState::default())
            .insert_resource(LibInfoUILoadingState::default())
            .insert_resource(LayersUIState::default())
            .init_resource::<NonSendMarker>()
            .add_system(file_menu_system)
            // .add_system(debug_cursor_ui_or_world_system)
            .add_system(lib_info_cell_picker_system)
            .add_system(load_dropdown_selected_cell_system)
            .add_system(layer_visibility_widget_system)
            .add_system(set_layer_visibility_system)
            .add_system(layer_zindex_stepthru_system);
    }
}

pub fn file_menu_system(
    // need this to make the system run on the main thread otherwise MacOS
    // will have a race condition with the file dialog open request where
    // sometimes the file dialog will not open and the app will go into
    // 'Not Responding'/spinning beachball state
    _marker: NonSend<NonSendMarker>,
    mut egui_ctx: ResMut<EguiContext>,
    mut open_vlsir_lib_event_writer: EventWriter<OpenVlsirLibEvent>,
) {
    egui::TopBottomPanel::top("top_panel").show(egui_ctx.ctx_mut(), |ui| {
        // The top panel is often a good place for a menu bar:
        egui::menu::bar(ui, |ui| {
            egui::menu::menu_button(ui, egui::RichText::new("File").size(17.0), |ui| {
                ui.spacing_mut().button_padding = (8.0, 8.0).into();
                if ui.button(egui::RichText::new("Load").size(16.0)).clicked() {
                    ui.close_menu();
                    let path = FileDialog::new()
                        .add_filter("protos", &["proto"])
                        .pick_file();
                    // handle file picking cancellation by only sending event if a file was selected
                    if let Some(path) = path {
                        open_vlsir_lib_event_writer.send(OpenVlsirLibEvent {
                            path: path.to_str().unwrap().to_owned(),
                        });
                    }
                }
                if ui.button(egui::RichText::new("Quit").size(16.0)).clicked() {
                    std::process::exit(0);
                }
            });
        });
    });
}

pub fn lib_info_cell_picker_system(
    mut egui_ctx: ResMut<EguiContext>,
    vlsir_lib: Res<VlsirLib>,
    vlsir_cell: Res<VlsirCell>,
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
                ui.horizontal(|ui| {
                    ui.label(format!("Current Library:"));
                    ui.add_space(4.0);
                    ui.label(format!(
                        "Loading {}...",
                        vlsir_lib
                            .path
                            .as_ref()
                            .unwrap()
                            .split("/")
                            .last()
                            .unwrap()
                            .strip_suffix(".proto")
                            .unwrap()
                    ));
                    ui.add(egui::Spinner::new());
                });
            } else if vlsir_lib.path.is_some() && vlsir_lib.lib.is_some() {
                ui.label(format!(
                    "Current Library: {}",
                    vlsir_lib.lib.as_ref().unwrap().name
                ));
            }

            ui.add_space(5.0);

            if let Some(names) = &vlsir_lib.cell_names {
                let len = names.len();
                ui.horizontal(|ui| {
                    ui.spacing_mut().slider_width = 300.0;
                    ui.add(egui::Label::new("Cells:"));
                    // ui.add_space(1.0);
                    ui.add_enabled_ui(true, |ui| {
                        egui::ComboBox::from_label("")
                            .show_index(ui, &mut temp, len, |i| names[i].to_owned())
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

            if let Some(num_shapes) = vlsir_cell.num_shapes.as_ref() {
                ui.add_space(5.0);
                ui.label(format!("No. shapes: {num_shapes}"));
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

pub fn layer_visibility_widget_system(
    mut egui_ctx: ResMut<EguiContext>,
    layers: Res<Layers>,
    mut state: ResMut<LayersUIState>,
) {
    let mut temp = state.layers.clone();

    if temp.is_empty() {
        let mut layers = layers
            .iter()
            .map(|(num, Layer { name, color })| (*num, name.clone(), *color))
            .collect::<Vec<(u8, Option<String>, Color)>>();

        layers.sort_by(|a, b| a.0.cmp(&b.0));

        for layer in layers {
            let label = if let Some(name) = layer.1 {
                format!("({}) {}", layer.0, name)
            } else {
                format!("{}", layer.0)
            };
            temp.push((true, layer.0, label));
        }
    }

    egui::Window::new("Layers")
        .resizable(true)
        .default_pos([5.0, 532.0])
        .show(egui_ctx.ctx_mut(), |ui| {
            for layer in temp.iter_mut() {
                ui.vertical(|ui| {
                    ui.add(egui::Checkbox::new(&mut layer.0, &layer.2));
                });
            }
        });

    if state.layers != temp {
        state.layers = temp;
    }
}

pub fn set_layer_visibility_system(
    layer_state: Res<LayersUIState>,
    mut prev: Local<Vec<(bool, u8, String)>>,
    mut shape_q: Query<(&InLayer, &mut Visibility)>,
) {
    if prev.len() == 0 {
        *prev = layer_state.layers.clone();
    }

    for ((curr_vis, layer, _), (prev_vis, _, _)) in layer_state.layers.iter().zip(prev.iter()) {
        if curr_vis != prev_vis {
            for (in_layer, mut vis) in shape_q.iter_mut() {
                if **in_layer == *layer {
                    vis.is_visible = *curr_vis;
                }
            }
        }
    }

    if *prev != layer_state.layers {
        *prev = layer_state.layers.clone();
    }
}

pub fn layer_zindex_stepthru_system(
    mut layer_state: ResMut<LayersUIState>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::LShift) {
        if let Some(elem) = layer_state.layers.iter_mut().rev().find(|(vis, _, _)| *vis) {
            elem.0 = false;
        }
    } else if keyboard.just_pressed(KeyCode::LControl) {
        if let Some(elem) = layer_state.layers.iter_mut().find(|(vis, _, _)| !vis) {
            elem.0 = true;
        }
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
