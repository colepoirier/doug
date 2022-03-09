use crate::shapes::{Path, PathBundle, Poly, PolyBundle, Rect, RectBundle, ShapeBundle};
use crate::{InLayer, Nom, UpdateViewportEvent, ViewportDimensions, ALPHA, WIDTH};

use bevy::prelude::*;
use bevy_prototype_lyon::entity;
use bevy_prototype_lyon::prelude::{
    shapes, DrawMode, FillMode, FillOptions, GeometryBuilder, StrokeMode, StrokeOptions,
};

use derive_more::{Deref, DerefMut};

use vlsir::{raw, Cell, LayerShapes, Library};

use std::slice::Iter;

#[derive(Component, Debug)]
pub struct LayerColors {
    colors: std::iter::Cycle<std::vec::IntoIter<Color>>,
}

impl Default for LayerColors {
    fn default() -> Self {
        Self {
            // IBM Design Language Color Library - Color blind safe palette
            // https://ibm-design-language.eu-de.mybluemix.net/design/language/resources/color-library/
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

pub fn get_shapes(cell: &Cell) -> Iter<LayerShapes> {
    cell.layout.as_ref().unwrap().shapes.iter()
}

#[derive(Debug, Default)]
pub struct ProtoGdsLib {
    pub lib: Option<Library>,
    pub cells: Vec<String>,
    pub selected: usize,
}

pub struct Layout21ImportPlugin;

impl Plugin for Layout21ImportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LayerColors::default())
            .insert_resource(ProtoGdsLib::default())
            .add_event::<LoadLibEvent>()
            .add_event::<LoadLibCompleteEvent>()
            .add_event::<LoadCellEvent>()
            .add_event::<LoadCellCompleteEvent>()
            .add_event::<UpdateViewportEvent>()
            .add_event::<ImportRectEvent>()
            .add_event::<ImportPolyEvent>()
            .add_event::<ImportPathEvent>()
            .add_stage("reset_world", SystemStage::parallel())
            .add_stage_after("reset_world", "import", SystemStage::parallel())
            // .add_startup_system(send_import_event_system)
            .add_system_to_stage("reset_world", despawn_all_shapes_system)
            .add_system_to_stage("import", load_proto_lib_system)
            .add_system_to_stage("import", load_proto_cell_system)
            .add_system_to_stage("import", load_cell_complete_system)
            .add_system_to_stage("import", import_path_system)
            .add_system_to_stage("import", import_rect_system)
            .add_system_to_stage("import", import_poly_system);
    }
}

