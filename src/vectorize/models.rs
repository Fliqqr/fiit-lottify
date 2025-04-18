use bevy::prelude::*;
use bevy_vello::vello::kurbo::PathEl;

#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    pub paths: Vec<PathEl>,
}

impl Shape {
    pub fn new(paths: Vec<PathEl>) -> Self {
        Self { paths }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeshShape {
    pub shape: Shape,
    pub color: Color,
    pub mesh_id: AssetId<Mesh>,
    pub name: Option<String>,
}

impl MeshShape {
    pub fn new(shape: Shape, color: Color, id: AssetId<Mesh>) -> Self {
        Self {
            shape,
            color,
            mesh_id: id,
            name: None,
        }
    }
}
