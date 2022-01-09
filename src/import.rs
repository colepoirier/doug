use std::fmt::Debug;

use crate::LayerColors;
use crate::{InLayer, LayerBundle, LayerColor, LayerNum, WIDTH};
use bevy::prelude::*;
use bevy::render::camera::OrthographicProjection;
use bevy_prototype_lyon::entity;
use bevy_prototype_lyon::prelude::{
    DrawMode, FillMode, FillOptions, GeometryBuilder, StrokeMode, StrokeOptions,
};
use bevy_prototype_lyon::shapes;
// use std::io::{BufWriter, Write};
use std::slice::Iter;

use layout21::protos::{Cell, LayerShapes};
use layout21::raw::gds;
use layout21::raw::proto::proto;
use layout21::raw::proto::ProtoExporter;
use layout21::raw::LayoutResult;

use crate::LoadCompleteEvent;
use crate::ALPHA;

use bevy::utils::HashMap;

#[derive(Component, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nom(String);

#[derive(Component, Default, Bundle)]
pub struct ShapeBundle {
    pub name: Nom,
    pub layer: InLayer,
    #[bundle]
    pub shape_lyon: entity::ShapeBundle,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Rect {
    pub width: u32,
    pub height: u32,
    pub origin: IVec2,
}

#[derive(Component, Default, Bundle)]
pub struct RectBundle {
    pub rect: Rect,
    #[bundle]
    pub shape: ShapeBundle,
}

#[derive(Component, Debug, Default)]
pub struct Poly;

#[derive(Component, Default, Bundle)]
pub struct PolyBundle {
    pub poly: Poly,
    #[bundle]
    pub shape: ShapeBundle,
}

#[derive(Component, Debug, Default)]
pub struct Path;

#[derive(Component, Default, Bundle)]
pub struct PathBundle {
    pub path: Path,
    #[bundle]
    pub shape: ShapeBundle,
}

pub fn get_shapes(cell: &Cell) -> Iter<LayerShapes> {
    cell.layout.as_ref().unwrap().shapes.iter()
}

pub fn load_proto_lib(
    commands: &mut Commands,
    layer_colors: &mut ResMut<LayerColors>,
    _load_complete_event_writer: &mut EventWriter<LoadCompleteEvent>,
    query: &mut Query<&mut Transform, With<OrthographicProjection>>,
) {
    let t = std::time::Instant::now();
    let plib: proto::Library = proto::open(
        "./models/dff1_lib.proto",
        // "./models/oscibear.proto",
    )
    .unwrap();

    let d = t.elapsed();
    info!("File open task duration {:?}", d);

    info!("{:?} {}", plib.units(), plib.units);

    // let cell = plib.cells.iter().nth(770).unwrap();
    let cell = plib.cells.iter().nth(0).unwrap();

    let len = get_shapes(cell)
        .map(|s| s.rectangles.len() + s.polygons.len() + s.paths.len())
        .collect::<Vec<usize>>();

    let len: usize = len.into_iter().sum();

    info!("{:?} {}", cell.name, len);

    for layer_shapes in cell.layout.as_ref().unwrap().shapes.iter() {
        let layer = layer_shapes.layer.as_ref().unwrap().number as u16;
        let color = layer_colors.get_color();
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
                    // info!(
                    //     "width: {} height: {} lower_left: {:?} net: {:?}",
                    //     width, height, lower_left, net
                    // );

                    // writer
                    //     .write(
                    //         &format!(
                    //             "lower_left: {:>9?} width: {:>9} height: {:>9} layer: {:>5?} net: {:>10?}\n",
                    //             lower_left,
                    //             width,
                    //             height,
                    //             layer_shapes.layer.as_ref().unwrap(),
                    //             net,
                    //         )
                    //         .as_bytes(),
                    //     )
                    //     .unwrap();

                    let proto::Point { x, y } = lower_left.as_ref().unwrap();
                    let ix = *x;
                    let iy = *y;
                    let iwidth = *width;
                    let iheight = *height;

                    let x = *x as f32;
                    let y = *y as f32;
                    let width = *width as f32;
                    let height = *height as f32;

                    let rect = shapes::Rectangle {
                        origin: shapes::RectangleOrigin::BottomLeft,
                        extents: (width, height).into(),
                    };

                    let transform =
                        Transform::from_translation(Vec3::new(x as f32, y as f32, layer as f32));

                    let shape_lyon = GeometryBuilder::build_as(
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
                    );

                    let shape = ShapeBundle {
                        name: Nom(net.to_string()),
                        shape_lyon,
                        layer: InLayer(layer),
                    };

                    RectBundle {
                        rect: Rect {
                            width: iwidth as u32,
                            height: iheight as u32,
                            origin: [ix as i32, iy as i32].into(),
                        },
                        shape,
                    }
                },
            )
            .collect::<Vec<RectBundle>>();

        let polys = layer_shapes
            .polygons
            .iter()
            .map(|proto::Polygon { vertices, net }| {
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
                );

                let shape = ShapeBundle {
                    name: Nom(net.to_string()),
                    layer: InLayer(layer),
                    shape_lyon,
                };

                PolyBundle { poly: Poly, shape }
            })
            .collect::<Vec<PolyBundle>>();

        let paths = layer_shapes
            .paths
            .iter()
            .map(|proto::Path { points, width, net }| {
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
                );

                let shape = ShapeBundle {
                    name: Nom(net.to_string()),
                    layer: InLayer(layer),
                    shape_lyon,
                };

                PathBundle { path: Path, shape }
            })
            .collect::<Vec<PathBundle>>();

        commands.spawn_batch(rects.into_iter());

        commands.spawn_batch(polys.into_iter().into_iter());

        commands.spawn_batch(paths.into_iter());
    }

    let mut camera_transform = query.single_mut();

    // info!(
    //     "[x] min: {}, max: {} [y] min: {}, max: {}",
    //     x_min, x_max, y_min, y_max
    // );

    // let sx = (x_max - x_min).abs();
    // let sy = (y_max - y_min).abs();

    // let s = sx.max(sy) as f32 / 1000.0;

    // camera_transform.scale.x = s;
    // camera_transform.scale.y = s;
}

#[test]
fn make_oscibear_proto() -> LayoutResult<()> {
    let gds = gds::gds21::GdsLibrary::load("./user_analog_project_wrapper.gds").unwrap();

    // Convert to Layout21::Raw
    let lib = gds::GdsImporter::import(&gds, None)?;
    info!("{}", lib.name);
    info!("{}", lib.cells.len());

    // // Convert to ProtoBuf
    let p = ProtoExporter::export(&lib)?;
    info!("{}", p.domain);

    proto::save(&p, "oscibear.proto").unwrap();
    Ok(())
}
