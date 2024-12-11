use dungeon_3d_generator::delaunary_3d::Delaunay3D;
use kiss3d::light::Light;
use kiss3d::nalgebra::Point3;
use kiss3d::window::Window;

fn main() {
    let mut window = Window::new("Dungeon 3D Generator");

    window.set_light(Light::StickToCamera);

    let points = vec![
        ((), nalgebra::Vector3::new(0.0, 0.0, 0.0)),
        ((), nalgebra::Vector3::new(1.0, 0.0, 0.0)),
        ((), nalgebra::Vector3::new(0.0, 0.0, 1.0)),
        ((), nalgebra::Vector3::new(0.0, 1.0, 1.0)),
    ];
    let delaunary = Delaunay3D::new(points.clone());

    while window.render() {
        for point in points.iter() {
            window.draw_point(
                &Point3::new(point.1.x, point.1.y, point.1.z),
                &Point3::new(0.0, 1.0, 1.0),
            );
        }

        for edge in delaunary.edges.iter() {
            window.draw_line(
                &Point3::new(edge.u.position.x, edge.u.position.y, edge.u.position.z),
                &Point3::new(edge.v.position.x, edge.v.position.y, edge.v.position.z),
                &Point3::new(0.0, 1.0, 1.0),
            );
        }
    }
}