#[derive(Debug, Default, Clone)]
pub struct LoadLibEvent {
    pub lib: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LoadLibCompleteEvent;

#[derive(Debug, Default, Clone, Copy, Deref, DerefMut)]
pub struct LoadCellEvent(pub usize);

#[derive(Debug, Default, Clone, Copy)]
pub struct LoadCellCompleteEvent {
    pub viewport_dimensions: ViewportDimensions,
}

pub struct ImportRectEvent {
    pub rect: raw::Rectangle,
    pub layer: u16,
    pub color: Color,
}

pub struct ImportPolyEvent {
    pub poly: raw::Polygon,
    pub layer: u16,
    pub color: Color,
}

pub struct ImportPathEvent {
    pub path: raw::Path,
    pub layer: u16,
    pub color: Color,
}

pub fn load_cell_complete_system(
    mut load_complete_event_reader: EventReader<LoadCellCompleteEvent>,
    mut update_viewport_event_writer: EventWriter<UpdateViewportEvent>,
    mut viewport: ResMut<ViewportDimensions>,
) {
    for &LoadCellCompleteEvent {
        viewport_dimensions,
    } in load_complete_event_reader.iter()
    {
        *viewport = viewport_dimensions;
        update_viewport_event_writer.send(UpdateViewportEvent);
    }
}

pub fn load_proto_lib_system(
    mut proto_gds_lib: ResMut<ProtoGdsLib>,
    mut load_proto_event_reader: EventReader<LoadLibEvent>,
    mut load_lib_complete_event_writer: EventWriter<LoadLibCompleteEvent>,
    mut load_cell_event_writer: EventWriter<LoadCellEvent>,
) {
    for LoadLibEvent { lib } in load_proto_event_reader.iter() {
        let t = std::time::Instant::now();

        let lib: Library = vlsir::open(lib).unwrap();

        let cells = lib
            .cells
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<String>>();

        let longest = cells.iter().max().unwrap();

        info!(
            "Longest cell name: {} chars, {}",
            longest.chars().count(),
            longest
        );

        let lib = Some(lib);

        *proto_gds_lib = ProtoGdsLib {
            lib,
            cells,
            selected: 0,
        };

        let d = t.elapsed();
        info!("Layout21 Proto import file open task duration {:?}", d);

        load_lib_complete_event_writer.send(LoadLibCompleteEvent);
        load_cell_event_writer.send(LoadCellEvent(0));
    }
}

pub fn despawn_all_shapes_system(
    mut commands: Commands,
    query: Query<Entity, With<entity::Path>>,
    mut load_proto_event_reader: EventReader<LoadLibEvent>,
    mut load_cell_event_reader: EventReader<LoadCellEvent>,
) {
    for _ in load_proto_event_reader.iter() {
        for e in query.iter() {
            commands.entity(e).despawn();
        }
    }
    for _ in load_cell_event_reader.iter() {
        for e in query.iter() {
            commands.entity(e).despawn();
        }
    }
}

pub fn load_proto_cell_system(
    proto_gds_lib: Res<ProtoGdsLib>,
    mut layer_colors: ResMut<LayerColors>,
    mut load_cell_event_reader: EventReader<LoadCellEvent>,
    mut load_cell_complete_event_writer: EventWriter<LoadCellCompleteEvent>,
    mut import_rect_event_writer: EventWriter<ImportRectEvent>,
    mut import_poly_event_writer: EventWriter<ImportPolyEvent>,
    mut import_path_event_writer: EventWriter<ImportPathEvent>,
) {
    for &cell_idx in load_cell_event_reader.iter() {
        let t = std::time::Instant::now();

        let mut viewport_dimensions = ViewportDimensions::default();

        // oscibear.proto
        // let cell = plib.cells.iter().nth(770).unwrap();
        // dff_lib.proto
        // let cell = plib.cells.iter().nth(0).unwrap();

        let cell = &proto_gds_lib.lib.as_ref().unwrap().cells[*cell_idx];

        let num_shapes = get_shapes(cell)
            .map(|s| s.rectangles.len() + s.polygons.len() + s.paths.len())
            .sum::<usize>();

        info!("Cell {}, num shapes: {num_shapes}", cell.name);

        for layer_shapes in cell.layout.as_ref().unwrap().shapes.iter() {
            let layer = layer_shapes.layer.as_ref().unwrap().number as u16;
            let color = layer_colors.get_color();

            for rect in layer_shapes.rectangles.iter() {
                let raw::Rectangle {
                    lower_left,
                    width,
                    height,
                    ..
                } = rect;
                let raw::Point { x, y } = lower_left.as_ref().unwrap();

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
                let raw::Polygon { vertices, .. } = poly;

                viewport_dimensions.update(
                    &(vertices.iter().fold(
                        ViewportDimensions::default(),
                        |mut vd, p: &raw::Point| {
                            let raw::Point { x, y } = p;
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
                let raw::Path { points, .. } = path;

                viewport_dimensions.update(
                    &(points.iter().fold(
                        ViewportDimensions::default(),
                        |mut vd, p: &raw::Point| {
                            let raw::Point { x, y } = p;
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

        load_cell_complete_event_writer.send(LoadCellCompleteEvent {
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
        let raw::Rectangle {
            net,
            lower_left,
            width,
            height,
        } = rect;

        let raw::Point { x, y } = lower_left.as_ref().unwrap();

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
        let raw::Polygon { vertices, net } = poly;

        let poly = shapes::Polygon {
            points: vertices
                .iter()
                .map(|raw::Point { x, y }| Vec2::new(*x as f32, *y as f32))
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
        let raw::Path { points, width, net } = path;

        let path = shapes::Polygon {
            points: points
                .iter()
                .map(|raw::Point { x, y }| Vec2::new(*x as f32, *y as f32))
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
