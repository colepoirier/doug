use crate::shapes::{Path, PathBundle, Poly, PolyBundle, Rect, RectBundle, ShapeBundle};
use crate::{
    InLayer, LayerColors, LoadCompleteEvent, LoadProtoEvent, Nom, ViewportDimensions, ALPHA, WIDTH,
};

use bevy::prelude::*;
// use bevy::render::camera::OrthographicProjection;
use bevy_prototype_lyon::prelude::{
    shapes, DrawMode, FillMode, FillOptions, GeometryBuilder, StrokeMode, StrokeOptions,
};

use layout21::protos::{Cell, LayerShapes};
use layout21::raw::proto::proto;

use std::slice::Iter;

pub fn get_shapes(cell: &Cell) -> Iter<LayerShapes> {
    cell.layout.as_ref().unwrap().shapes.iter()
}

pub struct ImportRectEvent {
    pub rect: proto::Rectangle,
    pub layer: u16,
    pub color: Color,
}

pub struct ImportPolyEvent {
    pub poly: proto::Polygon,
    pub layer: u16,
    pub color: Color,
}

pub struct ImportPathEvent {
    pub path: proto::Path,
    pub layer: u16,
    pub color: Color,
}

pub fn load_proto_lib_system(
    mut load_proto_event_reader: EventReader<LoadProtoEvent>,
    mut layer_colors: ResMut<LayerColors>,
    mut viewport: ResMut<ViewportDimensions>,
    mut load_complete_event_writer: EventWriter<LoadCompleteEvent>,
    mut import_rect_event_writer: EventWriter<ImportRectEvent>,
    mut import_poly_event_writer: EventWriter<ImportPolyEvent>,
    mut import_path_event_writer: EventWriter<ImportPathEvent>,
) {
    for LoadProtoEvent { lib } in load_proto_event_reader.iter() {
        let t = std::time::Instant::now();
        let plib: proto::Library = proto::open(lib).unwrap();

        let d = t.elapsed();
        info!("File open task duration {:?}", d);

        info!("{:?} {}", plib.units(), plib.units);

        // let cell = plib.cells.iter().nth(770).unwrap();
        let cell = plib.cells.iter().nth(0).unwrap();

        let len = get_shapes(cell)
            .map(|s| s.rectangles.len() + s.polygons.len() + s.paths.len())
            .sum::<usize>();

        info!("{:?} {}", cell.name, len);

        for layer_shapes in cell.layout.as_ref().unwrap().shapes.iter() {
            let layer = layer_shapes.layer.as_ref().unwrap().number as u16;
            let color = layer_colors.get_color();

            for rect in layer_shapes.rectangles.iter() {
                let proto::Rectangle {
                    lower_left,
                    width,
                    height,
                    ..
                } = rect;
                let proto::Point { x, y } = lower_left.as_ref().unwrap();

                viewport.update(&ViewportDimensions {
                    x_min: *x,
                    x_max: x + width,
                    y_min: *y,
                    y_max: y + height,
                });

                import_rect_event_writer.send(ImportRectEvent {
                    rect: rect.clone(),
                    layer,
                    color,
                });
            }

            for poly in layer_shapes.polygons.iter() {
                let proto::Polygon { vertices, .. } = poly;

                viewport.update(&vertices.iter().fold(
                    ViewportDimensions::default(),
                    |mut vd, p: &proto::Point| {
                        let proto::Point { x, y } = p;
                        vd.x_min = vd.x_min.min(*x);
                        vd.x_max = vd.x_max.max(*x);
                        vd.y_min = vd.y_min.min(*y);
                        vd.y_max = vd.y_max.min(*y);
                        vd
                    },
                ));
                import_poly_event_writer.send(ImportPolyEvent {
                    poly: poly.clone(),
                    layer,
                    color,
                });
            }

            for path in layer_shapes.paths.iter() {
                let proto::Path { points, .. } = path;

                viewport.update(&points.iter().fold(
                    ViewportDimensions::default(),
                    |mut vd, p: &proto::Point| {
                        let proto::Point { x, y } = p;
                        vd.x_min = vd.x_min.min(*x);
                        vd.x_max = vd.x_max.max(*x);
                        vd.y_min = vd.y_min.min(*y);
                        vd.y_max = vd.y_max.min(*y);
                        vd
                    },
                ));

                import_path_event_writer.send(ImportPathEvent {
                    path: path.clone(),
                    layer,
                    color,
                });
            }
        }

        let d = t.elapsed();
        info!("{:?}", d);

        info!("viewport {:?}", viewport);

        load_complete_event_writer.send(LoadCompleteEvent);
    }
}

