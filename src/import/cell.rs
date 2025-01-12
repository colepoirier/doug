use std::{sync::Arc, time::Instant};

use bevy::tasks::futures_lite::future;
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use layout21::{
    raw::{self, Cell, Element, Instance, Library, Point},
    utils::Ptr,
};

use crate::import::shapes::Shape;
use crate::shapes::{InLayer, Net};

use super::RectMesh;
use super::{
    shapes::{self, SpawnShape},
    Layer, Layers, VlsirLib,
};

pub struct LoadCellPlugin;

impl Plugin for LoadCellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_load_cell_task_sytem.run_if(resource_added::<LoadCell>),
                handle_load_cell_task_system.run_if(resource_exists::<LoadCell>),
            ),
        );
    }
}

#[derive(Resource)]
pub struct LoadCell {
    pub index: usize,
}

#[derive(Component)]
struct LoadCellTask(Task<LoadCellResult>);

struct LoadCellResult {
    start_time: Instant,
    spawn_shapes: Vec<SpawnShape>,
}

fn spawn_load_cell_task_sytem(
    load_vlsir_lib: Res<LoadCell>,
    vlsir_lib: Res<VlsirLib>,
    layers: Res<Layers>,
    mut commands: Commands,
) {
    println!("running because resource added");
    let index = load_vlsir_lib.index;

    let lib = vlsir_lib.lib.clone();
    let layers = layers.0.clone();

    commands.spawn(LoadCellTask(AsyncComputeTaskPool::get().spawn(
        async move {
            let start_time = Instant::now();

            let spawn_shapes = load_cell(index, &lib, &layers);

            LoadCellResult {
                start_time,
                spawn_shapes,
            }
        },
    )));
}

fn handle_load_cell_task_system(
    mut commands: Commands,
    mut load_cell_task_q: Query<(Entity, &mut LoadCellTask)>,
    rect_mesh: Res<RectMesh>,
) {
    println!("handle_load_cell_task_system running because resource exists");
    for (entity, mut task) in &mut load_cell_task_q {
        println!("handle_load_cell_task_system running because component exists");
        if let Some(load_vlsir_lib_result) = future::block_on(future::poll_once(&mut task.0)) {
            let LoadCellResult {
                start_time,
                spawn_shapes,
            } = load_vlsir_lib_result;
            println!(
                "Loaded cell with {} shapes in {}",
                spawn_shapes.len(),
                start_time.elapsed().as_secs_f32()
            );
            commands.entity(entity).despawn();
            commands.remove_resource::<LoadCell>();

            for s in spawn_shapes {
                match s.shape {
                    Shape::Rect(rect) => {
                        spawn_rect(
                            &mut commands,
                            rect,
                            rect_mesh.clone(),
                            s.layer,
                            s.net,
                            s.color,
                        );
                    }
                    Shape::Poly(poly) => {
                        spawn_poly(
                            &mut commands,
                            poly,
                            rect_mesh.clone(),
                            s.layer,
                            s.net,
                            s.color,
                        );
                    }
                    Shape::Path(path) => {
                        spawn_poly(
                            &mut commands,
                            path,
                            rect_mesh.clone(),
                            s.layer,
                            s.net,
                            s.color,
                        );
                    }
                }
            }
        }
    }
}

fn spawn_rect(
    commands: &mut Commands,
    rect: Rect,
    rect_mesh: Handle<Mesh>,
    layer: InLayer,
    net: Net,
    color: Handle<ColorMaterial>,
) {
    let scale = (rect.max.x - rect.min.x, rect.max.y - rect.min.y, 1.0).into();
    let translation = (
        rect.min.x + ((rect.max.x - rect.min.x) * 0.5),
        rect.min.y + ((rect.max.y - rect.min.y) * 0.5),
        *layer as f32,
    )
        .into();

    commands.spawn((
        Mesh2d(rect_mesh.clone()),
        MeshMaterial2d(color),
        Transform::default()
            .with_scale(scale)
            .with_translation(translation),
        layer,
        net,
    ));
}

