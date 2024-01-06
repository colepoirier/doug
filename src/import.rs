use crate::editing::ShapeStack;
use crate::shapes::{
    GeoPolygon, GeoRect, InLayer, Path, PathBundle, Poly, PolyBundle, Rect, RectBundle, ShapeBundle,
};
use crate::ui::{LayersUIState, LibInfoUIDropdownState};
use crate::{ALPHA, WIDTH};

use std::collections::HashMap;

use bevy::prelude::*;

use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_mod_picking::PickableBundle;
use bevy_prototype_lyon::entity;
use bevy_prototype_lyon::prelude::{
    shapes as lyon_shapes, Fill, Geometry, GeometryBuilder, ShapeBundle as LyonShapeBundle, Stroke,
    StrokeOptions,
};

use futures_lite::future;

use layout21::{
    raw::{
        self, proto::proto, proto::ProtoImporter, Abstract, BoundBox, BoundBoxTrait, Cell, Element,
        Instance, Layout, Library, Point, Shape,
    },
    utils::Ptr,
};

use std::slice::Iter;

#[derive(Resource, Component, Debug)]
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

#[derive(Resource, Debug, Default)]
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

#[derive(Resource, Debug, Default, Clone, Deref, DerefMut)]
pub struct Layers(HashMap<u8, Layer>);

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct VlsirCell {
    pub index: Option<usize>,
    pub num_shapes: Option<u64>,
}

#[derive(Debug, Default, Clone)]
pub struct CellContentsInfo {
    pub cell_name: String,
    pub layout: Option<LayoutInfo>,
    pub abstrakt: Option<AbstraktInfo>,
}

#[derive(Debug, Default, Clone)]
pub struct LayoutInfo {
    pub layout_name: String,
    pub elems: usize,
    pub insts: usize,
    pub annotations: usize,
}

#[derive(Debug, Default, Clone)]
pub struct AbstraktInfo {
    pub abstrakt_name: String,
    pub outline: BoundBox,
    pub ports: usize,
    pub blockages: usize,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum ImportSet {
    ResetWorld,
    Import,
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
            // .add_startup_system(send_import_event_system)
            .add_systems(
                Update,
                (
                    reset_state_on_new_lib_import,
                    reset_state_on_new_cell_import,
                ),
            )
            .add_systems(
                Update,
                (
                    spawn_vlsir_open_task_sytem,
                    handle_vlsir_open_task_system,
                    vlsir_open_task_duration_system,
                    import_lib_system,
                    load_cell_system,
                    load_cell_complete_system,
                    import_path_system,
                    import_rect_system,
                    import_poly_system,
                ),
            );
    }
}

#[derive(Component, Debug, Default, Clone, PartialEq, PartialOrd, Deref, DerefMut)]
pub struct Net(pub Option<String>);

#[derive(Event, Debug, Default, Clone)]
pub struct OpenVlsirLibEvent {
    pub path: String,
}

#[derive(Event, Debug, Default, Clone, Copy)]
pub struct OpenVlsirLibCompleteEvent;

#[derive(Event, Debug, Default, Clone, Copy)]
pub struct ImportLibCompleteEvent;

#[derive(Event, Debug, Default, Clone, Copy, Deref, DerefMut)]
pub struct LoadCellEvent(pub usize);

#[derive(Event, Debug, Default, Clone, Copy)]
pub struct LoadCellCompleteEvent;

#[derive(Event, Debug, Clone)]
pub struct ImportRectEvent {
    pub rect: Rect,
    pub net: Net,
    pub layer: u8,
    pub color: Color,
}

#[derive(Event, Debug, Clone)]
pub struct ImportPolyEvent {
    pub poly: Poly,
    pub net: Net,
    pub layer: u8,
    pub color: Color,
}

#[derive(Event, Debug, Default, Clone)]
pub struct ImportPathEvent {
    pub path: Path,
    pub net: Net,
    pub layer: u8,
    pub color: Color,
}

pub fn load_cell_complete_system(
    mut load_complete_event_reader: EventReader<LoadCellCompleteEvent>,
) {
}

#[derive(Debug, Component, Deref, DerefMut)]
pub struct VlsirOpenLibraryTask(Task<Library>);

