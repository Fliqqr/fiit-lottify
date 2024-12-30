use bevy::prelude::*;
use bevy_vello::prelude::*;

use kurbo::{Affine, PathEl, Point};

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

pub fn draw_collection(meshes: Vec<Mesh>, scene: &mut VelloScene) {
    let mut verts = Vec::new();
    let mut indecies = Vec::new();
    // let mut colors = Vec::new();

    let mut offset = 0;

    for mesh in meshes {
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

        // let clrs = vec![mesh.material.as_ref().unwrap(); positions.len()];

        offset += positions.len();

        verts.extend_from_slice(positions);
        indecies.extend(indices);
        // colors.extend(clrs);
    }

    for f in sf(&indecies, &verts) {
        let [a, b, c] = [f[0], f[1], f[2]];
        let normal = face_normal(verts[a], verts[b], verts[c]);

        if normal[2] <= 0.0 {
            continue;
        }

        let pa: Point = (verts[a][0], verts[a][1]).into();
        let pb: Point = (verts[b][0], verts[b][1]).into();
        let pc: Point = (verts[c][0], verts[c][1]).into();
        // let light = f32::log10(normal[2]) + 1.0;

        let perps = 0.0005;
        // let clr = colors[a].base_color.to_linear();

        scene.fill(
            peniko::Fill::EvenOdd,
            Affine::IDENTITY,
            // peniko::Color::rgb((1.0 * 1.0) as f64, 0.0, 0.0),
            // peniko::Color::rgb(
            //     (clr.red * light) as f64,
            //     (clr.green * light) as f64,
            //     (clr.blue * light) as f64,
            // ),
            peniko::Color::RED,
            None,
            &[
                PathEl::MoveTo(
                    (
                        pa.x + (pa.x * perps * verts[a][2] as f64),
                        -pa.y + (pa.y * perps * verts[a][2] as f64),
                    )
                        .into(),
                ),
                PathEl::LineTo(
                    (
                        pb.x + (pb.x * perps * verts[b][2] as f64),
                        -pb.y + (pb.y * perps * verts[b][2] as f64),
                    )
                        .into(),
                ),
                PathEl::LineTo(
                    (
                        pc.x + (pc.x * perps * verts[c][2] as f64),
                        -pc.y + (pc.y * perps * verts[c][2] as f64),
                    )
                        .into(),
                ),
                PathEl::ClosePath,
            ],
        );
    }
}
