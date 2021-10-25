use crate::geom::{InLayer, LayerBundle, LayerColor, LayerNum};
use crate::LayerColors;
use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use layout21raw::gds;
use layout21raw::proto::proto;
use layout21raw::proto::ProtoExporter;
use layout21raw::LayoutResult;

use crate::geom::ALPHA;

use std::cmp::{max, min};

use std::collections::HashMap;

use bevy_prototype_lyon::path::PathBuilder;
use bevy_prototype_lyon::prelude::{
    DrawMode, FillMode, FillOptions, Geometry, GeometryBuilder, StrokeMode, StrokeOptions,
};
use bevy_prototype_lyon::shapes;
use bevy_prototype_lyon::shapes::RectangleOrigin;

#[derive(Bundle)]
pub struct Rect {
    pub layer: InLayer,
    #[bundle]
    pub rect: ShapeBundle,
}

#[derive(Bundle)]
pub struct Poly {
    pub layer: InLayer,
    #[bundle]
    pub poly: ShapeBundle,
}

#[derive(Bundle)]
pub struct Path {
    pub layer: InLayer,
    #[bundle]
    pub path: ShapeBundle,
}

pub fn test_load_proto_lib(commands: &mut Commands, layer_colors: &mut ResMut<LayerColors>) {
    let plib: proto::Library = proto::open("./dff1_lib.proto").unwrap();

    let mut layers = plib.cells[0]
        .layout
        .as_ref()
        .unwrap()
        .shapes
        .iter()
        .map(|s| s.layer.as_ref().unwrap().number)
        .collect::<Vec<i64>>();

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

    plib.cells[0]
        .layout
        .as_ref()
        .unwrap()
        .shapes
        .iter()
        // .rev()
        // .take(10)
        .for_each(|layer_shapes| {
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
                        Rect {
                            rect: GeometryBuilder::build_as(
                                &rect,
                                DrawMode::Outlined {
                                    fill_mode: FillMode {
                                        color: *color.clone().set_a(0.25),
                                        options: FillOptions::default(),
                                    },
                                    outline_mode: StrokeMode {
                                        options: StrokeOptions::default().with_line_width(10.0),
                                        color: color,
                                    },
                                },
                                transform,
                            ),
                            layer: InLayer(layer_entity),
                        }
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

                    let transform = Transform::from_translation(Vec3::new(0.0, 0.0, layer as f32));

                    Poly {
                        poly: GeometryBuilder::build_as(
                            &poly,
                            DrawMode::Outlined {
                                fill_mode: FillMode {
                                    color: *color.clone().set_a(0.25),
                                    options: FillOptions::default(),
                                },
                                outline_mode: StrokeMode {
                                    options: StrokeOptions::default().with_line_width(10.0),
                                    color: color,
                                },
                            },
                            transform,
                        ),
                        layer: InLayer(layer_entity),
                    }
                })
                .collect::<Vec<Poly>>();

            // let paths = layer_shapes
            //     .paths
            //     .iter()
            //     .map(|proto::Path { points, width, .. }| {
            //         for p in points {
            //             x_min = std::cmp::min(x_min, p.x);
            //             x_max = std::cmp::max(x_max, p.x);
            //             y_min = std::cmp::min(y_min, p.y);
            //             y_max = std::cmp::max(y_max, p.y);
            //         }
            //         let points = points
            //             .iter()
            //             .map(|proto::Point { x, y }| Vec2::new(*x as f32, *y as f32))
            //             .collect::<Vec<Vec2>>();
            //         let mut path = PathBuilder::new();
            //         path.move_to(points[0]);

            //         (&points[1..]).iter().for_each(|p| {
            //             path.line_to(*p);
            //         });
            //         path.close();
            //         let path = path.build();

            //         let transform = Transform::from_translation(Vec3::new(0.0, 0.0, layer as f32));

            //         Path {
            //             path: GeometryBuilder::build_as(
            //                 &path,
            //                 // ShapeColors::outlined(*color.clone().set_a(ALPHA), color),
            //                 DrawMode::Outlined {
            //                     fill_mode: FillMode {
            //                         color: *color.clone().set_a(0.25),
            //                         options: FillOptions::default(),
            //                     },
            //                     outline_mode: StrokeMode {
            //                         options: StrokeOptions::default().with_line_width(10.0),
            //                         color: color,
            //                     },
            //                 },
            //                 transform,
            //             ),
            //             layer: InLayer(layer_entity),
            //         }
            //     })
            //     .collect::<Vec<Path>>();

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

            commands.spawn_batch(rects);
            commands.spawn_batch(polys);
            // commands.spawn_batch(paths);
            // println!("Done {:?}", layer);
        });
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
