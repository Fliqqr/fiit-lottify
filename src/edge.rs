use std::collections::HashSet;

struct Vertex {
    index: u64,
    neighbors: HashSet<u64>,
    edge: bool,
}

fn find_edge_verts() -> Vec<Vertex> {
    todo!()

    /*
    1. place a triangle

    2. for each vertex of triangle:
        2.1 take the set of all neighbors of the vertex
        2.2 count the number of edges within the subset
        2.3 if num edges == len set {
            vertex is not edge
        }
    */
}
