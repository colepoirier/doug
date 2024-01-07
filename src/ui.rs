use crate::{
    editing::Selected,
    import::{
        ImportLibCompleteEvent, Layer, Layers, LoadCellEvent, Net, OpenVlsirLibEvent, VlsirCell,
        VlsirLib,
    },
    shapes::{InLayer, Path, Poly, Rect},
};
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use futures_lite::future::{self, poll_once};
use geo::prelude::BoundingRect;
use layout21::raw::BoundBoxTrait;
use rfd::AsyncFileDialog;

pub struct UIPlugin;

#[derive(Resource, Debug, Default, Copy, Clone)]
pub struct LibInfoUIDropdownState {
    pub selected: usize,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct LayersUIState {
    pub layers: Vec<(bool, u8, String)>,
}

#[derive(Resource, Debug, Default, Copy, Clone)]
pub struct LibInfoUILoadingState {
    pub loading: bool,
}

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .insert_resource(LibInfoUIDropdownState::default())
            .insert_resource(LibInfoUILoadingState::default())
            .insert_resource(LayersUIState::default())
            .add_systems(
                Update,
                (
                    file_menu_system,
                    // debug_cursor_ui_or_world_system,
                    lib_info_cell_picker_system,
                    load_dropdown_selected_cell_system,
                    layer_visibility_widget_system,
                    set_layer_visibility_system,
                    layer_zindex_stepthru_system,
                    display_cursor_pos_system,
                    display_current_selection_info,
                    handle_file_pick_result_task_system,
                ),
            );
    }
}

#[derive(Debug, Component, Deref, DerefMut)]
pub struct FilePickResultTask(Task<Option<rfd::FileHandle>>);

pub fn file_menu_system(mut commands: Commands, mut egui_ctx: EguiContexts) {
    egui::TopBottomPanel::top("top_panel").show(egui_ctx.ctx_mut(), |ui| {
        // The top panel is often a good place for a menu bar:
        egui::menu::bar(ui, |ui| {
            egui::menu::menu_button(ui, egui::RichText::new("File").size(17.0), |ui| {
                ui.spacing_mut().button_padding = (8.0, 8.0).into();
                if ui.button(egui::RichText::new("Load").size(16.0)).clicked() {
                    ui.close_menu();
                    let task = IoTaskPool::get().spawn(async move {
                        AsyncFileDialog::new()
                            .add_filter("protos", &["proto"])
                            .pick_file()
                            .await
                    });

                    commands.spawn(FilePickResultTask(task));
                }
                if ui.button(egui::RichText::new("Quit").size(16.0)).clicked() {
                    std::process::exit(0);
                }
            });
        });
    });
}

fn handle_file_pick_result_task_system(
    mut commands: Commands,
    mut file_pick_result_task_q: Query<(Entity, &mut FilePickResultTask)>,
    mut open_vlsir_lib_event_writer: EventWriter<OpenVlsirLibEvent>,
) {
    for (entity, mut task) in file_pick_result_task_q.iter_mut() {
        if let Some(file_handle) = future::block_on(poll_once(&mut **task)) {
            // handle file picking cancellation by only sending event if a file was selected
            if let Some(file_handle) = file_handle {
                open_vlsir_lib_event_writer.send(OpenVlsirLibEvent {
                    path: file_handle.path().to_path_buf(),
                });
            }
            commands.entity(entity).despawn();
        }
    }
}

