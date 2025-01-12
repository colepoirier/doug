use bevy::prelude::*;

use bevy::utils::HashMap;

use cell::LoadCellPlugin;
use layout21::raw::{BoundBox, Cell, Element, Library};
use library::ImportLibraryPlugin;

use std::{slice::Iter, sync::Arc};

pub mod cell;
pub mod library;
pub mod poly_decomp;
pub mod shapes;

#[derive(Debug, Deref, Resource)]
pub struct RectMesh(pub Handle<Mesh>);

pub fn get_shapes(cell: &Cell) -> Iter<Element> {
    cell.layout.as_ref().unwrap().elems.iter()
}

#[derive(Resource, Debug, Default)]
pub struct VlsirLib {
    pub lib: Arc<Library>,
    pub cell_names: Vec<String>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Layer {
    pub name: Option<String>,
    pub color: Handle<ColorMaterial>,
}

#[derive(Resource, Debug, Default, Clone, Deref, DerefMut)]
pub struct Layers(Arc<HashMap<u8, Layer>>);

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct VlsirCell {
    pub index: Option<usize>,
    pub num_shapes: Option<u64>,
}

#[derive(Debug, Default, Clone)]
pub struct CellContentsInfo {
    pub cell_name: String,
    pub layout: Option<LayoutInfo>,
    pub abstract_info: Option<AbstractInfo>,
}

#[derive(Debug, Default, Clone)]
pub struct LayoutInfo {
    pub layout_name: String,
    pub elems: usize,
    pub insts: usize,
    pub annotations: usize,
}

#[derive(Debug, Default, Clone)]
pub struct AbstractInfo {
    pub abstract_name: String,
    pub outline: BoundBox,
    pub ports: usize,
    pub blockages: usize,
}

pub struct Layout21ImportPlugin;

impl Plugin for Layout21ImportPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ImportLibraryPlugin, LoadCellPlugin));
    }
}
