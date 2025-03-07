use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use bevy::prelude::*;
use bevy_vello::prelude::*;

use kurbo::PathEl;

fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let (a, b, c) = (Vec3::from(a), Vec3::from(b), Vec3::from(c));
    (b - a).cross(c - a).normalize().into()
}

fn sort_faces(indices: &[usize], verts: &[[f32; 3]]) -> Vec<[usize; 3]> {
    let mut faces: Vec<[usize; 3]> = indices
        .chunks_exact(3)
        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
        .collect();

    faces.sort_by(|f1, f2| {
        let avg_z1 = (verts[f1[0]][2] + verts[f1[1]][2] + verts[f1[2]][2]) / 3.0;
        let avg_z2 = (verts[f2[0]][2] + verts[f2[1]][2] + verts[f2[2]][2]) / 3.0;

        avg_z1
            .partial_cmp(&avg_z2)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    faces
}

fn hash_pos(pos: [f32; 3]) -> usize {
    ((pos[0].to_bits() as u64) * 31 + (pos[1].to_bits() as u64) * 7 + (pos[2].to_bits() as u64))
        as usize
}

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Hash, Debug, Clone)]
struct Edge {
    vertex_hash: usize,
    rotation: bool,
}

impl Edge {
    pub fn new(vertex_hash: usize, rotation: bool) -> Self {
        Self {
            vertex_hash,
            rotation,
        }
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        (self.vertex_hash == other.vertex_hash) && (self.rotation == other.rotation)
    }
}

impl Eq for Edge {}

enum JointLevel {
    SharedVertex,
    // Rename to overlap
    SharedEdge,
    MergeEdge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Path {
    edges: HashSet<Edge>,
}

impl Path {
    pub fn new(edges: Vec<Edge>) -> Self {
        Self {
            edges: HashSet::from_iter(edges),
        }
    }

    // Merges two paths into one
    pub fn merge(mut self, other: Path) -> Self {
        self.edges = self.symmetric_difference(&other);

        // if !self.edges.is_empty() && self.edges.len() != 2 {
        //     panic!(
        //         "Path has incompatible number of edges: {}",
        //         self.edges.len()
        //     );
        // }

        self
    }

    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    pub fn contains(&self, vertex_hash: usize) -> bool {
        for edge in &self.edges {
            if edge.vertex_hash == vertex_hash {
                return true;
            }
        }
        false
    }

    pub fn joint_level(&self, other: &Path) -> JointLevel {
        for e1 in &self.edges {
            for e2 in &other.edges {
                if e1.vertex_hash == e2.vertex_hash {
                    if e1.rotation != e2.rotation {
                        return JointLevel::MergeEdge;
                    }
                    // Overlap, rename
                    return JointLevel::SharedEdge;
                }
            }
        }
        JointLevel::SharedVertex
    }

    pub fn difference(&self, other: &Path) -> HashSet<Edge> {
        let mut out = HashSet::new();

        'outer: for e1 in &self.edges {
            for e2 in &other.edges {
                if e1.vertex_hash == e2.vertex_hash && e1.rotation != e2.rotation {
                    continue 'outer;
                }
            }
            out.insert(e1.clone());
        }
        out
    }

    pub fn symmetric_difference(&self, other: &Path) -> HashSet<Edge> {
        let mut left = self.difference(other);
        left.extend(other.difference(self));

        left
    }
}

#[derive(Debug, Clone)]
struct Vertex {
    paths: Vec<Path>,
    position: [f32; 2],
}

impl Vertex {
    pub fn new(paths: Vec<Path>, position: [f32; 2]) -> Self {
        Self { paths, position }
    }

    // pub fn add_path(&mut self, path: Path) {
    //     self.paths.push(path);
    // }
}

// Made by ChatGPT, take with a grain of salt
fn calc_rotation(ax1: [f32; 3], ax2: [f32; 3], side: [f32; 3]) -> bool {
    // Calculate the 2D edge vector
    let edge = [ax2[0] - ax1[0], ax2[1] - ax1[1]];

    // Calculate the vector from ax1 to side
    let to_side = [side[0] - ax1[0], side[1] - ax1[1]];

    // Compute the 2D cross product
    let cross = edge[0] * to_side[1] - edge[1] * to_side[0];

    // Return true if the side is on the left (positive cross product), false otherwise
    cross > 0.0
}

