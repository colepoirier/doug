use crate::LayerColors;
use crate::{InLayer, LayerBundle, LayerColor, LayerNum, WIDTH};
use bevy::prelude::*;
use bevy::render::camera::OrthographicProjection;
use bevy_prototype_lyon::entity;
use bevy_prototype_lyon::prelude::{
    DrawMode, FillOptions, GeometryBuilder, ShapeColors, StrokeOptions,
};
use bevy_prototype_lyon::shapes;
use std::io::{BufWriter, Write};

use layout21::raw::gds;
use layout21::raw::proto::proto;
use layout21::raw::proto::ProtoExporter;
use layout21::raw::LayoutResult;

use crate::LoadCompleteEvent;
use crate::ALPHA;

use bevy::utils::HashMap;

use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::*;

#[derive(Default, Bundle)]
pub struct RapierShapeBundle {
    #[bundle]
    collider: ColliderBundle,
    sync: ColliderPositionSync,
}

#[derive(Inspectable, Debug, Default)]
pub struct Rect;

#[derive(Inspectable, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nom(String);

#[derive(Default, Bundle)]
pub struct RectBundle {
    pub name: Nom,
    pub rect: Rect,
    pub layer: InLayer,
    #[bundle]
    pub shape_lyon: entity::ShapeBundle,
    #[bundle]
    pub shape_rapier: RapierShapeBundle,
}

#[derive(Inspectable, Debug, Default)]
pub struct Path;

#[derive(Default, Bundle)]
pub struct PathBundle {
    pub rect: Path,
    pub layer: InLayer,
    #[bundle]
    #[bundle]
    pub shape_lyon: entity::ShapeBundle,
    #[bundle]
    pub shape_rapier: RapierShapeBundle,
}

#[derive(Inspectable, Debug, Default)]
pub struct Polygon;

#[derive(Default, Bundle)]
pub struct PolygonBundle {
    pub rect: Polygon,
    pub layer: InLayer,
    #[bundle]
    #[bundle]
    pub shape_lyon: entity::ShapeBundle,
    #[bundle]
    pub shape_rapier: RapierShapeBundle,
}

