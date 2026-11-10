use glam::Vec3;
use graphics_lab::scene;

#[test]
fn every_triangle_is_non_degenerate_and_faces_its_declared_normal() {
    for mesh in [scene::cube(), scene::floor()] {
        for indices in mesh.indices.chunks_exact(3) {
            let vertices = [indices[0], indices[1], indices[2]].map(|i| mesh.vertices[i as usize]);
            let [a, b, c] = vertices.map(|v| Vec3::from_slice(&v.position));
            let cross = (b - a).cross(c - a);
            assert!(cross.length_squared() > 0.001);
            for vertex in vertices {
                let normal = Vec3::from_slice(&vertex.normal);
                assert!(
                    cross.normalize().dot(normal) > 0.99,
                    "triangle winding disagrees with normal"
                );
            }
        }
    }
}
