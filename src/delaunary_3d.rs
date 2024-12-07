use nalgebra::{Matrix4, Vector3};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

const ACCURACY: f32 = 1000.0;

///
/// Reference: https://github.com/vazgriz/DungeonGenerator/blob/master/Assets/Scripts3D/Delaunay3D.cs
///
#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Vector3<f32>,
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        (
            (self.position.x * ACCURACY) as i64,
            (self.position.y * ACCURACY) as i64,
            (self.position.z * ACCURACY) as i64,
        ) == (
            (other.position.x * ACCURACY) as i64,
            (other.position.y * ACCURACY) as i64,
            (other.position.z * ACCURACY) as i64,
        )
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (
            (self.position.x * ACCURACY) as i64,
            (self.position.y * ACCURACY) as i64,
            (self.position.z * ACCURACY) as i64,
        )
            .hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct Tetrahedron {
    pub a: Vertex,
    pub b: Vertex,
    pub c: Vertex,
    pub d: Vertex,
    pub is_bad: bool,
    circumcenter: Vector3<f32>,
    circumradius_squared: f32,
}

impl Tetrahedron {
    pub fn new(a: Vertex, b: Vertex, c: Vertex, d: Vertex) -> Self {
        let mut tetra = Tetrahedron {
            a,
            b,
            c,
            d,
            is_bad: false,
            circumcenter: Vector3::zeros(),
            circumradius_squared: 0.0,
        };
        tetra.calculate_circumsphere();
        tetra
    }