fn spawn_poly(
    commands: &mut Commands,
    poly: Vec<Rect>,
    rect_mesh: Handle<Mesh>,
    layer: InLayer,
    net: Net,
    color: Handle<ColorMaterial>,
) {
    if poly.len() < 2 {
        return;
    }
    let rect = poly[0];
    let translation = (rect.min.x, rect.min.y, *layer as f32).into();
    commands
        .spawn_empty()
        .insert((
            Transform::default().with_translation(translation),
            Visibility::Visible,
            layer,
            net,
        ))
        .with_children(|parent| {
            for rect in poly.iter() {
                let scale = (rect.max.x - rect.min.x, rect.max.y - rect.min.y, 1.0).into();
                let translation = (
                    rect.min.x + ((rect.max.x - rect.min.x) * 0.5),
                    rect.min.y + ((rect.max.y - rect.min.y) * 0.5),
                    1.0,
                )
                    .into();

                // info!("scale {scale:?}");
                // info!("translation {translation:?}");
                parent.spawn((
                    Mesh2d(rect_mesh.clone()),
                    MeshMaterial2d(color.clone()),
                    Transform::default()
                        .with_scale(scale)
                        .with_translation(translation),
                ));
            }
        });
}

pub fn load_cell(
    cell_idx: usize,
    lib: &Arc<Library>,
    layers: &Arc<HashMap<u8, Layer>>,
) -> Vec<SpawnShape> {
    let t = std::time::Instant::now();

    let lib_layers = &lib.layers;
    let cell = &lib.cells[cell_idx];

    let mut spawn_shapes = vec![];

    import_cell_shapes(
        &mut spawn_shapes,
        &cell,
        &Point::default(),
        &lib_layers,
        layers,
    );

    let d = t.elapsed();
    info!("Total load cell duration {:?}", d);
    return spawn_shapes;
}

pub fn import_cell_shapes(
    spawn_shapes: &mut Vec<SpawnShape>,
    cell: &Ptr<Cell>,
    offset: &Point,
    lib_layers: &Ptr<raw::Layers>,
    layers: &Arc<HashMap<u8, Layer>>,
) {
    let read_cell = cell.read().unwrap();
    let read_lib_layers = lib_layers.read().unwrap();

    let layout = read_cell.layout.as_ref().unwrap();

    for Element {
        net, layer, inner, ..
    } in layout.elems.iter()
    {
        let layer = read_lib_layers
            .get(*layer)
            .expect("This Element's LayerKey does not exist in this Library's Layers")
            .layernum as u8;

        let color = layers
            .get(&layer)
            .expect("This Element's layer does not exist in our Layers Resource")
            .color
            .clone();

        match inner {
            raw::Shape::Rect(rect) => {
                let mut rect = *rect;
                rect.p0 = rect.p0.shift(offset);
                rect.p1 = rect.p1.shift(offset);
                spawn_shapes.push(shapes::import_rect(rect, net, layer, color));
            }
            raw::Shape::Polygon(poly) => {
                let mut poly = poly.clone();
                poly.points = poly.points.iter().map(|p| p.shift(offset)).collect();
                spawn_shapes.push(shapes::import_poly(poly, net, layer, color));
            }
            raw::Shape::Path(path) => {
                if path.points.len() > 2 {
                    let mut path = path.clone();
                    path.points = path.points.iter().map(|p| p.shift(offset)).collect();
                    spawn_shapes.push(shapes::import_path(path, net, layer, color));
                }
            }
        }
    }

    for Instance {
        inst_name: _inst_name,
        cell,
        loc,
        reflect_vert: _reflect_vert,
        angle: _angle,
    } in layout.insts.iter()
    {
        import_cell_shapes(spawn_shapes, cell, loc, lib_layers, layers);
    }
}

pub fn count_cell_shapes(shape_count: &mut usize, cell: &Ptr<Cell>) {
    let read_cell = cell.read().unwrap();
    let layout = read_cell.layout.as_ref().unwrap();

    *shape_count += layout.elems.iter().count();

    layout
        .insts
        .iter()
        .for_each(|c| count_cell_shapes(shape_count, &c.cell));
}

#[test]
fn print_cell_shape_counts() {
    use layout21::raw::proto::proto;
    use layout21::raw::proto::ProtoImporter;
    let plib: proto::Library =
        proto::open("/Users/colepoirier/Dropbox/rust_2020_onwards/doug/doug/libs/oscibear.proto")
            .unwrap();
    let lib = ProtoImporter::import(&plib, None).unwrap();

    let mut res = lib
        .cells
        .iter()
        .enumerate()
        .map(|(idx, cell)| {
            let read_cell = cell.read().unwrap();
            let name = &read_cell.name;
            let mut shape_count = 0;
            count_cell_shapes(&mut shape_count, cell);
            (idx, name.clone(), shape_count)
        })
        .collect::<Vec<(usize, String, usize)>>();
    res.sort_unstable_by(|a, b| a.2.cmp(&b.2));

    for (idx, name, shape_count) in res {
        println!("{idx} {name} {shape_count}");
    }
}