#[derive(PartialEq)]
enum ProcessResult {
    Ok,
    Skip,
}

fn process_face(
    indices: Vec<usize>,
    positions: &[[f32; 3]],
    mapping: &mut HashMap<usize, Vertex>,
) -> ProcessResult {
    let mut face_indices = indices.clone();
    // println!("--face--");

    let mut to_commit = Vec::new();

    for _ in 0..3 {
        let index = face_indices.pop().unwrap();

        let pos = positions[index];
        let hash = hash_pos(pos);

        let e1 = Edge::new(
            hash_pos(positions[face_indices[0]]),
            calc_rotation(pos, positions[face_indices[0]], positions[face_indices[1]]),
        );
        let e2 = Edge::new(
            hash_pos(positions[face_indices[1]]),
            calc_rotation(pos, positions[face_indices[1]], positions[face_indices[0]]),
        );

        let mut new_path = Path::new([e1, e2].into());
        // println!("New path: {:?}", new_path);

        // Vertex is already in mapping
        if let Some(v) = mapping.get(&hash) {
            let mut vertex = v.clone();
            // println!("Already in: {}", hash);
            let mut replacement_paths: Vec<Path> = Vec::new();

            // Iterate through existing paths to check for overlap with new path
            for path in &vertex.paths {
                match path.joint_level(&new_path) {
                    // This should go to a different layer if no merging is done
                    JointLevel::SharedVertex => {
                        replacement_paths.push(path.clone());
                        continue;
                    }
                    // Aligned edges
                    JointLevel::SharedEdge => {
                        // Skip this face
                        // println!("SKIP");
                        // This might not work that easily
                        // mapping.insert(hash, vertex);
                        return ProcessResult::Skip;
                    }
                    JointLevel::MergeEdge => {
                        // println!("MERGE: {:?} {:?}", new_path, path);
                        new_path = new_path.merge(path.clone());
                        // println!("OUT: {:?}", new_path);
                    }
                }
            }

            if !new_path.is_empty() {
                replacement_paths.push(new_path);
            }

            vertex.paths = replacement_paths;

            // println!("Reinsert");
            // mapping.insert(hash, vertex);
            to_commit.push((hash, vertex));
        } else {
            // println!("New insert: {}", hash);
            to_commit.push((hash, Vertex::new(vec![new_path], [pos[0], pos[1]])));
            // mapping.insert(hash, Vertex::new(vec![new_path], [pos[0], pos[1]]));
        }

        face_indices.insert(0, index);
    }

    if !to_commit.is_empty() {
        for (hash, vert) in to_commit {
            mapping.insert(hash, vert);
        }
    }

    ProcessResult::Ok
}

#[derive(Clone)]
pub struct Shape {
    pub paths: Vec<PathEl>,
}

impl Shape {
    pub fn new(paths: Vec<PathEl>) -> Self {
        Self { paths }
    }
}

#[derive(Clone)]
pub struct MeshShape {
    pub shape: Shape,
    pub color: Color,
}

impl MeshShape {
    pub fn new(shape: Shape, color: Color) -> Self {
        Self { shape, color }
    }
}

pub fn round_to(num: impl Into<f64>, places: u32) -> f64 {
    let num = num.into();
    let pow = 10_f64.powf(places as f64);

    (num * pow).round() / pow
}

const SCALE: f32 = 100.0;