    fn calculate_circumsphere(&mut self) {
        // Matrix determinant calculation for circumcenter and circumradius
        let a_matrix = Matrix4::new(
            self.a.position.x,
            self.b.position.x,
            self.c.position.x,
            self.d.position.x,
            self.a.position.y,
            self.b.position.y,
            self.c.position.y,
            self.d.position.y,
            self.a.position.z,
            self.b.position.z,
            self.c.position.z,
            self.d.position.z,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        let det_a = a_matrix.determinant();

        let pos_sqr_a = self.a.position.norm_squared();
        let pos_sqr_b = self.b.position.norm_squared();
        let pos_sqr_c = self.c.position.norm_squared();
        let pos_sqr_d = self.d.position.norm_squared();

        let dx_matrix = Matrix4::new(
            pos_sqr_a,
            pos_sqr_b,
            pos_sqr_c,
            pos_sqr_d,
            self.a.position.y,
            self.b.position.y,
            self.c.position.y,
            self.d.position.y,
            self.a.position.z,
            self.b.position.z,
            self.c.position.z,
            self.d.position.z,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        let dx = dx_matrix.determinant();

        let dy_matrix = Matrix4::new(
            pos_sqr_a,
            pos_sqr_b,
            pos_sqr_c,
            pos_sqr_d,
            self.a.position.x,
            self.b.position.x,
            self.c.position.x,
            self.d.position.x,
            self.a.position.z,
            self.b.position.z,
            self.c.position.z,
            self.d.position.z,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        let dy = -dy_matrix.determinant();

        let dz_matrix = Matrix4::new(
            pos_sqr_a,
            pos_sqr_b,
            pos_sqr_c,
            pos_sqr_d,
            self.a.position.x,
            self.b.position.x,
            self.c.position.x,
            self.d.position.x,
            self.a.position.y,
            self.b.position.y,
            self.c.position.y,
            self.d.position.y,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        let dz = dz_matrix.determinant();

        let c_matrix = Matrix4::new(
            pos_sqr_a,
            pos_sqr_b,
            pos_sqr_c,
            pos_sqr_d,
            self.a.position.x,
            self.b.position.x,
            self.c.position.x,
            self.d.position.x,
            self.a.position.y,
            self.b.position.y,
            self.c.position.y,
            self.d.position.y,
            self.a.position.z,
            self.b.position.z,
            self.c.position.z,
            self.d.position.z,
        );
        let det_c = c_matrix.determinant();

        self.circumcenter =
            Vector3::new(dx / (2.0 * det_a), dy / (2.0 * det_a), dz / (2.0 * det_a));
        self.circumradius_squared =
            (dx * dx + dy * dy + dz * dz - 4.0 * det_a * det_c) / (4.0 * det_a * det_a);
    }

    pub fn circum_circle_contains(&self, v: &Vector3<f32>) -> bool {
        let dist = v - self.circumcenter;
        dist.norm_squared() <= self.circumradius_squared
    }

    pub fn contains_vertex(&self, v: &Vertex) -> bool {
        v == &self.a || v == &self.b || v == &self.c || v == &self.d
    }
}

#[derive(Clone, Debug, Eq)]
pub struct Triangle {
    pub u: Vertex,
    pub v: Vertex,
    pub w: Vertex,
    pub is_bad: bool,
}

impl Hash for Triangle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.u.hash(state);
        self.v.hash(state);
        self.w.hash(state);
    }
}

impl Triangle {
    pub fn new(u: Vertex, v: Vertex, w: Vertex) -> Self {
        Self {
            u,
            v,
            w,
            is_bad: false,
        }
    }
}

impl PartialEq for Triangle {
    fn eq(&self, other: &Self) -> bool {
        (self.u == other.u || self.u == other.v || self.u == other.w)
            && (self.v == other.u || self.v == other.v || self.v == other.w)
            && (self.w == other.u || self.w == other.v || self.w == other.w)
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Edge {
    pub u: Vertex,
    pub v: Vertex,
    pub is_bad: bool,
}

impl Hash for Edge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.u.hash(state);
        self.v.hash(state);
    }
}

impl Edge {
    pub fn new(u: Vertex, v: Vertex) -> Self {
        Self {
            u,
            v,
            is_bad: false,
        }
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        (self.u == other.u || self.v == other.u) && (self.u == other.v || self.v == other.v)
    }
}

#[derive(Clone, Debug)]
pub struct Delaunay3D {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
    pub triangles: Vec<Triangle>,
    pub tetrahedra: Vec<Tetrahedron>,
}

impl Delaunay3D {
    pub fn new(vertices: Vec<Vector3<f32>>) -> Self {
        let mut ret = Self {
            vertices: vertices
                .into_iter()
                .map(|v| Vertex { position: v })
                .collect(),
            edges: Vec::new(),
            triangles: Vec::new(),
            tetrahedra: Vec::new(),
        };
        ret.triangulate();
        ret
    }

    fn triangulate(&mut self) {
        let mut min_x = self.vertices[0].position.x;
        let mut min_y = self.vertices[0].position.y;
        let mut min_z = self.vertices[0].position.z;
        let mut max_x = min_x;
        let mut max_y = min_y;
        let mut max_z = min_z;

        for vertex in self.vertices.iter() {
            if vertex.position.x < min_x {
                min_x = vertex.position.x;
            }
            if vertex.position.x > max_x {
                max_x = vertex.position.x;
            }
            if vertex.position.y < min_y {
                min_y = vertex.position.y;
            }
            if vertex.position.y > max_y {
                max_y = vertex.position.y;
            }
            if vertex.position.z < min_z {
                min_z = vertex.position.z;
            }
            if vertex.position.z > max_z {
                max_z = vertex.position.z;
            }
        }

        let dx = max_x - min_x;
        let dy = max_y - min_y;
        let dz = max_z - min_z;
        let delta_max = dx.max(dy.max(dz)) * 2.0;

        let p1 = Vertex {
            position: Vector3::new(min_x - 1.0, min_y - 1.0, min_z - 1.0),
        };
        let p2 = Vertex {
            position: Vector3::new(max_x + delta_max, min_y - 1.0, min_z - 1.0),
        };
        let p3 = Vertex {
            position: Vector3::new(min_x - 1.0, max_y + delta_max, min_z - 1.0),
        };
        let p4 = Vertex {
            position: Vector3::new(min_x - 1.0, min_y - 1.0, max_z + delta_max),
        };

        self.tetrahedra.push(Tetrahedron::new(
            p1.clone(),
            p2.clone(),
            p3.clone(),
            p4.clone(),
        ));

        for vertex in self.vertices.iter() {
            let mut triangles = Vec::new();
            for tetrahedron in self.tetrahedra.iter_mut() {
                if tetrahedron.circum_circle_contains(&vertex.position) {
                    tetrahedron.is_bad = true;
                    triangles.push(Triangle::new(
                        tetrahedron.a.clone(),
                        tetrahedron.b.clone(),
                        tetrahedron.c.clone(),
                    ));
                    triangles.push(Triangle::new(
                        tetrahedron.a.clone(),
                        tetrahedron.b.clone(),
                        tetrahedron.d.clone(),
                    ));
                    triangles.push(Triangle::new(
                        tetrahedron.a.clone(),
                        tetrahedron.c.clone(),
                        tetrahedron.d.clone(),
                    ));
                    triangles.push(Triangle::new(
                        tetrahedron.b.clone(),
                        tetrahedron.c.clone(),
                        tetrahedron.d.clone(),
                    ));
                }
            }

            for i in 0..triangles.len() {
                for j in (i + 1)..triangles.len() {
                    if triangles[i] == triangles[j] {
                        triangles[i].is_bad = true;
                        triangles[j].is_bad = true;
                    }
                }
            }

            self.tetrahedra.retain(|tetrahedron| !tetrahedron.is_bad);
            triangles.retain(|triangle| !triangle.is_bad);

            for triangle in triangles {
                self.tetrahedra.push(Tetrahedron::new(
                    triangle.u,
                    triangle.v,
                    triangle.w,
                    vertex.clone(),
                ));
            }
        }

        self.tetrahedra.retain(|tetrahedron| {
            !tetrahedron.contains_vertex(&p1)
                && !tetrahedron.contains_vertex(&p2)
                && !tetrahedron.contains_vertex(&p3)
                && !tetrahedron.contains_vertex(&p4)
        });

        let mut triangle_set = HashSet::new();
        let mut edge_set = HashSet::new();

        for tetrahedron in self.tetrahedra.iter() {
            let abc = Triangle::new(
                tetrahedron.a.clone(),
                tetrahedron.b.clone(),
                tetrahedron.c.clone(),
            );
            let abd = Triangle::new(
                tetrahedron.a.clone(),
                tetrahedron.b.clone(),
                tetrahedron.d.clone(),
            );
            let acd = Triangle::new(
                tetrahedron.a.clone(),
                tetrahedron.c.clone(),
                tetrahedron.d.clone(),
            );
            let bcd = Triangle::new(
                tetrahedron.b.clone(),
                tetrahedron.c.clone(),
                tetrahedron.d.clone(),
            );

            if triangle_set.insert(abc.clone()) {
                self.triangles.push(abc);
            }
            if triangle_set.insert(abd.clone()) {
                self.triangles.push(abd);
            }
            if triangle_set.insert(acd.clone()) {
                self.triangles.push(acd);
            }
            if triangle_set.insert(bcd.clone()) {
                self.triangles.push(bcd);
            }

            let ab = Edge::new(tetrahedron.a.clone(), tetrahedron.b.clone());
            let bc = Edge::new(tetrahedron.b.clone(), tetrahedron.c.clone());
            let ca = Edge::new(tetrahedron.c.clone(), tetrahedron.a.clone());
            let da = Edge::new(tetrahedron.d.clone(), tetrahedron.a.clone());
            let db = Edge::new(tetrahedron.d.clone(), tetrahedron.b.clone());
            let dc = Edge::new(tetrahedron.d.clone(), tetrahedron.c.clone());

            if edge_set.insert(ab.clone()) {
                self.edges.push(ab);
            }
            if edge_set.insert(bc.clone()) {
                self.edges.push(bc);
            }
            if edge_set.insert(ca.clone()) {
                self.edges.push(ca);
            }
            if edge_set.insert(da.clone()) {
                self.edges.push(da);
            }
            if edge_set.insert(db.clone()) {
                self.edges.push(db);
            }
            if edge_set.insert(dc.clone()) {
                self.edges.push(dc);
            }
        }
    }
}
