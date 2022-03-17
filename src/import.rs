use crate::shapes::{Path, PathBundle, Poly, PolyBundle, Rect, RectBundle, ShapeBundle};
use crate::{InLayer, UpdateViewportEvent, ViewportDimensions, ALPHA, WIDTH};

use std::collections::HashMap;

use bevy::prelude::*;

use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_prototype_lyon::entity;
use bevy_prototype_lyon::prelude::{
    shapes as lyon_shapes, DrawMode, FillMode, FillOptions, GeometryBuilder, StrokeMode,
    StrokeOptions,
};

use futures_lite::future;

use derive_more::{Deref, DerefMut};

use layout21raw::{
    proto::ProtoImporter, BoundBox, BoundBoxTrait, Cell, Element, Library, Point, Shape,
};
use vlsir;

use std::slice::Iter;

#[derive(Component, Debug)]
pub struct LayerColors {
    colors: std::iter::Cycle<std::vec::IntoIter<Color>>,
}

impl Default for LayerColors {
    fn default() -> Self {
        Self {
            // IBM Design Language Color Library - Color blind safe palette
            // https://web.archive.org/web/20220304221053/https://ibm-design-language.eu-de.mybluemix.net/design/language/resources/color-library/
            // Color Names: Ultramarine 40, Indigo 50, Magenta 50 , Orange 40, Gold 20
            // It just looks pretty
            colors: vec!["648FFF", "785EF0", "DC267F", "FE6100", "FFB000"]
                .into_iter()
                .map(|c| Color::hex(c).unwrap())
                .collect::<Vec<Color>>()
                .into_iter()
                .cycle(),
        }
    }
}

impl LayerColors {
    pub fn get_color(&mut self) -> Color {
        self.colors.next().unwrap()
    }
}

pub fn get_shapes(cell: &Cell) -> Iter<Element> {
    cell.layout.as_ref().unwrap().elems.iter()
}

#[derive(Debug, Default)]
pub struct VlsirLib {
    pub path: Option<String>,
    pub lib: Option<Library>,
    pub cell_names: Option<Vec<String>>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Layer {
    pub name: Option<String>,
    pub color: Color,
}

#[derive(Debug, Default, Clone, Deref, DerefMut)]
pub struct Layers(HashMap<u16, Layer>);

#[derive(Debug, Default)]
pub struct VlsirCell {
    pub index: Option<usize>,
    pub num_shapes: Option<u64>,
}

pub struct Layout21ImportPlugin;

impl Plugin for Layout21ImportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LayerColors::default())
            .insert_resource(Layers::default())
            .insert_resource(VlsirLib::default())
            .insert_resource(VlsirCell::default())
            .add_event::<OpenVlsirLibEvent>()
            .add_event::<OpenVlsirLibCompleteEvent>()
            .add_event::<ImportLibCompleteEvent>()
            .add_event::<LoadCellEvent>()
            .add_event::<LoadCellCompleteEvent>()
            .add_event::<ImportRectEvent>()
            .add_event::<ImportPolyEvent>()
            .add_event::<ImportPathEvent>()
            .add_stage("reset_world", SystemStage::parallel())
            .add_stage_after("reset_world", "async_import", SystemStage::parallel())
            .add_stage_after("reset_world", "import", SystemStage::parallel())
            // .add_startup_system(send_import_event_system)
            .add_system_set_to_stage(
                "reset_world",
                SystemSet::new()
                    .with_system(reset_state_on_new_lib_import)
                    .with_system(reset_state_on_new_cell_import),
            )
            .add_system_set_to_stage(
                "import",
                SystemSet::new()
                    .with_system(spawn_vlsir_open_task_sytem)
                    .with_system(handle_vlsir_open_task_system)
                    .with_system(vlsir_open_task_duration_system)
                    .with_system(import_lib_system)
                    .with_system(load_cell_system)
                    .with_system(load_cell_complete_system)
                    .with_system(import_path_system)
                    .with_system(import_rect_system)
                    .with_system(import_poly_system),
            );
    }
}

#[derive(Component, Debug, Default, Clone, PartialEq, PartialOrd, Deref, DerefMut)]
pub struct Net(Option<String>);

