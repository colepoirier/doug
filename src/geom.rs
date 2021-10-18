use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use std::collections::HashMap;

use derive_more::{Deref, DerefMut};

// Set a default alpha-value for most shapes
pub const ALPHA: f32 = 0.25;

// pub enum ShapeType {
//   Rect,
//   Poly,
// }

// impl ShapeType {
//     pub fn as_str() -> &'static str {
//         match &self{
//             Rect => "RECT",
//             Poly =>"POLY",
//         }
//     }
// }

// struct PolyArgs  {
//   points: Vec<number>; // FIXME: real version can also be `PIXI.IPointData`
// };

// type RectArgs = {
//   x: number;
//   y: number;
//   width: number;
//   height: number;
// };
// type Point = {
//   x: number;
//   y: number;
// };

// pub fn spawn_shape_sytem() => {
//   if (tp === ShapeType.Rect) {
//     const rargs = args as RectArgs;
//     const { x, y, width, height } = rargs;
//     return new PIXI.Rectangle(x, y, width, height);
//   }
//   if (tp === ShapeType.Poly) {
//     const rargs = args as PolyArgs;
//     return new PIXI.Polygon(rargs.points);
//   }
//   throw new Error("Unknown shape type");
// };

// pub struct Layer {
//   name: Name,
//   shapes: Vec<Shape>,
//   paths: Vec<Path>,
//   num: u64,
//   color: u32,
//   ctr: PIXI.Container;
// }

//   constructor(args: any) {
//     this.shapes = [];
//     this.paths = [];
//     this.name = args.name;
//     this.num = args.num;
//     this.color = args.color || 0xffffff;
//     let ctr = new PIXI.Container();

//     ctr.interactive = true;
//     ctr.visible = true;
//     ctr.buttonMode = true;
//     this.ctr = ctr;
//   }
//   createShape(tp: ShapeType, args: any) {
//     const shape = new Shape(tp, this, args);
//     this.shapes.push(shape);
//     this.ctr.addChild(shape.gr);
//     return shape;
//   }
//   createRect(args: any) {
//     return this.createShape(ShapeType.Rect, args);
//   }
//   createPolygon(args: any) {
//     return this.createShape(ShapeType.Poly, args);
//   }
//   addPath(path: Path) {
//     return this.paths.push(path);
//   }
// }

// export class Shape {
//   layer: Layer; // Layer ref
//   tp: ShapeType; // Enumerated shape-types
//   raw: PIXI.Rectangle | PIXI.Polygon; // The raw PIXI shape object
//   args: any;
//   gr: PIXI.Graphics;

//   constructor(tp: ShapeType, layer: Layer, args: any) {
//     this.tp = tp;
//     this.layer = layer;
//     this.args = args;

//     let gr = new PIXI.Graphics();
//     this.gr = gr;
//     gr.interactive = true;
//     gr.visible = true;
//     gr.buttonMode = true;

//     layer.ctr.addChild(gr);
//     gr.beginFill(layer.color, ALPHA);
//     gr.lineStyle(10, layer.color, 1);

//     this.raw = newPixiShape(tp, args);
//     // this.raw.interactive = true; // FIXME: is this a thing?
//     gr.drawShape(this.raw);
//   }
// }

#[derive(Debug, Clone)]
pub struct Layer;

#[derive(Debug, Clone)]
pub struct InLayer(Entity);

#[derive(Debug, Clone)]
pub struct Path {
    layer: InLayer,
    width: u64,
}

#[derive(Debug, Default, Clone, Deref, DerefMut)]
pub struct LayerMap(pub HashMap<Name, Entity>);

impl Path {
    pub fn spawn(
        commands: &mut Commands,
        color_query: &Query<&Color, With<Layer>>,
        layer: Entity,
        width: f32,
        points: &[Vec2],
    ) {
        let color = color_query.get(layer).unwrap();

        let mut path = PathBuilder::new();
        path.move_to(points[0]);

        (&points[1..]).iter().for_each(|p| {
            path.line_to(*p);
        });
        path.close();
        let path = path.build();

        commands.spawn_bundle(GeometryBuilder::build_as(
            &path,
            ShapeColors::outlined(*color * ALPHA, *color),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default().with_line_width(width),
            },
            Transform::default(),
        ));
    }
}