pub fn test_load_proto_lib(
    commands: &mut Commands,
    layer_colors: &mut ResMut<LayerColors>,
    _load_complete_event_writer: &mut EventWriter<LoadCompleteEvent>,
    query: &mut Query<(&mut Transform, &mut OrthographicProjection)>,
) {
    let plib: proto::Library = proto::open(
        // "./dff1_lib.proto",
        "./oscibear.proto",
    )
    .unwrap();

    let mut layers = Vec::<i64>::new();
    for cell in plib.cells.iter() {
        layers.extend(
            cell.layout
                .as_ref()
                .unwrap()
                .shapes
                .iter()
                .map(|s| s.layer.as_ref().unwrap().number)
                .collect::<Vec<i64>>(),
        );
    }

    layers.sort();
    // info!("{}", layers.len());

    layers.dedup();
    // info!("{}", layers.len());
    // info!("{:?}", layers);

    let layers = layers
        .iter()
        .map(|&num| (num, layer_colors.get_color()))
        .collect::<Vec<(i64, Color)>>();

    // info!("{:?}", layers);

    let mut layer_map = HashMap::<u16, (Entity, Color)>::default();

    for (num, color) in layers {
        let id = commands
            .spawn_bundle(LayerBundle {
                num: LayerNum(num as u16),
                color: LayerColor(color),
                ..Default::default()
            })
            .id();

        layer_map.insert(num as u16, (id, color));
    }

    // let mut x_min: i64 = 0;
    // let mut x_max: i64 = 0;
    // let mut y_min: i64 = 0;
    // let mut y_max: i64 = 0;

    // plib.cells.iter().enumerate().for_each(|(i, cell)| {
    //     let mut rects = 0;
    //     let mut polys = 0;
    //     let mut paths = 0;
    //     for layer_shapes in cell.layout.as_ref().unwrap().shapes.iter()
    //     // .rev()
    //     // .take(10)
    //     {
    //         // info!("{:?}", cell);

    //         rects += layer_shapes.rectangles.len();
    //         polys += layer_shapes.polygons.len();
    //         paths += layer_shapes.paths.len();

    //         for proto::Rectangle {
    //             width,
    //             height,
    //             lower_left,
    //             ..
    //         } in layer_shapes.rectangles.iter()
    //         {
    //             let proto::Point { x, y } = lower_left.as_ref().unwrap();
    //             let width = *width;
    //             let height = *height;
    //             let x = *x;
    //             let y = *y;
    //             x_min = std::cmp::min(x_min, x);
    //             x_max = std::cmp::max(x_max, x + width);
    //             y_min = std::cmp::min(y_min, y);
    //             y_max = std::cmp::max(y_max, y + height);
    //         }
    //     }
    //     if paths > 1 {
    //         info!(
    //             "index: {}, name: {} rects: {}, polys: {}, paths: {}",
    //             i,
    //             cell.name,
    //             rects,
    //             polys,
    //             paths,
    //             // cell.layout.as_ref().unwrap().instances
    //         );
    //         info!(
    //             "x min: {}, max: {}, y min: {}, max: {}",
    //             x_min, x_max, y_min, y_max
    //         );
    //     }
    // });

    // info!(
    //     "x min: {}, max: {}, y min: {}, max: {}",
    //     x_min, x_max, y_min, y_max
    // );

    let mut x_min: i64 = 0;
    let mut x_max: i64 = 0;
    let mut y_min: i64 = 0;
    let mut y_max: i64 = 0;

    info!("{:?} {}", plib.units(), plib.units);

    let f = std::fs::File::create("debug.txt").unwrap();

    let mut writer = std::io::BufWriter::new(f);

    for cell in plib.cells.iter().nth(770) {
        let len = cell
            .layout
            .as_ref()
            .unwrap()
            .shapes
            .iter()
            .map(|s| s.paths.len())
            .collect::<Vec<usize>>();

        let len: usize = len.into_iter().sum();

        info!("{:?} {}", cell.name, len);
        // break;
        for layer_shapes in cell.layout.as_ref().unwrap().shapes.iter()
        // .rev()
        // .take(10)
        {
            // info!("{:?}", cell);
            let layer = layer_shapes.layer.as_ref().unwrap().number as u16;
            let (_, color) = layer_map.get(&layer).unwrap();
            let color = *color;
            let rects = layer_shapes
                .rectangles
                .iter()
                .map(
                    |proto::Rectangle {
                         width,
                         height,
                         lower_left,
                         net,
                     }| {
                        info!(
                            "width: {} height: {} lower_left: {:?} net: {:?}",
                            width, height, lower_left, net
                        );

                        writer
                            .write(
                                &format!(
                                    "lower_left: {:>9?} width: {:>9} height: {:>9} layer: {:>5?} net: {:>10?}\n",
                                    lower_left,
                                    width,
                                    height,
                                    layer_shapes.layer.as_ref().unwrap(),
                                    net,
                                )
                                .as_bytes(),
                            )
                            .unwrap();

                        let proto::Point { x, y } = lower_left.as_ref().unwrap();
                        let ix = *x;
                        let iy = *y;
                        x_min = std::cmp::min(x_min, ix);
                        x_max = std::cmp::max(x_max, ix + width);
                        y_min = std::cmp::min(y_min, iy);
                        y_max = std::cmp::max(y_max, iy + height);

                        let x = *x as f32;
                        let y = *y as f32;
                        let width = *width as f32;
                        let height = *height as f32;

                        let rect = shapes::Rectangle {
                            origin: shapes::RectangleOrigin::BottomLeft,
                            width,
                            height,
                        };

                        let transform = Transform::from_translation(Vec3::new(
                            x as f32,
                            y as f32,
                            layer as f32,
                        ));

                        let shape_lyon = GeometryBuilder::build_as(
                            &rect,
                            ShapeColors {
                                main: *color.clone().set_a(ALPHA),
                                outline: color,
                            },
                            DrawMode::Outlined {
                                fill_options: FillOptions::default(),
                                outline_options: StrokeOptions::default().with_line_width(WIDTH),
                            },
                            transform,
                        );

                        RectBundle {
                            name: Nom(net.clone()),
                            layer: InLayer(layer),
                            shape_rapier: RapierShapeBundle {
                                collider: ColliderBundle {
                                    shape: ColliderShape::cuboid(width, height),
                                    position: [x, y].into(),
                                    flags: (ActiveEvents::INTERSECTION_EVENTS
                                        | ActiveEvents::CONTACT_EVENTS)
                                        .into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            shape_lyon,
                            ..Default::default()
                        }
                    },
                )
                .collect::<Vec<RectBundle>>();

            // for r in rects.iter() {
            //     info!("{:?}", r.shape_lyon.transform)
            // }

            let polys = layer_shapes
                .polygons
                .iter()
                .map(|proto::Polygon { vertices, .. }| {
                    for p in vertices {
                        x_min = std::cmp::min(x_min, p.x);
                        x_max = std::cmp::max(x_max, p.x);
                        y_min = std::cmp::min(y_min, p.y);
                        y_max = std::cmp::max(y_max, p.y);
                    }

                    let poly = shapes::Polygon {
                        points: vertices
                            .iter()
                            .map(|proto::Point { x, y }| Vec2::new(*x as f32, *y as f32))
                            .collect::<Vec<Vec2>>(),
                        closed: true,
                    };

                    let transform = Transform::from_translation(Vec3::new(0.0, 0.0, layer as f32));

                    let shape_lyon = GeometryBuilder::build_as(
                        &poly,
                        ShapeColors {
                            main: *color.clone().set_a(ALPHA),
                            outline: color,
                        },
                        DrawMode::Outlined {
                            fill_options: FillOptions::default(),
                            outline_options: StrokeOptions::default().with_line_width(WIDTH),
                        },
                        transform,
                    );

                    let vertices = vertices
                        .iter()
                        .map(|proto::Point { x, y }| point![*x as f32, *y as f32])
                        .collect::<Vec<Point<f32>>>();

                    PolygonBundle {
                        layer: InLayer(layer),
                        shape_lyon,
                        shape_rapier: RapierShapeBundle {
                            collider: ColliderBundle {
                                shape: ColliderShape::convex_polyline(vertices).unwrap(),
                                flags: (ActiveEvents::INTERSECTION_EVENTS
                                    | ActiveEvents::CONTACT_EVENTS)
                                    .into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                })
                .collect::<Vec<PolygonBundle>>();

            let paths = layer_shapes
                .paths
                .iter()
                .map(|proto::Path { points, width, .. }| {
                    for p in points {
                        x_min = std::cmp::min(x_min, p.x);
                        x_max = std::cmp::max(x_max, p.x);
                        y_min = std::cmp::min(y_min, p.y);
                        y_max = std::cmp::max(y_max, p.y);
                    }

                    let path = shapes::Polygon {
                        points: points
                            .iter()
                            .map(|proto::Point { x, y }| Vec2::new(*x as f32, *y as f32))
                            .collect::<Vec<Vec2>>(),
                        closed: false,
                    };

                    let transform = Transform::from_translation(Vec3::new(0.0, 0.0, layer as f32));

                    let shape_lyon = GeometryBuilder::build_as(
                        &path,
                        ShapeColors {
                            main: *color.clone().set_a(ALPHA),
                            outline: color,
                        },
                        DrawMode::Outlined {
                            fill_options: FillOptions::default(),
                            outline_options: StrokeOptions::default().with_line_width(WIDTH),
                        },
                        transform,
                    );

                    let points = points
                        .iter()
                        .map(|proto::Point { x, y }| point![*x as f32, *y as f32])
                        .collect::<Vec<Point<f32>>>();

                    PathBundle {
                        layer: InLayer(layer),
                        shape_lyon,
                        shape_rapier: RapierShapeBundle {
                            collider: ColliderBundle {
                                shape: ColliderShape::polyline(points, None),
                                flags: (ActiveEvents::INTERSECTION_EVENTS
                                    | ActiveEvents::CONTACT_EVENTS)
                                    .into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                })
                .collect::<Vec<PathBundle>>();

            // commands.spawn_batch(rects);

            // info!("{}", rects.len());
            // for mut r in rects {
            //     r.rect.visible.is_visible = true;
            //     r.rect.visible.is_transparent = true;
            //     info!(
            //         "{:?} {:?} {:?} {:?} {:?}",
            //         r.rect.path.0,
            //         r.rect.draw,
            //         r.rect.global_transform,
            //         r.rect.transform,
            //         r.rect.visible
            //     );
            //     commands.spawn_bundle(r).insert(GlobalTransform::default());
            //     // std::thread::sleep(std::time::Duration::from_millis(100));
            // }
            info!(
                "x min: {}, max: {}, y min: {}, max: {}",
                x_min, x_max, y_min, y_max
            );
            // commands.spawn_batch(rects);
            // commands.spawn_batch(polys);
            // let chunk_size = 100_000;
            // for (i, p) in paths.chunks(chunk_size).enumerate() {
            //     commands.spawn_batch(p.to_vec());
            //     info!("{}", chunk_size * i);
            //     std::thread::sleep(std::time::Duration::from_secs(1));
            // }

            // std::thread::sleep(std::time::Duration::from_millis(10000));

            commands.spawn_batch(
                rects
                    .into_iter()
                    .rev()
                    .take(30_000)
                    .collect::<Vec<RectBundle>>(),
            );

            commands.spawn_batch(
                polys
                    .into_iter()
                    .into_iter()
                    .rev()
                    .take(30_000)
                    .collect::<Vec<PolygonBundle>>(),
            );

            commands.spawn_batch(
                paths
                    .into_iter()
                    .rev()
                    .take(30_000)
                    .collect::<Vec<PathBundle>>(),
            );

            // info!("Done {:?}", layer);
        }
    }

    let (mut transform, _) = query.single_mut().unwrap();
    // let s = (x_max - x_min).abs().max((y_max - y_min).abs()) as f32 / 2.0;
    info!(
        "x min {} max {}   y min {} max {}",
        x_min, x_max, y_min, y_max
    );

    let sx = (x_max - x_min).abs();
    let sy = (y_max - y_min).abs();

    let s = sx.max(sy) as f32 / 1000.0;

    transform.scale.x = s;
    transform.scale.y = s;
    transform.translation.x = 1920.0;
    transform.translation.y = 1080.0;

    // info!("Scale: {}", proj.scale);
}

fn read_lib_gds_write_proto() -> LayoutResult<()> {
    let gds = gds::gds21::GdsLibrary::load("./dff1_lib.golden.gds").unwrap();

    // Convert to Layout21::Raw
    let lib = gds::GdsImporter::import(&gds, None)?;
    assert_eq!(lib.name, "dff1_lib");
    assert_eq!(lib.cells.len(), 1);

    // Get the first (and only) cell
    let cell = lib.cells.first().unwrap().clone();
    let cell = cell.read()?;
    assert_eq!(cell.name, "dff1");

    // Convert to ProtoBuf
    let p = ProtoExporter::export(&lib)?;
    assert_eq!(p.domain, "dff1_lib");

    proto::save(&p, "dff1_lib.proto").unwrap();

    // And compare against the golden version
    let p2 = proto::open("./dff1_lib.golden.vlsir.bin").unwrap();
    assert_eq!(p, p2);

    Ok(())
}

#[test]
fn make_oscibear_proto() -> LayoutResult<()> {
    let gds = gds::gds21::GdsLibrary::load("./user_analog_project_wrapper.gds").unwrap();

    // Convert to Layout21::Raw
    let lib = gds::GdsImporter::import(&gds, None)?;
    info!("{}", lib.name);
    info!("{}", lib.cells.len());

    // // Get the first (and only) cell
    // let cell = lib.cells.first().unwrap().clone();
    // let cell = cell.read()?;
    // assert_eq!(cell.name, "dff1");

    // // Convert to ProtoBuf
    let p = ProtoExporter::export(&lib)?;
    info!("{}", p.domain);

    proto::save(&p, "oscibear.proto").unwrap();

    // // And compare against the golden version
    // let p2 = proto::open("./dff1_lib.golden.vlsir.bin").unwrap();
    // assert_eq!(p, p2);

    Ok(())
}