pub fn import_rect_system(
    mut commands: Commands,
    mut import_rect_event_reader: EventReader<ImportRectEvent>,
    // mut rtree_shape_collect_event_writer: EventWriter<RTreeShapeImportEvent>
) {
    for ImportRectEvent { rect, layer, color } in import_rect_event_reader.iter() {
        let proto::Rectangle {
            net,
            lower_left,
            width,
            height,
        } = rect;

        let proto::Point { x, y } = lower_left.as_ref().unwrap();

        let rect = shapes::Rectangle {
            origin: shapes::RectangleOrigin::BottomLeft,
            extents: (*width as f32, *height as f32).into(),
        };

        let transform = Transform::from_translation(Vec3::new(*x as f32, *y as f32, *layer as f32));

        let shape_lyon = GeometryBuilder::build_as(
            &rect,
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
            name: Nom(net.to_string()),
            shape_lyon,
            layer: InLayer(*layer),
        };

        commands.spawn_bundle(RectBundle {
            rect: Rect {
                width: *width as u32,
                height: *height as u32,
                origin: (*x as i32, *y as i32).into(),
            },
            shape,
        });
    }
}

pub fn import_poly_system(
    mut commands: Commands,
    mut import_poly_event_reader: EventReader<ImportPolyEvent>,
) {
    for ImportPolyEvent { poly, layer, color } in import_poly_event_reader.iter() {
        let proto::Polygon { vertices, net } = poly;

        let poly = shapes::Polygon {
            points: vertices
                .iter()
                .map(|proto::Point { x, y }| Vec2::new(*x as f32, *y as f32))
                .collect::<Vec<Vec2>>(),
            closed: true,
        };

        let transform = Transform::from_translation(Vec3::new(0.0, 0.0, *layer as f32));

        let shape_lyon = GeometryBuilder::build_as(
            &poly,
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
            name: Nom(net.to_string()),
            layer: InLayer(*layer),
            shape_lyon,
        };

        commands.spawn_bundle(PolyBundle {
            poly: Poly {
                verts: vertices.clone(),
            },
            shape,
        });
    }
}

pub fn import_path_system(
    mut commands: Commands,
    mut import_path_event_reader: EventReader<ImportPathEvent>,
) {
    for ImportPathEvent { path, layer, color } in import_path_event_reader.iter() {
        let proto::Path { points, width, net } = path;

        let path = shapes::Polygon {
            points: points
                .iter()
                .map(|proto::Point { x, y }| Vec2::new(*x as f32, *y as f32))
                .collect::<Vec<Vec2>>(),
            closed: false,
        };

        let transform = Transform::from_translation(Vec3::new(0.0, 0.0, *layer as f32));

        let shape_lyon = GeometryBuilder::build_as(
            &path,
            DrawMode::Outlined {
                fill_mode: FillMode {
                    color: *color.clone().set_a(ALPHA),
                    options: FillOptions::default(),
                },
                outline_mode: StrokeMode {
                    options: StrokeOptions::default().with_line_width(*width as f32),
                    color: *color,
                },
            },
            transform,
        );

        let shape = ShapeBundle {
            name: Nom(net.to_string()),
            layer: InLayer(*layer),
            shape_lyon,
        };

        commands.spawn_bundle(PathBundle {
            path: Path {
                verts: points.clone(),
            },
            shape,
        });
    }
}

#[cfg(test)]
mod tests {
    use layout21::protos::save;
    use layout21::raw::{gds, proto::ProtoExporter, LayoutResult};

    #[test]
    fn make_oscibear_proto() -> LayoutResult<()> {
        let gds = gds::gds21::GdsLibrary::load("./user_analog_project_wrapper.gds").unwrap();

        // Convert to Layout21::Raw
        let lib = gds::GdsImporter::import(&gds, None)?;
        println!("{}", lib.name);
        println!("{}", lib.cells.len());

        // // Convert to ProtoBuf
        let p = ProtoExporter::export(&lib)?;
        println!("{}", p.domain);

        save(&p, "oscibear.proto").unwrap();
        Ok(())
    }
}
