use crate::LayerColors;
use crate::{InLayer, LayerBundle, LayerColor, LayerNum, WIDTH};
use bevy::prelude::*;
use bevy::render::camera::{ActiveCamera, OrthographicProjection};
use bevy_inspector_egui::Inspectable;
use bevy_prototype_lyon::entity::ShapeBundle;
use layout21::raw::gds;
use layout21::raw::proto::proto;
use layout21::raw::proto::ProtoExporter;
use layout21::raw::LayoutResult;

use crate::Paths;

use crate::LoadCompleteEvent;
use crate::ALPHA;

use std::collections::HashMap;

use bevy_prototype_lyon::prelude::{
    DrawMode, FillMode, FillOptions, GeometryBuilder, StrokeMode, StrokeOptions,
};
use bevy_prototype_lyon::shapes;
use bevy_prototype_lyon::shapes::RectangleOrigin;

#[derive(Bundle, Clone)]
pub struct Rect {
    pub layer: InLayer,
    #[bundle]
    pub rect: ShapeBundle,
}

#[derive(Bundle, Clone)]
pub struct Poly {
    pub layer: InLayer,
    #[bundle]
    pub poly: ShapeBundle,
}

#[derive(Bundle, Inspectable, Debug, Default, Clone)]
pub struct Path {
    pub layer: InLayer,
    #[bundle]
    pub path: ShapeBundle,
}

