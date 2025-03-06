// for mut t in trans.iter_mut() {
//     // t.rotate_y(0.01);
//     // t.rotate_x(0.01);
//     // *t = Transform::from_rotation(
//     //     Quat::from_rotation_y(4.1),
//     //     // Quat::from_xyzw(3.0, 7.1, 0.0, 1.0),
//     // );
// }

// for (t, m, name, mat_handle) in query.iter_mut() {
//     if let Some(mesh) = meshes.get(m) {
//         let material = materials.get(mat_handle).expect("Mesh has no material");
//         // println!("{}", name);

//         // if name.as_str() != "Object_0" {
//         //     continue;
//         // }

//         let transformed_mesh = mesh.clone().transformed_by((*t).into());

//         t_meshes.push(transformed_mesh);
//         t_colors.push(material.base_color);
//     }
// }

// draw_collection(t_meshes, t_colors, &mut scene);

// fn draw_frame(indices: &[usize], verts: &[[f32; 3]], scene: &mut VelloScene) {
//     let sorted_faces = sort_faces(indices, verts);

//     // println!("Faces: {:?}", sorted_faces);

//     for face in sorted_faces {
//         let [a, b, c] = [face[0], face[1], face[2]];
//         let normal = face_normal(verts[a], verts[b], verts[c]);

//         if normal[2] <= 0.0 {
//             // println!("Normal exc.");
//             continue;
//         }

//         scene.stroke(
//             &Stroke {
//                 width: 0.01,
//                 ..Default::default()
//             },
//             Affine::IDENTITY,
//             peniko::Color::GRAY,
//             None,
//             &[
//                 PathEl::MoveTo((verts[a][0], -verts[a][1]).into()),
//                 PathEl::LineTo((verts[b][0], -verts[b][1]).into()),
//                 PathEl::LineTo((verts[c][0], -verts[c][1]).into()),
//                 PathEl::ClosePath,
//             ],
//         );
//     }
// }

// fn draw_mapping(mapping: &HashMap<usize, Vertex>, scene: &mut VelloScene) {
//     let mut stroke = Vec::new();

//     for (_, vertex) in mapping.iter() {
//         for path in &vertex.paths {
//             let mut edge_iter = path.edges.iter();
//             let e1 = edge_iter.next().unwrap();
//             let e2 = edge_iter.next().unwrap();

//             let e1_pos = mapping.get(&e1.vertex_hash).unwrap().position;
//             let e2_pos = mapping.get(&e2.vertex_hash).unwrap().position;

//             stroke.push(PathEl::MoveTo((e1_pos[0], -e1_pos[1]).into()));
//             stroke.push(PathEl::LineTo(
//                 (vertex.position[0], -vertex.position[1]).into(),
//             ));
//             stroke.push(PathEl::LineTo((e2_pos[0], -e2_pos[1]).into()));

//             scene.stroke(
//                 &Stroke {
//                     width: 0.01,
//                     ..Default::default()
//                 },
//                 Affine::IDENTITY,
//                 peniko::Color::GRAY,
//                 None,
//                 &stroke.as_slice(),
//             );

//             stroke.clear();
//         }
//     }
// }

// scene.fill(
//     peniko::Fill::NonZero,
//     kurbo::Affine::default(),
//     peniko::Color::BLUE,
//     None,
//     &[
//         PathEl::MoveTo((-10.0, -10.0).into()),
//         PathEl::LineTo((10.0, -10.0).into()),
//         PathEl::LineTo((10.0, 10.0).into()),
//         PathEl::LineTo((-10.0, 10.0).into()),
//         PathEl::ClosePath,
//         PathEl::MoveTo((-5.0, -5.0).into()),
//         PathEl::LineTo((-5.0, 5.0).into()),
//         PathEl::LineTo((5.0, 5.0).into()),
//         PathEl::LineTo((5.0, -5.0).into()),
//         PathEl::ClosePath,
//     ],
// );