pub fn lib_info_cell_picker_system(
    mut egui_ctx: EguiContexts,
    vlsir_lib: Res<VlsirLib>,
    vlsir_cell: Res<VlsirCell>,
    mut open_vlsir_lib_event_reader: EventReader<OpenVlsirLibEvent>,
    mut import_lib_complete_event_reader: EventReader<ImportLibCompleteEvent>,
    mut dropdown_state: ResMut<LibInfoUIDropdownState>,
    mut loading_state: ResMut<LibInfoUILoadingState>,
) {
    let mut temp = dropdown_state.selected;

    for _ in open_vlsir_lib_event_reader.read() {
        loading_state.loading = true;
    }

    for _ in import_lib_complete_event_reader.read() {
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
                            .to_str()
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
    mut egui_ctx: EguiContexts,
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
            ui.vertical(|ui| {
                for layer in temp.iter_mut() {
                    ui.add(egui::Checkbox::new(&mut layer.0, &layer.2));
                }
            });
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
                    if *curr_vis {
                        *vis = Visibility::Visible;
                    } else {
                        *vis = Visibility::Hidden;
                    }
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
    if keyboard.just_pressed(KeyCode::Key1) {
        if let Some(elem) = layer_state.layers.iter_mut().rev().find(|(vis, _, _)| *vis) {
            elem.0 = false;
        }
    } else if keyboard.just_pressed(KeyCode::Key2) {
        if let Some(elem) = layer_state.layers.iter_mut().find(|(vis, _, _)| !vis) {
            elem.0 = true;
        }
    }
}

pub fn display_cursor_pos_system(mut egui_ctx: EguiContexts) {
    egui::Window::new("Cursor World Position")
        .resizable(false)
        .default_pos([5.0, 142.0])
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.label(format!("x: {} nm, y: {} nm", 0, 0))
        });
}

pub fn display_current_selection_info(
    mut egui_ctx: EguiContexts,
    selected_q: Query<(Entity, &InLayer, &Net), With<Selected>>,
    rect_q: Query<&Rect>,
    poly_q: Query<&Poly>,
    path_q: Query<&Path>,
) {
    // egui::Window::new("Layers")
    //     .resizable(true)
    //     .default_pos([5.0, 532.0])
    //     .show(egui_ctx.ctx_mut(), |ui| {
    //         for layer in temp.iter_mut() {
    //             ui.vertical(|ui| {
    //                 ui.add(egui::Checkbox::new(&mut layer.0, &layer.2));
    //             });
    //         }
    //     });
    egui::Window::new("Currently Selected Shapes")
        .resizable(true)
        .default_pos([5.0, 220.0])
        .show(egui_ctx.ctx_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if selected_q.is_empty() {
                    ui.label(format!("No shape is currently selected"));
                } else {
                    //     let mut layers = layers
                    //     .iter()
                    //     .map(|(num, Layer { name, color })| (*num, name.clone(), *color))
                    //     .collect::<Vec<(u8, Option<String>, Color)>>();

                    // layers.sort_by(|a, b| a.0.cmp(&b.0));
                    let mut selected = selected_q.iter().collect::<Vec<(Entity, &InLayer, &Net)>>();
                    selected.sort_by(|a, b| match a.1.cmp(b.1) {
                        std::cmp::Ordering::Equal => a.0.cmp(&b.0),
                        other => other,
                    });
                    selected
                        .iter()
                        .for_each(|(entity, InLayer(layer), Net(net))| {
                            let mut x_min = 0;
                            let mut y_min = 0;
                            let mut x_max = 0;
                            let mut y_max = 0;

                            let mut shape = "";

                            if let Ok(s) = rect_q.get(*entity) {
                                let min = s.bounding_rect().min();
                                let max = s.bounding_rect().max();
                                x_min = min.x;
                                y_min = min.y;
                                x_max = max.x;
                                y_max = max.y;
                                shape = "Rect";
                            } else if let Ok(s) = poly_q.get(*entity) {
                                let min = s.bounding_rect().unwrap().min();
                                let max = s.bounding_rect().unwrap().max();
                                x_min = min.x;
                                y_min = min.y;
                                x_max = max.x;
                                y_max = max.y;
                                shape = "Poly";
                            } else if let Ok(s) = path_q.get(*entity) {
                                let min = s.points.bbox().p0;
                                let max = s.points.bbox().p1;
                                x_min = min.x as i32;
                                y_min = min.y as i32;
                                x_max = max.x as i32;
                                y_max = max.y as i32;
                                shape = "Path"
                            }

                            if let Some(net) = net {
                                ui.label(format!(
                                    "Layer: {layer:?}, Entity: {entity:?}, Net: {net:?}, {shape}, Bbox: min[{x_min}, {y_min}], max[{x_max}, {y_max}]",
                                ));
                            } else {
                                ui.label(format!("Layer: {layer:?}, Entity: {entity:?}, {shape}, Bbox: min[{x_min}, {y_min}], max[{x_max}, {y_max}]"));
                            }
                        });
                }
            })
        });
}

// figure out if cursor is hovering over UI or over bevy 'app world'
pub fn debug_cursor_ui_or_world_system(mut egui_ctx: EguiContexts) {
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
