use crate::import::LoadProtoEvent;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use rfd::FileDialog;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin).add_system(ui_system);
    }
}

pub fn ui_system(
    mut egui_ctx: ResMut<EguiContext>,
    mut load_proto_event_writer: EventWriter<LoadProtoEvent>,
) {
    egui::TopBottomPanel::top("top_panel").show(egui_ctx.ctx_mut(), |ui| {
        // The top panel is often a good place for a menu bar:
        egui::menu::bar(ui, |ui| {
            egui::menu::menu_button(ui, "File", |ui| {
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                } else if ui.button("Load").clicked() {
                    let proto = FileDialog::new()
                        .add_filter("protos", &["proto"])
                        .pick_file()
                        .unwrap();
                    load_proto_event_writer.send(LoadProtoEvent {
                        lib: String::from(proto.to_str().unwrap()),
                    });
                }
            });
        });
    });
}
