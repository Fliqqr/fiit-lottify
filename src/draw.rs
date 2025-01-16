use std::{
    collections::{hash_map::IterMut, HashMap, HashSet},
    hash::{self, DefaultHasher},
};

use bevy::{asset::AssetContainer, prelude::*, utils::PassHash};
use bevy_vello::prelude::*;

use kurbo::{Affine, PathEl, Point, Stroke};

use crate::lottie::{self};

/*
How to determine whether a vertex is an edge vertex:

Assuming vertex 'V' which has neighbors 'nV'

The number of 'nV' has to be equal to the number of edges amongst 'nV'.

Example;
if vertex has 3 neighbors, the neighbors have to have 3 edges amongst themselves.
*/

/*
[(1, 2, 5), (1, 5, 4), (2, 5, 3), (4, 5, 3)]

1 - 2, 5, 4
2 - 1, 5, 3
5 - 1, 2, 4, 3
4 - 1, 5, 3
3 - 2, 5, 4
*/

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

struct Vertex {
    neighbors: HashSet<usize>,
    path: bool,
    ind: usize,
}

fn draw_frame(indices: &[usize], verts: &[[f32; 3]], scene: &mut VelloScene) {
    let sorted_faces = sort_faces(indices, verts);

    // println!("Faces: {:?}", sorted_faces);

    for face in sorted_faces {
        let [a, b, c] = [face[0], face[1], face[2]];
        let normal = face_normal(verts[a], verts[b], verts[c]);

        if normal[2] <= 0.0 {
            // println!("Normal exc.");
            continue;
        }

        scene.stroke(
            &Stroke {
                width: 0.001,
                ..Default::default()
            },
            Affine::IDENTITY,
            peniko::Color::GRAY,
            None,
            &[
                PathEl::MoveTo((verts[a][0], -verts[a][1]).into()),
                PathEl::LineTo((verts[b][0], -verts[b][1]).into()),
                PathEl::LineTo((verts[c][0], -verts[c][1]).into()),
                PathEl::ClosePath,
            ],
        );
    }
}

fn generate_shape(indices: &[usize], verts: &[[f32; 3]]) -> Vec<PathEl> {
    let mut mapping: HashMap<usize, Vertex> = HashMap::new();
    let sorted_faces = sort_faces(indices, verts);

    // println!("Faces: {:?}", sorted_faces);
    let mut f_count = 0;

    for face in sorted_faces {
        let [a, b, c] = [face[0], face[1], face[2]];
        let normal = face_normal(verts[a], verts[b], verts[c]);

        println!("normal: {}", normal[2]);

        // This threshold shouldn't be needed, but Im currently too lazy
        // to figure out what's causing the real issue
        if normal[2] <= 0.0 {
            println!("Normal exc.");
            continue;
        }

        f_count += 1;

        let hash_a = hash_pos(verts[a]);
        let hash_b = hash_pos(verts[b]);
        let hash_c = hash_pos(verts[c]);

        for vert in [a, b, c] {
            let hash = hash_pos(verts[vert]);
            let nbr = HashSet::from([hash_a, hash_b, hash_c])
                .difference(&[hash].into())
                .cloned()
                .collect();

            if let Some(v) = mapping.get_mut(&hash) {
                v.neighbors = v.neighbors.symmetric_difference(&nbr).cloned().collect();
                continue;
            }

            mapping.insert(
                hash,
                Vertex {
                    neighbors: nbr,
                    path: false,
                    ind: vert,
                },
            );
        }
    }

    // println!("Faces: {}", f_count);

    let mut used = Vec::new();
    let mut out = Vec::new();

    for (hash, vert) in mapping.iter() {
        if !vert.neighbors.is_empty() && !used.contains(hash) {
            let mut pos = verts[vert.ind];
            let mut current_vertex;

            out.push(PathEl::MoveTo((pos[0], -pos[1]).into()));
            used.push(*hash);

            current_vertex = Some(vert);

            'outer: while let Some(curr) = current_vertex {
                // println!("{:?}", curr.ind);
                for nbr in &curr.neighbors {
                    if used.contains(nbr) {
                        continue;
                    };

                    let n_vert = mapping.get(nbr).unwrap();
                    pos = verts[n_vert.ind];

                    out.push(PathEl::LineTo((pos[0], -pos[1]).into()));
                    used.push(*nbr);

                    current_vertex = Some(n_vert);
                    continue 'outer;
                }
                break 'outer;
            }
            out.push(PathEl::ClosePath);
        }
    }
    out
}

pub fn draw_collection(meshes: Vec<Mesh>, colors: Vec<Color>, scene: &mut VelloScene) {
    // println!("Meshes: {}", meshes.len());

    for (mesh, color) in meshes.iter().zip(colors) {
        let lin_color = color.to_linear();

        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .expect("`Mesh::ATTRIBUTE_POSITION` vertex attributes should be of type `float3`");

        let indices = mesh.indices().unwrap().iter().collect::<Vec<usize>>();

        // println!("Pos {:?}", positions);
        // println!("Ind {:?}", indices);

        draw_frame(&indices, positions, scene);

        scene.fill(
            peniko::Fill::NonZero,
            // &Stroke {
            //     width: 0.002,
            //     ..Default::default()
            // },
            Affine::IDENTITY,
            peniko::Color::rgb(
                (lin_color.red) as f64,
                (lin_color.green) as f64,
                (lin_color.blue) as f64,
            ),
            None,
            &generate_shape(&indices, positions).as_slice(),
        );
    }

    // let mut lottie = lottie::Lottie::new();

    // for f in sf(&indices, &verts) {
    //     let [a, b, c] = [f[0], f[1], f[2]];
    //     let normal = face_normal(verts[a], verts[b], verts[c]);

    //     if normal[2] <= 0.0 {
    //         continue;
    //     }

    //     let pa: Point = (verts[a][0], verts[a][1]).into();
    //     let pb: Point = (verts[b][0], verts[b][1]).into();
    //     let pc: Point = (verts[c][0], verts[c][1]).into();

    //     let light = f32::log10(normal[2]) + 1.0;

    //     let lin_cs = cs[a].to_linear();

    //     lottie.add_shape(
    //         vec![(pa.x, -pa.y), (pb.x, -pb.y), (pc.x, -pc.y)],
    //         (lin_cs.red as f64, lin_cs.green as f64, lin_cs.blue as f64),
    //     );

    // lottie.save_as("assets/generated.json");
}