pub fn spawn_vlsir_open_task_sytem(
    mut commands: Commands,
    mut vlsir_lib: ResMut<VlsirLib>,
    mut open_vlsir_lib_event_reader: EventReader<OpenVlsirLibEvent>,
) {
    for OpenVlsirLibEvent { path } in open_vlsir_lib_event_reader.read() {
        vlsir_lib.path = Some(path.clone());

        let path = vlsir_lib.path.clone().unwrap();

        let thread_pool = AsyncComputeTaskPool::get();

        let task: Task<Library> = thread_pool.spawn(async move {
            // enable to test UI Lib Info "Library:" loading spinner animation
            // std::thread::sleep(std::time::Duration::from_secs(5));
            let plib: proto::Library = proto::open(path).unwrap();
            ProtoImporter::import(&plib, None).unwrap()
        });

        let task = VlsirOpenLibraryTask(task);

        commands.spawn(task);
    }
}

pub fn handle_vlsir_open_task_system(
    mut commands: Commands,
    mut lib: ResMut<VlsirLib>,
    mut vlsir_open_task_q: Query<(Entity, &mut VlsirOpenLibraryTask)>,
    mut vlsir_open_lib_complete_event_writer: EventWriter<OpenVlsirLibCompleteEvent>,
) {
    for (entity, mut task) in vlsir_open_task_q.iter_mut() {
        if let Some(vlsir_lib) = future::block_on(future::poll_once(&mut **task)) {
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
    for OpenVlsirLibEvent { path: p } in open_vlsir_lib_event_reader.read() {
        *duration = time.elapsed_seconds_f64();
        *path = Some(p.clone());
    }

    for _ in open_vlsir_lib_complete_event_reader.read() {
        info!(
            "Vlisr open lib file '{path:?}' task duration {:?}",
            time.elapsed_seconds_f64() - *duration
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
    for _ in vlsir_open_lib_complete_event_reader.read() {
        let lib = vlsir_lib.lib.as_ref().unwrap();
        {
            let lib_layers = &lib.layers.read().unwrap().slots;

            for raw::Layer { layernum, name, .. } in lib_layers.values() {
                let num = *layernum as u8;
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

        // info!("Cell Names: {cell_names:?}");

        let longest_name = cell_names.iter().max().unwrap();

        info!(
            "Longest cell name: {{ len: {} name: {}}}",
            longest_name.chars().count(),
            longest_name
        );

        // let largest_magnitudes = lib
        //     .cells
        //     .iter()
        //     .map(|c| {
        //         let bbox = c
        //             .read()
        //             .unwrap()
        //             .layout
        //             .as_ref()
        //             .unwrap_or(&raw::Layout::default())
        //             .bbox();
        //         if bbox.is_empty() {
        //             (0, 0)
        //         } else {
        //             let Point { x: x_min, y: y_min } = bbox.p0;
        //             let Point { x: x_max, y: y_max } = bbox.p1;
        //             let x = x_min.abs().max(x_max.abs());
        //             let y = y_min.abs().max(y_max.abs());
        //             (x, y)
        //         }
        //     })
        //     .collect::<Vec<(isize, isize)>>();

        let dbg_cell_contents_info = lib
            .cells
            .iter()
            .map(|c| {
                let c = c.read().unwrap();

                let name = c.name.clone();

                let layout = if let Some(Layout {
                    annotations,
                    elems,
                    insts,
                    name,
                }) = c.layout.as_ref()
                {
                    Some(LayoutInfo {
                        layout_name: name.clone(),
                        elems: elems.len(),
                        insts: insts.len(),
                        annotations: annotations.len(),
                    })
                } else {
                    None
                };

                let abstrakt = if let Some(Abstract {
                    name,
                    outline,
                    ports,
                    blockages,
                }) = c.abs.as_ref()
                {
                    Some(AbstraktInfo {
                        abstrakt_name: name.clone(),
                        outline: outline.points.bbox(),
                        ports: ports.len(),
                        blockages: blockages.values().fold(0, |mut acc, b| {
                            acc += b.len();
                            acc
                        }),
                    })
                } else {
                    None
                };
                CellContentsInfo {
                    cell_name: name.clone(),
                    layout,
                    abstrakt,
                }
            })
            .collect::<Vec<CellContentsInfo>>();

        // let mut f = std::fs::File::create(format!("{}_cell_contents.dbg", lib.name)).unwrap();
        // use std::io::Write;
        // f.write(format!("{dbg_cell_contents_info:#?}").as_bytes())
        //     .unwrap();

        // let max_magnitudes =
        //     largest_magnitudes
        //         .iter()
        //         .fold(raw::Point::default(), |mut acc, &(x, y)| {
        //             acc.x = acc.x.max(x);
        //             acc.y = acc.y.max(y);
        //             acc
        //         });

        // let max_x = max_magnitudes.x;
        // let max_y = max_magnitudes.y;

        // info!("Largest cell extents in this library: [ x: {max_x}, y: {max_y} ]");

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
    mut shape_stack: ResMut<ShapeStack>,
    mut ui_dropdown_state: ResMut<LibInfoUIDropdownState>,
    mut ui_layer_state: ResMut<LayersUIState>,
    mut vlsir_open_lib_event_reader: EventReader<OpenVlsirLibEvent>,
) {
    for _ in vlsir_open_lib_event_reader.read() {
        info!("All state reset on new lib import!");

        *layer_colors = LayerColors::default();
        *layers = Layers::default();
        *vlsir_lib = VlsirLib::default();
        *shape_stack = ShapeStack::default();
        ui_dropdown_state.selected = 0;
        ui_layer_state.layers = vec![];

        for e in query.iter() {
            commands.entity(e).despawn();
        }
    }
}

pub fn reset_state_on_new_cell_import(
    mut commands: Commands,
    query: Query<Entity, With<entity::Path>>,
    mut load_cell_event_reader: EventReader<LoadCellEvent>,
    mut shape_stack: ResMut<ShapeStack>,
) {
    for _ in load_cell_event_reader.read() {
        *shape_stack = ShapeStack::default();
        for e in query.iter() {
            commands.entity(e).despawn();
        }
    }
}

pub fn load_cell_system(
    vlsir_lib: Res<VlsirLib>,
    mut cell_info: ResMut<VlsirCell>,
    layers: Res<Layers>,
    mut load_cell_event_reader: EventReader<LoadCellEvent>,
    mut load_cell_complete_event_writer: EventWriter<LoadCellCompleteEvent>,
    mut import_rect_event_writer: EventWriter<ImportRectEvent>,
    mut import_poly_event_writer: EventWriter<ImportPolyEvent>,
    mut import_path_event_writer: EventWriter<ImportPathEvent>,
    windows: Query<&Window>,
    mut camera_q: Query<(&mut Transform, &mut OrthographicProjection, &Camera)>,
) {
    for &cell_idx in load_cell_event_reader.read() {
        if let (Some(lib), Some(_)) = (vlsir_lib.lib.as_ref(), layers.iter().nth(0)) {
            let t = std::time::Instant::now();

            cell_info.index = Some(*cell_idx);

            let lib_layers = &lib.layers;

            let cell = &lib.cells[*cell_idx];

            let len_elems = cell.read().unwrap().layout.as_ref().unwrap().elems.len();
            let len_insts = cell.read().unwrap().layout.as_ref().unwrap().insts.len();

            if len_elems == 0 && len_insts == 0 {
                continue;
            }

            // let mut f = std::fs::File::create("cell_insts_debug").unwrap();
            // use std::io::Write;
            // f.write(
            //     format!(
            //         "Cell '{}' num_el: {} instances: {:#?}",
            //         cell.read().unwrap().layout.as_ref().unwrap().name,
            //         cell.read().unwrap().layout.as_ref().unwrap().elems.len(),
            //         cell.read().unwrap().layout.as_ref().unwrap().insts
            //     )
            //     .as_bytes(),
            // )
            // .unwrap();

            let mut shape_count: u64 = 0;

            import_cell_shapes(
                &cell,
                false,
                &mut shape_count,
                &Point::default(),
                &mut cell_info,
                &lib_layers,
                &layers,
                &mut import_rect_event_writer,
                &mut import_poly_event_writer,
                &mut import_path_event_writer,
                &windows,
                &mut camera_q,
            );

            cell_info.num_shapes = Some(shape_count);

            load_cell_complete_event_writer.send(LoadCellCompleteEvent);

            let d = t.elapsed();
            info!("Total Layout21 Proto import duration {:?}", d);
        }
    }
}

pub fn import_cell_shapes(
    cell: &Ptr<Cell>,
    mut bbox_set: bool,
    shape_count: &mut u64,
    offset: &Point,
    cell_info: &mut ResMut<VlsirCell>,
    lib_layers: &Ptr<raw::Layers>,
    layers: &Res<Layers>,
    import_rect_event_writer: &mut EventWriter<ImportRectEvent>,
    import_poly_event_writer: &mut EventWriter<ImportPolyEvent>,
    import_path_event_writer: &mut EventWriter<ImportPathEvent>,
    windows: &Query<&Window>,
    camera_q: &mut Query<(&mut Transform, &mut OrthographicProjection, &Camera)>,
) {
    let read_cell = cell.read().unwrap();
    let read_lib_layers = lib_layers.read().unwrap();

    let layout = read_cell.layout.as_ref().unwrap();

    let bbox = layout.bbox();

    if !bbox_set {
        if !bbox.is_empty() {
            let center = bbox.center();
            let Point { x: x_min, y: y_min } = bbox.p0;
            let Point { x: x_max, y: y_max } = bbox.p1;

            let window = windows.single();

            let width = (x_max - x_min) as f32;
            let height = (y_max - y_min) as f32;

            info!("width: {width}, height: {height}");

            let padding = 100.0;

            let screen_width = window.width() - (2.0 * padding);
            let screen_height = window.height() - (2.0 * padding);

            let width_ratio = width / screen_width;
            let height_ratio = height / screen_height;

            info!("width/viewport_width: {width_ratio}, height/viewport_height: {height_ratio}");

            let scale = width_ratio.max(height_ratio);

            let world_width = screen_width * scale;
            let world_height = screen_height * scale;

            info!("world_width: {world_width}, world_height: {world_height}");

            let (mut cam_t, mut proj, cam) = camera_q.single_mut();

            proj.scale = scale;

            cam_t.translation.x = center.x as f32;
            cam_t.translation.y = center.y as f32;

            bbox_set = true;
        }
    }

    for Element {
        net, layer, inner, ..
    } in layout.elems.iter()
    {
        if *shape_count % 1_000 == 0 {
            info!("Shapes spawned: {}", shape_count);
        }

        // if *shape_count > 90_000 {
        //     return;
        // }

        let net = Net(net.clone());

        let layer = read_lib_layers
            .get(*layer)
            .expect("This Element's LayerKey does not exist in this Library's Layers")
            .layernum as u8;

        let color = layers
            .get(&layer)
            .expect(&format!(
                "This Element's layer does not exist in our Layers Resource"
            ))
            .color;

        match inner {
            Shape::Rect(r) => {
                let BoundBox { p0, p1 } = r.bbox();
                let p0 = p0.shift(offset);
                let p1 = p1.shift(offset);

                let p0 = (p0.x as i32, p0.y as i32);
                let p1 = (p1.x as i32, p1.y as i32);

                let rect = GeoRect::new(p0, p1);
                import_rect_event_writer.send(ImportRectEvent {
                    rect: Rect(rect),
                    net,
                    layer,
                    color,
                });
            }
            Shape::Polygon(p) => {
                let poly = GeoPolygon::new(
                    p.points
                        .iter()
                        .map(|p| {
                            let p = p.shift(offset);
                            (p.x as i32, p.y as i32)
                        })
                        .collect(),
                    vec![],
                );
                import_poly_event_writer.send(ImportPolyEvent {
                    poly: Poly(poly),
                    net,
                    layer,
                    color,
                });
            }
            Shape::Path(p) => {
                if p.points.len() > 2 {
                    let mut p = p.clone();
                    p.points = p.points.iter().map(|p| p.shift(offset)).collect();
                    import_path_event_writer.send(ImportPathEvent {
                        path: Path(p.clone()),
                        net,
                        layer,
                        color,
                    });
                }
            }
        }

        *shape_count += 1;
    }

    for Instance {
        inst_name,
        cell,
        loc,
        reflect_vert,
        angle,
    } in layout.insts.iter()
    {
        import_cell_shapes(
            cell,
            bbox_set,
            shape_count,
            loc,
            cell_info,
            lib_layers,
            layers,
            import_rect_event_writer,
            import_poly_event_writer,
            import_path_event_writer,
            &windows,
            camera_q,
        );
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
    } in import_rect_event_reader.read()
    {
        let x_min = rect.min().x as f32;
        let y_min = rect.min().y as f32;

        let x_max = rect.max().x as f32;
        let y_max = rect.max().y as f32;

        let lyon_poly = lyon_shapes::Polygon {
            points: vec![
                (x_min, y_min).into(),
                (x_max, y_min).into(),
                (x_max, y_max).into(),
                (x_min, y_max).into(),
            ],
            closed: true,
        };

        let shape = shape_bundle(net, layer, lyon_poly, color);

        commands.spawn(RectBundle {
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
    } in import_poly_event_reader.read()
    {
        let lyon_poly = lyon_shapes::Polygon {
            points: poly
                .exterior()
                .coords()
                .map(|c| Vec2::new(c.x as f32, c.y as f32))
                .collect::<Vec<Vec2>>(),
            closed: true,
        };

        let shape = shape_bundle(net, layer, lyon_poly, color);

        commands.spawn(PolyBundle {
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
    } in import_path_event_reader.read()
    {
        let num_points = path.points.len();
        let mut forward_poly_points = Vec::with_capacity(num_points);
        let mut backward_poly_points = Vec::with_capacity(num_points);
        info!("width of path is {}", path.width);
        info!("{:?}", path.points);
        assert_eq!(
            path.width % 2,
            0,
            "width must be even for our code's assumptions to hold!"
        );
        let half_width = (path.width / 2) as isize; // assuming that widths are even!
        for ix in 0..num_points {
            let p0 = path.points[ix];
            let p1 = path.points[(ix + 1) % num_points];
            // let corrected_point = p0.shift(offset);
            if p0.x == p1.x {
                // vertical
                forward_poly_points.push(Point {
                    x: p0.x + half_width,
                    y: p0.y,
                });
                backward_poly_points.push(Point {
                    x: p0.x - half_width,
                    y: p0.y,
                });
            } else {
                // horizontal
                forward_poly_points.push(Point {
                    x: p0.x,
                    y: p0.y - half_width,
                });
                backward_poly_points.push(Point {
                    x: p0.x,
                    y: p0.y + half_width,
                });
            }
        }
        let points = forward_poly_points
            .into_iter()
            .chain(backward_poly_points.into_iter().rev())
            .map(|Point { x, y }| Vec2::new(x as f32, y as f32))
            .collect();

        info!("{points:?}");

        let lyon_path = lyon_shapes::Polygon {
            points,
            closed: true,
        };

        // let lyon_path = lyon_shapes::Polygon {
        //     points: path
        //         .points
        //         .iter()
        //         .map(|Point { x, y }| Vec2::new(*x as f32, *y as f32))
        //         .collect::<Vec<Vec2>>(),
        //     closed: false,
        // };

        // let shape_lyon = GeometryBuilder::build_as(
        //     &lyon_path,
        //     DrawMode::Outlined {
        //         fill_mode: FillMode {
        //             color: *color.clone().set_a(ALPHA),
        //             options: FillOptions::default(),
        //         },
        //         outline_mode: StrokeMode {
        //             // BUG: this is a bug, creates a rectangle that's border is like 900 when the shape is only 10px,
        //             // the border makes up the entire shape, the actual shape is a tiny line in the middle completely
        //             // covered by the stroke of the border
        //             options: StrokeOptions::default().with_line_width(path.width as f32),
        //             color: *color,
        //         },
        //     },
        //     transform,
        // );

        let shape = shape_bundle(net, layer, lyon_path, color);

        commands.spawn(PathBundle {
            path: path.clone(),
            shape,
        });
    }
}

pub fn shape_bundle(net: &Net, layer: &u8, geom: impl Geometry, color: &Color) -> ShapeBundle {
    let shape_lyon = LyonShapeBundle {
        path: GeometryBuilder::build_as(&geom),
        spatial: SpatialBundle::from_transform(Transform::from_translation(Vec3::new(
            0.0,
            0.0,
            *layer as f32,
        ))),
        ..default()
    };

    ShapeBundle {
        net: net.clone(),
        layer: InLayer(*layer),
        shape_lyon,
        fill: Fill::color(*color.clone().set_a(ALPHA)),
        stroke: Stroke {
            options: StrokeOptions::default().with_line_width(WIDTH),
            color: *color,
        },
        pickable: PickableBundle::default(),
    }
}

#[cfg(test)]
mod tests {
    use layout21::raw::{
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