fn generate_shape(
    indices: &[usize],
    positions: &[[f32; 3]],
    color: Color,
) -> Result<MeshShape, Shape> {
    let mut tmp = Vec::new();

    let mut curr_face_batch = sort_faces(indices, positions);
    let mut skipped_faces = Vec::new();

    let mut rotation = None::<bool>;

    'face_batch_exists: loop {
        let mut mapping: HashMap<usize, Vertex> = HashMap::new();

        for face in &curr_face_batch {
            let [a, b, c] = [face[0], face[1], face[2]];
            let normal = face_normal(positions[a], positions[b], positions[c]);

            if normal[2] <= 0.0 {
                continue;
            }

            if process_face(vec![a, b, c], positions, &mut mapping) == ProcessResult::Skip {
                skipped_faces.push(*face);
            }
        }

        // println!("Mapping:");
        // for (hash, vert) in mapping.iter() {
        //     println!("{} {:?}", hash, vert);
        // }

        // draw_mapping(&mapping, scene);

        let mut used: Vec<&Path> = Vec::new();
        let mut shape = Vec::new();

        // TODO: Only allow 1 edge rotation across all sub-shapes and merge them into a single shape
        'vertex: for (hash, vertex) in mapping.iter() {
            if vertex.paths.is_empty() {
                continue;
            }

            let mut curr = *hash;
            let mut next = vertex.paths[0].edges.iter().next().unwrap();

            let mut flag = false;

            for path in &vertex.paths {
                if !used.contains(&path) {
                    // This should in theory always find a valid edge, but who knows
                    for edge in &path.edges {
                        if let Some(rot) = rotation {
                            if edge.rotation == rot {
                                next = edge;
                            }
                        } else {
                            rotation = Some(next.rotation);
                        }
                    }

                    // next = path.edges.iter().next().unwrap();
                    used.push(path);
                    flag = true;
                    break;
                }
            }

            if !flag {
                continue 'vertex;
            }

            shape.push(PathEl::MoveTo(
                (
                    round_to(vertex.position[0] * SCALE, 3),
                    -round_to(vertex.position[1] * SCALE, 3),
                )
                    .into(),
            ));

            'outer: loop {
                let next_vert = mapping.get(&next.vertex_hash).unwrap();
                let mut found_next = false;

                // println!("curr: {} -> next: {:?}", curr, next.vertex_hash);
                // println!("{:?}", next_vert);

                shape.push(PathEl::LineTo(
                    (
                        round_to(next_vert.position[0] * SCALE, 3),
                        -round_to(next_vert.position[1] * SCALE, 3),
                    )
                        .into(),
                ));
                // shape.push(PathEl::QuadTo(
                //     (next_vert.position[0] + 0.01, -next_vert.position[1] + 0.01).into(),
                //     (next_vert.position[0], -next_vert.position[1]).into(),
                // ));
                // used.push(next.vertex_hash);

                'path: for path in &next_vert.paths {
                    if !path.contains(curr) {
                        continue;
                    }

                    for edge in &path.edges {
                        if edge.vertex_hash != curr {
                            curr = next.vertex_hash;
                            next = edge;
                            found_next = true;

                            used.push(path);

                            break 'path;
                        }
                    }
                    return Err(Shape::new(shape));
                }

                if next.vertex_hash == *hash {
                    break 'outer;
                }

                if !found_next {
                    return Err(Shape::new(shape));
                }
            }

            shape.push(PathEl::ClosePath);
            tmp.extend(shape.clone());
            // out.shapes.push(Shape::new(shape.clone()));

            shape.clear();
        }

        // out.shapes.push(Shape::new(shape.clone()));

        if !skipped_faces.is_empty() {
            curr_face_batch = skipped_faces.clone();
            skipped_faces.clear();
        } else {
            break 'face_batch_exists;
        }
    }

    let out = MeshShape::new(Shape::new(tmp), color);

    Ok(out)
}

pub fn generate_collection(meshes: Vec<Mesh>, colors: Vec<Color>) -> Vec<MeshShape> {
    let mut out = Vec::new();

    for (mesh, color) in meshes.iter().zip(colors) {
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .expect("`Mesh::ATTRIBUTE_POSITION` vertex attributes should be of type `float3`");

        let indices = mesh.indices().unwrap().iter().collect::<Vec<usize>>();

        // Could run these in parallel
        if let Ok(shapes) = generate_shape(&indices, positions, color) {
            out.push(shapes);
        }
    }

    out
}
