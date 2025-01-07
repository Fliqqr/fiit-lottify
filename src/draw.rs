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
    loop_edges: u64,
    is_edge: bool,
    ind: usize,
}

fn draw_frame(indices: &[usize], verts: &[[f32; 3]], scene: &mut VelloScene) {
    let sorted_faces = sort_faces(indices, verts);

    println!("Faces: {:?}", sorted_faces);

    for face in sorted_faces {
        let [a, b, c] = [face[0], face[1], face[2]];
        let normal = face_normal(verts[a], verts[b], verts[c]);

        if normal[2] <= 0.0 {
            println!("Normal exc.");
            continue;
        }

        scene.stroke(
            &Stroke {
                width: 0.01,
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

    for face in sorted_faces {
        let [a, b, c] = [face[0], face[1], face[2]];
        let normal = face_normal(verts[a], verts[b], verts[c]);

        if normal[2] <= 0.0 {
            // println!("Normal exc.");
            continue;
        }

        let hash_a = hash_pos(verts[a]);
        let hash_b = hash_pos(verts[b]);
        let hash_c = hash_pos(verts[c]);

        // println!(
        //     "face: \na: {:?} {}\nb: {:?} {}\nc: {:?} {}",
        //     verts[a], hash_a, verts[b], hash_b, verts[c], hash_c
        // );

        if let Some(v) = mapping.get_mut(&hash_a) {
            v.loop_edges += 1;
            v.neighbors.extend([hash_b, hash_c]);

            if v.loop_edges == (v.neighbors.len() as u64) {
                v.is_edge = false;
            }
        } else {
            mapping.insert(
                hash_pos(verts[a]),
                Vertex {
                    neighbors: HashSet::from([hash_b, hash_c]),
                    loop_edges: 1,
                    is_edge: true,
                    ind: a,
                },
            );
        }

        if let Some(v) = mapping.get_mut(&hash_b) {
            v.loop_edges += 1;
            v.neighbors.extend([hash_a, hash_c]);

            if v.loop_edges == (v.neighbors.len() as u64) {
                v.is_edge = false;
            }
        } else {
            mapping.insert(
                hash_pos(verts[b]),
                Vertex {
                    neighbors: HashSet::from([hash_a, hash_c]),
                    loop_edges: 1,
                    is_edge: true,
                    ind: b,
                },
            );
        }

        if let Some(v) = mapping.get_mut(&hash_c) {
            v.loop_edges += 1;
            v.neighbors.extend([hash_a, hash_b]);

            if v.loop_edges == (v.neighbors.len() as u64) {
                v.is_edge = false;
            }
        } else {
            mapping.insert(
                hash_pos(verts[c]),
                Vertex {
                    neighbors: HashSet::from([hash_a, hash_b]),
                    loop_edges: 1,
                    is_edge: true,
                    ind: c,
                },
            );
        }
    }

    // println!("Mapping: {}", mapping.len());

    let mut count = 0;
    for (_, vert) in mapping.iter() {
        if vert.is_edge {
            count += 1;
        }
    }
    // println!("Edges: {}", count);

    if count < 3 {
        return vec![];
    }

    let mut seq = vec![];
    let mut out = vec![];
    let mut used = vec![];

    for (index, vert) in mapping.iter() {
        if vert.is_edge && !used.contains(&vert.ind) {
            let mut curr = vert;
            seq.push(vert.ind);
            used.push(vert.ind);

            'outer: loop {
                let mut priority: Option<&Vertex> = None;
                let mut least = usize::MAX;

                for nbr in &curr.neighbors {
                    let n_vert = mapping.get(nbr).unwrap();

                    if n_vert.is_edge
                        && !seq.contains(&n_vert.ind)
                        && n_vert.neighbors.len() < least
                    {
                        priority = Some(n_vert);
                        least = n_vert.neighbors.len();
                    }
                }
                if let Some(next) = priority {
                    seq.push(next.ind);
                    used.push(next.ind);
                    curr = next;
                    continue 'outer;
                }

                break 'outer;
            }
            let mut iterator = seq.iter();

            let first = iterator.next().unwrap();
            let pos = verts[*first];
            out.push(PathEl::MoveTo((pos[0], -pos[1]).into()));

            // println!("seq: {:?}", seq);

            for vert in iterator {
                let pos = verts[*vert];

                out.push(PathEl::LineTo((pos[0], -pos[1]).into()));
            }
            out.push(PathEl::ClosePath);
            seq.clear();
        }
    }

    // println!("{:?}", out);

    out
}

pub fn draw_collection(meshes: Vec<Mesh>, colors: Vec<Color>, scene: &mut VelloScene) {
    let mut offset = 0;

    // println!("Meshes: {}", meshes.len());

    for (mesh, color) in meshes.iter().zip(colors) {
        let lin_color = color.to_linear();

        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .expect("`Mesh::ATTRIBUTE_POSITION` vertex attributes should be of type `float3`");

        let indices = mesh
            .indices()
            .unwrap()
            .iter()
            .map(|i| i + offset)
            .collect::<Vec<usize>>();

        // println!("Pos {:?}", positions);
        // println!("Ind {:?}", indices);

        draw_frame(&indices, positions, scene);

        scene.fill(
            peniko::Fill::NonZero,
            // &Stroke {
            //     width: 0.01,
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
            // &[
            //     PathEl::MoveTo((10.0, 10.0).into()),
            //     PathEl::LineTo((-10.0, 10.0).into()),
            //     PathEl::LineTo((-10.0, -10.0).into()),
            //     PathEl::ClosePath,
            //     PathEl::MoveTo((50.0, 50.0).into()),
            //     PathEl::LineTo((30.0, 30.0).into()),
            //     PathEl::LineTo((40.0, 30.0).into()),
            //     PathEl::ClosePath,
            // ],
        );

        // offset += positions.len();
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
