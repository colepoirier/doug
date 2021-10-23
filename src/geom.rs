use bevy::prelude::*;

use derive_more::{Deref, DerefMut};

// Set a default alpha-value for most shapes
pub const ALPHA: f32 = 0.25;
#[derive(Debug, Default, Clone, Copy)]
pub struct Layer;

#[derive(Debug, Default, Bundle, Clone, Copy)]
pub struct LayerBundle {
    pub layer: Layer,
    pub num: LayerNum,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct InLayer(pub Entity);

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deref, DerefMut)]
pub struct LayerNum(pub u16);

// #[derive(Debug, Default, Clone, Deref, DerefMut)]
// pub struct LayerMap(pub HashMap<Name, Entity>);

// impl Path {
//     pub fn spawn(
//         commands: &mut Commands,
//         color_query: &Query<(&LayerNum, &Color), With<Layer>>,
//         layer: LayerNum,
//         width: f32,
//         points: &[Vec2],
//     ) {
//         let color = color_query
//             .iter()
//             .filter(|(&layer_num, _)| layer == layer_num)
//             .collect::<Vec<(&LayerNum, &Color)>>()
//             .iter()
//             .nth(0)
//             .expect("Should not be calling path spawn for layer that doesn't already exist.")
//             .1;

//         let mut path = PathBuilder::new();
//         path.move_to(points[0]);

//         (&points[1..]).iter().for_each(|p| {
//             path.line_to(*p);
//         });
//         path.close();
//         let path = path.build();

//         commands.spawn_bundle(GeometryBuilder::build_as(
//             &path,
//             ShapeColors::outlined(*color.clone().set_a(ALPHA), *color),
//             DrawMode::Outlined {
//                 fill_options: FillOptions::default(),
//                 outline_options: StrokeOptions::default().with_line_width(width),
//             },
//             Transform::default(),
//         ));
//     }
// }

// pub fn get_layers_with_z_index(query: Query<&LayerNum, With<Layer>>) -> Vec<(usize, u16)> {
//     let mut layers = query.iter().map(|l| **l).collect::<Vec<u16>>();
//     layers.sort();
//     layers
//         .into_iter()
//         .enumerate()
//         .collect::<Vec<(usize, u16)>>()
// }
