#[cfg(feature = "glium")]
#[macro_use]
extern crate glium;

use std::f32::consts::PI;
use std::iter;

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[cfg(feature = "glium")]
implement_vertex!(Point, x, y);

impl Point {
    pub fn midpoint(&self, rhs: &Point) -> Point {
        Point {
            x: (self.x + rhs.x) / 2.0,
            y: (self.y + rhs.y) / 2.0,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct Attractor {
    pub attractor: [f32; 2],
}

#[cfg(feature = "glium")]
implement_vertex!(Attractor, attractor);

impl From<Point> for Attractor {
    fn from(p: Point) -> Attractor {
        Attractor {
            attractor: [p.x, p.y],
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct Kind {
    pub kind: u32,
}

#[cfg(feature = "glium")]
implement_vertex!(Kind, kind);

#[derive(Copy, Clone, Debug)]
pub struct EvenqCoordinate {
    x: usize,
    y: usize,
}

impl EvenqCoordinate {
    fn center(&self, size: f32) -> Point {
        let x = size * (3.0 / 2.0) * (self.x as f32);
        let y = size * 3f32.sqrt() * ((self.y as f32) - 0.5 * ((self.x & 1) as f32));
        Point { x, y }
    }
}

fn coordinates(rows: usize, columns: usize) -> impl Iterator<Item = EvenqCoordinate> {
    (0..rows).flat_map(move |row| (0..columns).map(move |col| EvenqCoordinate { x: row, y: col }))
}

pub struct Hexagon {
    pub center: Point,
    pub corners: [Point; 6],
    pub midpoints: [Point; 12],
}

/// A corner point that's pulled by the center attractor into the inside of the tile
const INTERNAL: Kind = Kind { kind: 0 };
/// The midpoint of the hexagon edge, possibly pulled by the corner attractor
const MID: Kind = Kind { kind: 1 };
/// A fixed corner on the hexagon (ignored attractor)
const CORNER: Kind = Kind { kind: 2 };


fn flat_hex_corner(center: Point, size: f32, i: usize) -> Point {
    let angle_deg = 60.0 * (i as f32);
    let angle_rad = PI / 180.0 * angle_deg;
    Point {
        x: center.x + size * angle_rad.cos(),
        y: center.y + size * angle_rad.sin(),
    }
}

impl Hexagon {
    fn corners(center: Point, size: f32) -> [Point; 6] {
        let mut corners: [Point; 6] = Default::default();
        for i in 0..6 {
            corners[i] = flat_hex_corner(center, size, i);
        }
        corners
    }

    fn midpoints(corners: &[Point; 6]) -> [Point; 12] {
        let mut midpoints: [Point; 12] = Default::default();
        for i in 0..6 {
            midpoints[2 * i] = corners[i].midpoint(&corners[(i + 1) % corners.len()]);
            midpoints[2 * i + 1] = corners[i].midpoint(&corners[(i + 1) % corners.len()]);
        }
        midpoints
    }

    pub fn new(center: Point, size: f32) -> Hexagon {
        let corners = Self::corners(center, size);
        let midpoints = Self::midpoints(&corners);
        Hexagon {
            center,
            corners,
            midpoints,
        }
    }

    fn add_to_mesh(
        self,
        points: &mut Vec<Point>,
        lines: &mut Vec<u16>,
        triangles: &mut Vec<u32>,
        kinds: &mut Vec<Kind>,
        attractors: &mut Vec<Attractor>,
    ) {
        points.push(self.center);
        // These two are unused for the center point, since it doesn't show up
        // in any lines. However, to avoid offset errors, there must be
        // something here to avoid offset errors in the parallel vecs.
        attractors.push(self.center.into());
        kinds.push(INTERNAL);

        let internals_idx = points.len();
        points.extend(self.corners.iter().cloned());
        kinds.extend(iter::repeat(INTERNAL).take(self.corners.len()));
        attractors.extend(iter::repeat::<Attractor>(self.center.into()).take(self.corners.len()));
        assert_eq!(points.len(), kinds.len());
        assert_eq!(points.len(), attractors.len());

        let midpoints_idx = points.len();
        points.extend(self.midpoints.iter().cloned());
        kinds.extend(iter::repeat(MID).take(self.midpoints.len()));
        for i in 0..(self.midpoints.len() / 2) {
            attractors.push(self.corners[i].into());
            attractors.push(self.corners[(i + 1) % self.corners.len()].into());
        }
        assert_eq!(points.len(), kinds.len());
        assert_eq!(points.len(), attractors.len());

        let corners_idx = points.len();
        points.extend(self.corners.iter().cloned());
        kinds.extend(iter::repeat(CORNER).take(self.corners.len()));
        attractors.extend(iter::repeat::<Attractor>(self.center.into()).take(self.corners.len())); // ignored
        assert_eq!(points.len(), kinds.len());
        assert_eq!(points.len(), attractors.len());

        for i in 0..6 {
            // Internal to first midpoint.
            lines.push((internals_idx + i) as u16);
            lines.push((midpoints_idx + (i * 2 + 1) % self.midpoints.len()) as u16);

            // Other internal to second midpoint.
            lines.push((internals_idx + (i + 1) % self.corners.len()) as u16);
            lines.push((midpoints_idx + (i * 2)) as u16);

            // Internal-midpoint-corner
            triangles.push((internals_idx + i) as u32);
            triangles.push((midpoints_idx + (i * 2 + 1) % self.midpoints.len()) as u32);
            triangles.push((corners_idx + i) as u32);

            // Internal-second midpoint-second corner
            triangles.push((internals_idx + (i + 1) % self.corners.len()) as u32);
            triangles.push((midpoints_idx + (i * 2)) as u32);
            triangles.push((corners_idx + (i + 1) % self.corners.len()) as u32);
        }
    }
}

pub fn hexagons(rows: usize, columns: usize, size: f32) -> impl Iterator<Item = Hexagon> {
    coordinates(rows, columns)
        .map(move |coord| coord.center(size))
        .map(move |center| Hexagon::new(center, size))
}

pub fn mesh(rows: usize, columns: usize, size: f32) -> (Vec<Point>, Vec<u16>, Vec<u32>, Vec<Attractor>, Vec<Kind>) {
    hexagons(rows, columns, size)
        .map(|mut hex| {
            hex.center.x -= 1.0;
            hex.center.y -= 1.0;
            for p in &mut hex.corners {
                p.x -= 1.0;
                p.y -= 1.0;
            }
            for p in &mut hex.midpoints {
                p.x -= 1.0;
                p.y -= 1.0;
            }
            hex
        })
        .fold(
            (vec![], vec![], vec![], vec![], vec![]),
            |(mut points, mut lines, mut triangles, mut attractors, mut kinds), hex| {
                hex.add_to_mesh(&mut points, &mut lines, &mut triangles, &mut kinds, &mut attractors);

                (points, lines, triangles, attractors, kinds)
            },
        )
}

pub const VERTEX_SHADER: &str = "
                #version 140

                in float x;
                in float y;

                in vec2 attractor;
                in uint kind;

                uniform float a;
                uniform float b;
                uniform vec2 offset;

                void main() {
                    float multiplier = kind == uint(2) ? 0 : (kind == uint(0) ? b : a);

                    vec2 p = vec2(x, y) + offset;
                    vec2 v = attractor + offset - p;
                    gl_Position = vec4(p + multiplier * v, 0.0, 1.0);
                }
";

pub const FRAGMENT_SHADER: &str = "
                #version 140

                uniform vec3 color;

                out vec4 f_color;

                void main() {
                    f_color = vec4(color, 1.0);
                }
";

pub const VERTEX_SHADER_WEB: &str = "
                attribute vec2 position;
                attribute vec2 attractor;
                attribute float kind;
                uniform float a;
                uniform float b;

                void main() {
                    float multiplier = kind < 0.5 ? b : a;

                    vec2 p = position;
                    vec2 v = attractor - p;
                    gl_Position = vec4(p + multiplier * v, 0.0, 1.0);
                }
";

pub const FRAGMENT_SHADER_WEB: &str = "
                void main() {
                    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
                }
";

