use bevy::prelude::*;
use bevy_vello::prelude::*;

use kurbo::{Affine, PathEl, Point};

use crate::lottie::{self};

/*
How to determine whether a vertex is an edge vertex:

Assuming vertex 'V' which has neighbors 'nV'

The number of 'nV' has to be equal to the number of edges amongst 'nV'.

Example;
if vertex has 3 neighbors, the neighbors have to have 3 edges amongst themselves.
*/

fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let (a, b, c) = (Vec3::from(a), Vec3::from(b), Vec3::from(c));
    (b - a).cross(c - a).normalize().into()
}

fn sf(indecies: &[usize], verts: &[[f32; 3]]) -> Vec<[usize; 3]> {
    let mut faces: Vec<[usize; 3]> = indecies
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

pub fn draw_collection(meshes: Vec<Mesh>, colors: Vec<Color>, scene: &mut VelloScene) {
    let mut verts = Vec::new();
    let mut indecies = Vec::new();
    let mut cs = Vec::new();

    let mut offset = 0;

    for (mesh, color) in meshes.iter().zip(colors) {
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

        let clrs = vec![color; positions.len()];

        offset += positions.len();

        verts.extend_from_slice(positions);
        indecies.extend(indices);
        cs.extend(clrs);
    }

    let mut lottie = lottie::Lottie::new();

    for f in sf(&indecies, &verts) {
        let [a, b, c] = [f[0], f[1], f[2]];
        let normal = face_normal(verts[a], verts[b], verts[c]);

        if normal[2] <= 0.0 {
            continue;
        }

        let pa: Point = (verts[a][0], verts[a][1]).into();
        let pb: Point = (verts[b][0], verts[b][1]).into();
        let pc: Point = (verts[c][0], verts[c][1]).into();

        let light = f32::log10(normal[2]) + 1.0;

        let lin_cs = cs[a].to_linear();

        lottie.add_shape(
            vec![(pa.x, -pa.y), (pb.x, -pb.y), (pc.x, -pc.y)],
            (lin_cs.red as f64, lin_cs.green as f64, lin_cs.blue as f64),
        );

        scene.fill(
            peniko::Fill::EvenOdd,
            Affine::IDENTITY,
            peniko::Color::rgb(
                (lin_cs.red * light) as f64,
                (lin_cs.green * light) as f64,
                (lin_cs.blue * light) as f64,
            ),
            None,
            &[
                PathEl::MoveTo((pa.x, -pa.y).into()),
                PathEl::LineTo((pb.x, -pb.y).into()),
                PathEl::LineTo((pc.x, -pc.y).into()),
                PathEl::ClosePath,
            ],
        );
    }

    // lottie.save_as("assets/generated.json");
}
