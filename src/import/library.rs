use std::{sync::Arc, time::Instant};

use crate::import::cell::LoadCell;
use crate::ALPHA;

use super::{Layer, Layers, VlsirLib};
use bevy::tasks::futures_lite::future;
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use layout21::raw::{proto::proto, proto::ProtoImporter, Layer as VlsirLayer};

pub struct ImportLibraryPlugin;

impl Plugin for ImportLibraryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_vlsir_open_task_sytem.run_if(resource_added::<LoadVlsirLib>),
                handle_vlsir_open_task_system.run_if(resource_exists::<LoadVlsirLib>),
            ),
        );
    }
}

fn layer_colors() -> std::iter::Cycle<std::vec::IntoIter<Srgba>> {
    // IBM Design Language Color Library - Color blind safe palette
    // https://web.archive.org/web/20220304221053/https://ibm-design-language.eu-de.mybluemix.net/design/language/resources/color-library/
    // Color Names: Ultramarine 40, Indigo 50, Magenta 50 , Orange 40, Gold 20
    // It just looks pretty
    ["648FFF", "785EF0", "DC267F", "FE6100", "FFB000"]
        .into_iter()
        .map(|c| Srgba::hex(c).unwrap())
        .collect::<Vec<Srgba>>()
        .into_iter()
        .cycle()
}

#[derive(Resource)]
pub struct LoadVlsirLib {
    pub path: String,
}

#[derive(Component)]
struct LoadVlsirLibTask(Task<LoadVlsirLibResult>);

struct LoadVlsirLibResult {
    start_time: Instant,
    lib: VlsirLib,
    layers: HashMap<u8, (Option<String>, Color)>,
}

fn spawn_vlsir_open_task_sytem(load_vlsir_lib: Res<LoadVlsirLib>, mut commands: Commands) {
    println!("running because resource added");
    let path = load_vlsir_lib.path.clone();

    commands.spawn(LoadVlsirLibTask(AsyncComputeTaskPool::get().spawn(
        async move {
            let start_time = Instant::now();
            let plib: proto::Library = proto::open(path).unwrap();
            let lib = ProtoImporter::import(&plib, None).unwrap();

            let mut layer_colors = layer_colors();
            let mut layers = HashMap::<u8, (Option<String>, Color)>::new();

            {
                let lib_layers = &lib.layers.read().unwrap().slots;
                for VlsirLayer { layernum, name, .. } in lib_layers.values() {
                    let num = *layernum as u8;
                    if let Some(_) = layers.insert(
                        num,
                        (
                            name.clone(),
                            Color::Srgba(layer_colors.next().unwrap()).with_alpha(ALPHA),
                        ),
                        // Layer {
                        //     name: name.clone(),
                        //     color: layer_colors.next().unwrap(),
                        // },
                    ) {
                        panic!(
                            "Library layers corrupted multiple definitions for layer number {}",
                            num
                        );
                    }
                }
            }

            let lib = Arc::new(lib);

            let cell_names = lib
                .cells
                .iter()
                .map(|c| c.read().unwrap().name.clone())
                .collect::<Vec<String>>();

            LoadVlsirLibResult {
                start_time,
                lib: VlsirLib { lib, cell_names },
                layers,
            }
        },
    )));
}

fn handle_vlsir_open_task_system(
    mut commands: Commands,
    mut vlsir_open_task_q: Query<(Entity, &mut LoadVlsirLibTask)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut task) in &mut vlsir_open_task_q {
        if let Some(load_vlsir_lib_result) = future::block_on(future::poll_once(&mut task.0)) {
            let LoadVlsirLibResult {
                start_time,
                lib,
                layers,
            } = load_vlsir_lib_result;
            println!("Loaded lib in {}", start_time.elapsed().as_secs_f32());
            let layers = Layers(Arc::new(
                layers
                    .into_iter()
                    .map(|(num, (name, color))| {
                        (
                            num,
                            Layer {
                                name,
                                color: materials.add(color),
                            },
                        )
                    })
                    .collect::<HashMap<u8, Layer>>(),
            ));
            commands.entity(entity).despawn();
            commands.insert_resource(lib);
            commands.insert_resource(layers);
            commands.remove_resource::<LoadVlsirLib>();
            commands.insert_resource(LoadCell { index: 956 });
            // commands.insert_resource(LoadCell { index: 160 });
            // commands.insert_resource(LoadCell { index: 800 });
            // commands.insert_resource(LoadCell { index: 80 });
            // commands.insert_resource(LoadCell { index: 961 });
        }
    }
}