#[derive(Debug, Default, Clone)]
pub struct OpenVlsirLibEvent {
    pub path: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OpenVlsirLibCompleteEvent;

#[derive(Debug, Default, Clone, Copy)]
pub struct ImportLibCompleteEvent;

#[derive(Debug, Default, Clone, Copy, Deref, DerefMut)]
pub struct LoadCellEvent(pub usize);

#[derive(Debug, Default, Clone, Copy)]
pub struct LoadCellCompleteEvent;

pub struct ImportRectEvent {
    pub rect: Rect,
    pub net: Net,
    pub layer: u16,
    pub color: Color,
}

pub struct ImportPolyEvent {
    pub poly: Poly,
    pub net: Net,
    pub layer: u16,
    pub color: Color,
}

pub struct ImportPathEvent {
    pub path: Path,
    pub net: Net,
    pub layer: u16,
    pub color: Color,
}

pub fn load_cell_complete_system(
    mut load_complete_event_reader: EventReader<LoadCellCompleteEvent>,
) {
    for _ in load_complete_event_reader.iter() {}
}

pub fn spawn_vlsir_open_task_sytem(
    mut commands: Commands,
    mut vlsir_lib: ResMut<VlsirLib>,
    mut open_vlsir_lib_event_reader: EventReader<OpenVlsirLibEvent>,
    thread_pool: Res<AsyncComputeTaskPool>,
) {
    for OpenVlsirLibEvent { path } in open_vlsir_lib_event_reader.iter() {
        vlsir_lib.path = Some(path.clone());

        let path = vlsir_lib.path.clone().unwrap();

        let task: Task<Library> = thread_pool.spawn(async move {
            // enable to test UI Lib Info "Library:" loading spinner animation
            // std::thread::sleep(std::time::Duration::from_secs(5));
            let plib: vlsir::Library = vlsir::open(path).unwrap();
            ProtoImporter::import(&plib, None).unwrap()
        });
        commands.spawn().insert(task);
    }
}

pub fn handle_vlsir_open_task_system(
    mut commands: Commands,
    mut lib: ResMut<VlsirLib>,
    mut vlsir_open_task_q: Query<(Entity, &mut Task<Library>)>,
    mut vlsir_open_lib_complete_event_writer: EventWriter<OpenVlsirLibCompleteEvent>,
) {
    for (entity, mut task) in vlsir_open_task_q.iter_mut() {
        if let Some(vlsir_lib) = future::block_on(future::poll_once(&mut *task)) {
            lib.lib = Some(vlsir_lib);
            vlsir_open_lib_complete_event_writer.send(OpenVlsirLibCompleteEvent);
            commands.entity(entity).despawn();
        }
    }
}

pub fn vlsir_open_task_duration_system(
    time: Res<Time>,
    mut duration: Local<f64>,
    mut path: Local<Option<String>>,
    mut open_vlsir_lib_event_reader: EventReader<OpenVlsirLibEvent>,
    mut open_vlsir_lib_complete_event_reader: EventReader<OpenVlsirLibCompleteEvent>,
) {
    for OpenVlsirLibEvent { path: p } in open_vlsir_lib_event_reader.iter() {
        *duration = time.seconds_since_startup();
        *path = Some(p.clone());
    }

    for _ in open_vlsir_lib_complete_event_reader.iter() {
        info!(
            "Vlisr open lib file '{path:?}' task duration {:?}",
            time.seconds_since_startup() - *duration
        );
    }
}

pub fn import_lib_system(
    mut vlsir_lib: ResMut<VlsirLib>,
    mut layer_colors: ResMut<LayerColors>,
    mut layers: ResMut<Layers>,
    mut vlsir_open_lib_complete_event_reader: EventReader<OpenVlsirLibCompleteEvent>,
    mut import_lib_complete_event_writer: EventWriter<ImportLibCompleteEvent>,
    mut load_cell_event_writer: EventWriter<LoadCellEvent>,
) {
    for _ in vlsir_open_lib_complete_event_reader.iter() {
        let lib = vlsir_lib.lib.as_ref().unwrap();
        {
            let lib_layers = &lib.layers.read().unwrap().slots;

            for layout21raw::Layer { layernum, name, .. } in lib_layers.values() {
                let num = *layernum as u16;
                if let Some(_) = layers.insert(
                    num,
                    Layer {
                        name: name.clone(),
                        color: layer_colors.get_color(),
                    },
                ) {
                    panic!(
                        "Library layers corrupted multiple definitions for layer number {}",
                        num
                    );
                }
            }
        }

        let cell_names = lib
            .cells
            .iter()
            .map(|c| c.read().unwrap().name.clone())
            .collect::<Vec<String>>();

        info!("Cell Names: {cell_names:?}");

        let longest_name = cell_names.iter().max().unwrap();

        info!(
            "Longest cell name: {} chars, {}",
            longest_name.chars().count(),
            longest_name
        );

        vlsir_lib.cell_names = Some(cell_names);

        import_lib_complete_event_writer.send(ImportLibCompleteEvent);
        load_cell_event_writer.send(LoadCellEvent(0));
    }
}

pub fn reset_state_on_new_lib_import(
    mut commands: Commands,
    query: Query<Entity, With<entity::Path>>,
    mut layer_colors: ResMut<LayerColors>,
    mut layers: ResMut<Layers>,
    mut vlsir_lib: ResMut<VlsirLib>,
    mut vlsir_open_lib_event_reader: EventReader<OpenVlsirLibEvent>,
) {
    for _ in vlsir_open_lib_event_reader.iter() {
        info!("All state reset on new lib import!");

        *layer_colors = LayerColors::default();
        *layers = Layers::default();
        *vlsir_lib = VlsirLib::default();

        for e in query.iter() {
            commands.entity(e).despawn();
        }
    }
}

pub fn reset_state_on_new_cell_import(
    mut commands: Commands,
    query: Query<Entity, With<entity::Path>>,
    mut load_cell_event_reader: EventReader<LoadCellEvent>,
) {
    for _ in load_cell_event_reader.iter() {
        for e in query.iter() {
            commands.entity(e).despawn();
        }
    }
}

pub fn load_cell_system(
    vlsir_lib: Res<VlsirLib>,
    mut cell_info: ResMut<VlsirCell>,
    layers: Res<Layers>,
    mut update_viewport_event_writer: EventWriter<UpdateViewportEvent>,
    mut load_cell_event_reader: EventReader<LoadCellEvent>,
    mut load_cell_complete_event_writer: EventWriter<LoadCellCompleteEvent>,
    mut import_rect_event_writer: EventWriter<ImportRectEvent>,
    mut import_poly_event_writer: EventWriter<ImportPolyEvent>,
    mut import_path_event_writer: EventWriter<ImportPathEvent>,
) {
    for &cell_idx in load_cell_event_reader.iter() {
        if let Some(lib) = vlsir_lib.lib.as_ref() {
            let t = std::time::Instant::now();

            let lib_layers = lib.layers.read().unwrap();

            let cell = lib.cells[*cell_idx].read().unwrap();

            let layout = cell.layout.as_ref().unwrap();

            *cell_info = VlsirCell {
                index: Some(*cell_idx),
                num_shapes: Some(layout.elems.len() as u64),
            };

            if layout.elems.len() == 0 {
                continue;
            }

            let bbox = layout.bbox();
            let center = bbox.center();
            let Point { x: x_min, y: y_min } = bbox.p0;
            let Point { x: x_max, y: y_max } = bbox.p1;

            update_viewport_event_writer.send(UpdateViewportEvent {
                viewport: ViewportDimensions {
                    x_min: x_min as i64,
                    x_max: x_max as i64,
                    y_min: y_min as i64,
                    y_max: y_max as i64,
                    center,
                },
            });

            // info!("Cell: {}, num shapes: {num_shapes}", cell.name);

            for Element {
                net, layer, inner, ..
            } in layout.elems.iter()
            {
                let net = Net(net.clone());

                let layer = lib_layers
                    .get(*layer)
                    .expect("This Element's LayerKey does not exist in this Library's Layers")
                    .layernum as u16;

                let color = layers
                    .get(&layer)
                    .expect("This Element's layer num does not exist in our Layers Resource")
                    .color;

                match inner {
                    Shape::Rect(r) => {
                        let BoundBox { p0, p1 } = r.bbox();
                        let rect = layout21raw::Rect { p0, p1 };
                        import_rect_event_writer.send(ImportRectEvent {
                            rect: Rect(rect),
                            net,
                            layer,
                            color,
                        });
                    }
                    Shape::Polygon(p) => {
                        import_poly_event_writer.send(ImportPolyEvent {
                            poly: Poly(p.clone()),
                            net,
                            layer,
                            color,
                        });
                    }
                    Shape::Path(p) => {
                        import_path_event_writer.send(ImportPathEvent {
                            path: Path(p.clone()),
                            net,
                            layer,
                            color,
                        });
                    }
                }
            }

            load_cell_complete_event_writer.send(LoadCellCompleteEvent);

            let d = t.elapsed();
            info!("Total Layout21 Proto import duration {:?}", d);
        }
    }
}

pub fn import_rect_system(
    mut commands: Commands,
    mut import_rect_event_reader: EventReader<ImportRectEvent>,
) {
    for ImportRectEvent {
        rect,
        net,
        layer,
        color,
    } in import_rect_event_reader.iter()
    {
        let (width, height) = (*rect).bbox().size();

        let layout21raw::Rect {
            p0: bottom_left, ..
        } = **rect;

        let Point { x: x_min, y: y_min } = bottom_left;

        let lyon_rect = lyon_shapes::Rectangle {
            origin: lyon_shapes::RectangleOrigin::BottomLeft,
            extents: (width as f32, height as f32).into(),
        };

        let transform =
            Transform::from_translation(Vec3::new(x_min as f32, y_min as f32, *layer as f32));

        let shape_lyon = GeometryBuilder::build_as(
            &lyon_rect,
            DrawMode::Outlined {
                fill_mode: FillMode {
                    color: *color.clone().set_a(ALPHA),
                    options: FillOptions::default(),
                },
                outline_mode: StrokeMode {
                    options: StrokeOptions::default().with_line_width(WIDTH),
                    color: *color,
                },
            },
            transform,
        );

        let shape = ShapeBundle {
            net: net.to_owned(),
            shape_lyon,
            layer: InLayer(*layer),
        };

        commands.spawn_bundle(RectBundle {
            rect: rect.clone(),
            shape,
        });
    }
}

pub fn import_poly_system(
    mut commands: Commands,
    mut import_poly_event_reader: EventReader<ImportPolyEvent>,
) {
    for ImportPolyEvent {
        net,
        poly,
        layer,
        color,
    } in import_poly_event_reader.iter()
    {
        let lyon_poly = lyon_shapes::Polygon {
            points: (*poly)
                .points
                .iter()
                .map(|Point { x, y }| Vec2::new(*x as f32, *y as f32))
                .collect::<Vec<Vec2>>(),
            closed: true,
        };

        let transform = Transform::from_translation(Vec3::new(0.0, 0.0, *layer as f32));

        let shape_lyon = GeometryBuilder::build_as(
            &lyon_poly,
            DrawMode::Outlined {
                fill_mode: FillMode {
                    color: *color.clone().set_a(ALPHA),
                    options: FillOptions::default(),
                },
                outline_mode: StrokeMode {
                    options: StrokeOptions::default().with_line_width(WIDTH),
                    color: *color,
                },
            },
            transform,
        );

        let shape = ShapeBundle {
            net: net.to_owned(),
            layer: InLayer(*layer),
            shape_lyon,
        };

        commands.spawn_bundle(PolyBundle {
            poly: poly.clone(),
            shape,
        });
    }
}

pub fn import_path_system(
    mut commands: Commands,
    mut import_path_event_reader: EventReader<ImportPathEvent>,
) {
    for ImportPathEvent {
        net,
        path,
        layer,
        color,
    } in import_path_event_reader.iter()
    {
        let lyon_path = lyon_shapes::Polygon {
            points: path
                .points
                .iter()
                .map(|Point { x, y }| Vec2::new(*x as f32, *y as f32))
                .collect::<Vec<Vec2>>(),
            closed: false,
        };

        let transform = Transform::from_translation(Vec3::new(0.0, 0.0, *layer as f32));

        let shape_lyon = GeometryBuilder::build_as(
            &lyon_path,
            DrawMode::Outlined {
                fill_mode: FillMode {
                    color: *color.clone().set_a(ALPHA),
                    options: FillOptions::default(),
                },
                outline_mode: StrokeMode {
                    options: StrokeOptions::default().with_line_width(path.width as f32),
                    color: *color,
                },
            },
            transform,
        );

        let shape = ShapeBundle {
            net: net.clone(),
            layer: InLayer(*layer),
            shape_lyon,
        };

        commands.spawn_bundle(PathBundle {
            path: path.clone(),
            shape,
        });
    }
}

#[cfg(test)]
mod tests {
    use layout21raw::{
        gds::gds21::GdsLibrary, gds::GdsImporter, proto::ProtoExporter, LayoutResult,
    };
    use vlsir::save;

    #[test]
    fn make_oscibear_proto() -> LayoutResult<()> {
        let gds = GdsLibrary::load("./user_analog_project_wrapper.gds").unwrap();

        // Convert to Layout21::Raw
        let lib = GdsImporter::import(&gds, None)?;
        println!("{}", lib.name);
        println!("{}", lib.cells.len());

        // // Convert to ProtoBuf
        let p = ProtoExporter::export(&lib)?;
        println!("{}", p.domain);

        save(&p, "oscibear.proto").unwrap();
        Ok(())
    }
}