pub fn test_load_proto_lib(
    commands: &mut Commands,
    layer_colors: &mut ResMut<LayerColors>,
    paths: &mut ResMut<Paths>,
    load_complete_event_writer: &mut EventWriter<LoadCompleteEvent>,
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
    // println!("{}", layers.len());

    layers.dedup();
    // println!("{}", layers.len());
    // println!("{:?}", layers);

    let layers = layers
        .iter()
        .map(|&num| (num, layer_colors.get_color()))
        .collect::<Vec<(i64, Color)>>();

    // println!("{:?}", layers);

    let mut layer_map = HashMap::<u16, (Entity, Color)>::new();

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

    let mut x_min: i64 = 0;
    let mut x_max: i64 = 0;
    let mut y_min: i64 = 0;
    let mut y_max: i64 = 0;

    plib.cells.iter().enumerate().for_each(|(i, cell)| {
        let mut rects = 0;
        let mut polys = 0;
        let mut paths = 0;
        for layer_shapes in cell.layout.as_ref().unwrap().shapes.iter()
        // .rev()
        // .take(10)
        {
            // println!("{:?}", cell);

            rects += layer_shapes.rectangles.len();
            polys += layer_shapes.polygons.len();
            paths += layer_shapes.paths.len();

            for proto::Rectangle {
                width,
                height,
                lower_left,
                ..
            } in layer_shapes.rectangles.iter()
            {
                let proto::Point { x, y } = lower_left.as_ref().unwrap();
                let width = *width;
                let height = *height;
                let x = *x;
                let y = *y;
                x_min = std::cmp::min(x_min, x);
                x_max = std::cmp::max(x_max, x + width);
                y_min = std::cmp::min(y_min, y);
                y_max = std::cmp::max(y_max, y + height);
            }
        }
        if paths > 1 {
            println!(
                "index: {}, name: {} rects: {}, polys: {}, paths: {}",
                i,
                cell.name,
                rects,
                polys,
                paths,
                // cell.layout.as_ref().unwrap().instances
            );
            println!(
                "x min: {}, max: {}, y min: {}, max: {}",
                x_min, x_max, y_min, y_max
            );
        }
        if i == 960 {
            let (mut transform, mut proj) = query.single_mut();
            proj.scale = (y_max - y_min) as f32 / 1.75;
            transform.translation.x = (x_max - x_min) as f32 / 10.0;
            transform.translation.y = (y_max - y_min) as f32 / 2.0;
        }
    });

    println!(
        "x min: {}, max: {}, y min: {}, max: {}",
        x_min, x_max, y_min, y_max
    );

    // // return early to test the min max
    // return;

    let mut x_min: i64 = 0;
    let mut x_max: i64 = 0;
    let mut y_min: i64 = 0;
    let mut y_max: i64 = 0;

    for cell in plib.cells.iter().nth(960) {
        let len = cell
            .layout
            .as_ref()
            .unwrap()
            .shapes
            .iter()
            .map(|s| s.paths.len())
            .collect::<Vec<usize>>();

        let len: usize = len.into_iter().sum();

        println!("{:?} {}", cell.name, len);
        // break;
        for layer_shapes in cell.layout.as_ref().unwrap().shapes.iter()
        // .rev()
        // .take(10)
        {
            // println!("{:?}", cell);
            let layer = layer_shapes.layer.as_ref().unwrap().number as u16;
            let (layer_entity, color) = layer_map.get(&layer).unwrap();
            let color = *color;
            let layer_entity = *layer_entity;
            let rects = layer_shapes
                .rectangles
                .iter()
                .map(
                    |proto::Rectangle {
                         width,
                         height,
                         lower_left,
                         ..
                     }| {
                        let proto::Point { x, y } = lower_left.as_ref().unwrap();
                        let width = *width;
                        let height = *height;
                        let x = *x;
                        let y = *y;
                        x_min = std::cmp::min(x_min, x);
                        x_max = std::cmp::max(x_max, x + width);
                        y_min = std::cmp::min(y_min, y);
                        y_max = std::cmp::max(y_max, y + height);

                        let rect = shapes::Rectangle {
                            origin: RectangleOrigin::BottomLeft,
                            extents: Vec2::new(width as f32, height as f32),
                        };
                        // println!("{:?}", rect);
                        let transform = Transform::from_translation(Vec3::new(
                            x as f32,
                            y as f32,
                            layer as f32,
                        ));
                        // println!("{:?}", transform);
                        let rect = Rect {
                            rect: GeometryBuilder::build_as(
                                &rect,
                                DrawMode::Outlined {
                                    fill_mode: FillMode {
                                        color: *color.clone().set_a(ALPHA),
                                        options: FillOptions::default(),
                                    },
                                    outline_mode: StrokeMode {
                                        options: StrokeOptions::default().with_line_width(WIDTH),
                                        color: color,
                                    },
                                },
                                transform,
                            ),
                            layer: InLayer(layer_entity),
                        };
                        // println!(
                        //     "{:?}, {:?}",
                        //     rect.rect.transform, rect.rect.global_transform
                        // );
                        rect
                    },
                )
                .collect::<Vec<Rect>>();

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
                    let vertices = vertices
                        .iter()
                        .map(|proto::Point { x, y }| Vec2::new(*x as f32, *y as f32))
                        .collect::<Vec<Vec2>>();

                    let poly = shapes::Polygon {
                        points: vertices,
                        closed: true,
                    };
                    // println!("{:?}", poly);

                    let transform = Transform::from_translation(Vec3::new(0.0, 0.0, layer as f32));

                    Poly {
                        poly: GeometryBuilder::build_as(
                            &poly,
                            DrawMode::Outlined {
                                fill_mode: FillMode {
                                    color: *color.clone().set_a(ALPHA),
                                    options: FillOptions::default(),
                                },
                                outline_mode: StrokeMode {
                                    options: StrokeOptions::default().with_line_width(WIDTH),
                                    color: color,
                                },
                            },
                            transform,
                        ),
                        layer: InLayer(layer_entity),
                    }
                })
                .collect::<Vec<Poly>>();

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

                    let points = points
                        .iter()
                        .map(|proto::Point { x, y }| Vec2::new(*x as f32, *y as f32))
                        .collect::<Vec<Vec2>>();

                    let path = shapes::Polygon {
                        points: points,
                        closed: false,
                    };

                    // println!("{:?}", path);

                    let transform = Transform::from_translation(Vec3::new(0.0, 0.0, layer as f32));

                    Path {
                        path: GeometryBuilder::build_as(
                            &path,
                            DrawMode::Outlined {
                                fill_mode: FillMode {
                                    color: *color.clone().set_a(ALPHA),
                                    options: FillOptions::default(),
                                },
                                outline_mode: StrokeMode {
                                    options: StrokeOptions::default().with_line_width(WIDTH),
                                    color: color,
                                },
                            },
                            transform,
                        ),
                        layer: InLayer(layer_entity),
                    }
                })
                .collect::<Vec<Path>>();

            // commands.spawn_batch(rects);

            // println!("{}", rects.len());
            // for mut r in rects {
            //     r.rect.visible.is_visible = true;
            //     r.rect.visible.is_transparent = true;
            //     println!(
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
            println!(
                "x min: {}, max: {}, y min: {}, max: {}",
                x_min, x_max, y_min, y_max
            );
            // commands.spawn_batch(rects);
            // commands.spawn_batch(polys);
            // let chunk_size = 100_000;
            // for (i, p) in paths.chunks(chunk_size).enumerate() {
            //     commands.spawn_batch(p.to_vec());
            //     println!("{}", chunk_size * i);
            //     std::thread::sleep(std::time::Duration::from_secs(1));
            // }

            // std::thread::sleep(std::time::Duration::from_millis(10000));
            for r in rects
                .into_iter()
                .rev()
                .take(30_000)
                .collect::<Vec<Rect>>()
                .chunks(10_000)
            {
                commands.spawn_batch(r.to_vec());
            }
            for p in polys
                .into_iter()
                .rev()
                .take(30_000)
                .collect::<Vec<Poly>>()
                .chunks(10_000)
            {
                commands.spawn_batch(p.to_vec());
            }
            for p in paths
                .into_iter()
                .rev()
                .take(30_000)
                .collect::<Vec<Path>>()
                .chunks(10_000)
            {
                commands.spawn_batch(p.to_vec());
            }
            // println!("Done {:?}", layer);
        }
    }
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
    println!("{}", lib.name);
    println!("{}", lib.cells.len());

    // // Get the first (and only) cell
    // let cell = lib.cells.first().unwrap().clone();
    // let cell = cell.read()?;
    // assert_eq!(cell.name, "dff1");

    // // Convert to ProtoBuf
    let p = ProtoExporter::export(&lib)?;
    println!("{}", p.domain);

    proto::save(&p, "oscibear.proto").unwrap();

    // // And compare against the golden version
    // let p2 = proto::open("./dff1_lib.golden.vlsir.bin").unwrap();
    // assert_eq!(p, p2);

    Ok(())
}
