use crate::shapes::{Path, PathBundle, Poly, PolyBundle, Rect, RectBundle, ShapeBundle};
use crate::{InLayer, Nom, UpdateViewportEvent, ViewportDimensions, ALPHA, WIDTH};

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{
    shapes, DrawMode, FillMode, FillOptions, GeometryBuilder, StrokeMode, StrokeOptions,
};

use layout21::protos::{Cell, LayerShapes};
use layout21::raw::proto::proto;

use std::slice::Iter;

#[derive(Component, Debug)]
pub struct LayerColors {
    colors: std::iter::Cycle<std::vec::IntoIter<Color>>,
}

impl Default for LayerColors {
    fn default() -> Self {
        Self {
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

pub fn get_shapes(cell: &Cell) -> Iter<LayerShapes> {
    cell.layout.as_ref().unwrap().shapes.iter()
}

pub struct Layout21ImportPlugin;

impl Plugin for Layout21ImportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LayerColors::default())
            .add_event::<LoadProtoEvent>()
            .add_event::<LoadCompleteEvent>()
            .add_event::<UpdateViewportEvent>()
            .add_event::<ImportRectEvent>()
            .add_event::<ImportPolyEvent>()
            .add_event::<ImportPathEvent>()
            .add_stage("import", SystemStage::parallel())
            .add_stage_after("import", "update_viewport", SystemStage::parallel())
            .add_startup_system(send_import_event_system)
            .add_system(load_proto_lib_system)
            .add_system(load_complete_system)
            .add_system(import_path_system)
            .add_system(import_rect_system)
            .add_system(import_poly_system);
    }
}

#[derive(Debug, Default, Clone)]
pub struct LoadProtoEvent {
    lib: String,
}
#[derive(Debug, Default, Clone, Copy)]
pub struct LoadCompleteEvent {
    pub viewport_dimensions: ViewportDimensions,
}

fn send_import_event_system(mut my_events: EventWriter<LoadProtoEvent>) {
    my_events.send(LoadProtoEvent {
        lib: "./models/dff1_lib.proto".into(),
        // "./models/oscibear.proto",
    });
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

pub fn load_complete_system(
    mut load_complete_event_reader: EventReader<LoadCompleteEvent>,
    mut update_viewport_event_writer: EventWriter<UpdateViewportEvent>,
    mut viewport: ResMut<ViewportDimensions>,
) {
    for &LoadCompleteEvent {
        viewport_dimensions,
    } in load_complete_event_reader.iter()
    {
        *viewport = viewport_dimensions;
        update_viewport_event_writer.send(UpdateViewportEvent);
    }
}

pub fn load_proto_lib_system(
    mut layer_colors: ResMut<LayerColors>,
    mut load_proto_event_reader: EventReader<LoadProtoEvent>,
    mut load_complete_event_writer: EventWriter<LoadCompleteEvent>,
    mut import_rect_event_writer: EventWriter<ImportRectEvent>,
    mut import_poly_event_writer: EventWriter<ImportPolyEvent>,
    mut import_path_event_writer: EventWriter<ImportPathEvent>,
) {
    for LoadProtoEvent { lib } in load_proto_event_reader.iter() {
        let t = std::time::Instant::now();

        let mut viewport_dimensions = ViewportDimensions::default();

        let plib: proto::Library = proto::open(lib).unwrap();

        let d = t.elapsed();
        info!("Layout21 Proto import file open task duration {:?}", d);

        info!("{:?} {}", plib.units(), plib.units);

        // oscibear.proto
        // let cell = plib.cells.iter().nth(770).unwrap();
        // dff_lib.proto
        let cell = plib.cells.iter().nth(0).unwrap();

        let len = get_shapes(cell)
            .map(|s| s.rectangles.len() + s.polygons.len() + s.paths.len())
            .sum::<usize>();

        info!("Cell {:?}, len: {}", cell.name, len);

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

                viewport_dimensions = ViewportDimensions {
                    x_min: *x,
                    x_max: x + width,
                    y_min: *y,
                    y_max: y + height,
                };

                import_rect_event_writer.send(ImportRectEvent {
                    rect: rect.clone(),
                    layer,
                    color,
                });
            }

            for poly in layer_shapes.polygons.iter() {
                let proto::Polygon { vertices, .. } = poly;

                viewport_dimensions.update(
                    &(vertices.iter().fold(
                        ViewportDimensions::default(),
                        |mut vd, p: &proto::Point| {
                            let proto::Point { x, y } = p;
                            vd.x_min = vd.x_min.min(*x);
                            vd.x_max = vd.x_max.max(*x);
                            vd.y_min = vd.y_min.min(*y);
                            vd.y_max = vd.y_max.min(*y);
                            vd
                        },
                    )),
                );
                import_poly_event_writer.send(ImportPolyEvent {
                    poly: poly.clone(),
                    layer,
                    color,
                });
            }

            for path in layer_shapes.paths.iter() {
                let proto::Path { points, .. } = path;

                viewport_dimensions.update(
                    &(points.iter().fold(
                        ViewportDimensions::default(),
                        |mut vd, p: &proto::Point| {
                            let proto::Point { x, y } = p;
                            vd.x_min = vd.x_min.min(*x);
                            vd.x_max = vd.x_max.max(*x);
                            vd.y_min = vd.y_min.min(*y);
                            vd.y_max = vd.y_max.min(*y);
                            vd
                        },
                    )),
                );

                import_path_event_writer.send(ImportPathEvent {
                    path: path.clone(),
                    layer,
                    color,
                });
            }
        }

        info!("load_proto_lib_system {:?}", viewport_dimensions);

        load_complete_event_writer.send(LoadCompleteEvent {
            viewport_dimensions,
        });

        let d = t.elapsed();
        info!("Total Layout21 Proto import duration {:?}", d);
    }
}

pub fn import_rect_system(
    mut commands: Commands,
    mut import_rect_event_reader: EventReader<ImportRectEvent>,
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
