use mouse_position::mouse_position::{Mouse};
use std::{arch::x86_64, thread, time};

const END_FIGURE_TIMEOUT: u8 = 5;
const FRAMERATE_FPS: u64 = 20;
const TOLERANCE: f32 = 0.3;

fn get_mouse_position() -> Coordinate {
    let position = Mouse::get_mouse_position();
    match position {
        Mouse::Position { x, y } => Coordinate { x: x, y: y },
        Mouse::Error => Coordinate { x: 0, y: 0 },
    }
}

fn divide_into_triangles(points: &Vec<Coordinate>) -> Triangles {
    let mut a: Vec<Coordinate> = Vec::new();
    let mut b: Vec<Coordinate> = Vec::new();
    let mut c: Vec<Coordinate> = Vec::new();
    let mut n: u8 = 1;
    for coordinate in points.clone() {
        if a.len()+b.len()+c.len()+3 <= points.len() {
            match n {
                1 => a.push(coordinate),
                2 => b.push(coordinate),
                3 => c.push(coordinate),
                _ => ()
            }
            n += 1;
            if n > 3 {
                n = 1;
            }
        }
    }
    Triangles { a: a, b: b, c: c }   
}

fn centroids_of_triangles(triangles: Triangles) -> Vec<Coordinate> {
    let mut results: Vec<Coordinate> = Vec::new();
    for i in 0..triangles.a.len() {
        let x = triangles.a[i].x;
        let y = triangles.a[i].y;
        results.push(Coordinate { x: x, y: y });
    }
    results
}

#[derive(Clone, Default, PartialEq)]
struct Coordinate {
    x: i32,
    y: i32,
}

#[derive(Clone, Default)]
struct Recording {
    coordinates: Vec<Coordinate>,
    initialized: bool,
    running: bool,
    stop_coordinate: Coordinate,
    coordinate_unchanged_cycles: u8,
}

struct Triangles {
        a: Vec<Coordinate>,
        b: Vec<Coordinate>,
        c: Vec<Coordinate>,
    }

enum RecordingStatus {
    Waiting,
    Running,
    Finished,
}

#[derive(Clone, PartialEq)]
enum ShapeName {
    Circle,
    Ellipse,
    Unknown,
    Undefined,
}

#[derive(Clone, PartialEq)]
struct Shape {
    coordinates: Vec<Coordinate>,
    shape_type: ShapeName,
    distances: DistanceSet,
}

#[derive(Clone, PartialEq)]
struct DistanceSet {
    min: i32,
    max: i32,
    minimax: i32,
}

#[derive(Clone, PartialEq)]
struct CenterDistanceSet {
    min: i32,
    max: i32,
    avg: i32,
    above: i32,
    below: i32,
    values: i32,
    passes_percent: i32,
}

impl Recording {
    fn clear(&mut self) {
        self.coordinates = Vec::new();
    }

    fn init(&mut self) {
        self.initialized = true;
        self.running = false;
        self.stop_coordinate = get_mouse_position();
        self.coordinate_unchanged_cycles = 0;
        
    }

    fn update(&mut self) -> RecordingStatus {
        if !self.initialized {
            self.init();
            println!("Initialized recording.");
        }
        let current_mouse_coordinate: Coordinate = get_mouse_position();
        if !(self.stop_coordinate == current_mouse_coordinate) {
            self.running = true;
            self.coordinates.push(current_mouse_coordinate.clone());
            self.stop_coordinate = current_mouse_coordinate;
            if self.coordinates.len() == 0 {
                println!("Recording started.");
            }
            RecordingStatus::Running
        } else {
            if self.running {
                self.coordinate_unchanged_cycles += 1;
                if self.coordinate_unchanged_cycles >= END_FIGURE_TIMEOUT {
                    self.initialized = false;
                    self.stop_coordinate = get_mouse_position();
                    println!("Recording finished.");
                    RecordingStatus::Finished
                } else {
                    RecordingStatus::Running
                }
            } else {
                RecordingStatus::Waiting
            }
        }
    }
}

impl Coordinate {
    fn distance(&self, other: &Coordinate) -> i32 {
        let mut distance = (((self.x-other.x).pow(2)+(self.y-other.y).pow(2)) as f32).sqrt() as i32;
        distance = distance;
        distance
    }
}

impl Shape {
    fn get_shape_name(&self) -> ShapeName {
        
        let passes: i32 = self.get_center_distances().passes_percent;
        if self.get_center_distances().passes_percent >= 100 - (TOLERANCE * 100.0) as i32 {
            println!("CIRCLE");
            ShapeName::Circle
        } else {
            println!("UNKNOWN");
            ShapeName::Unknown
        }
    }

    fn find_center(&self) -> Coordinate {
        let mut average_coordinate: Coordinate = Coordinate { x: 0, y: 0 };
        for coordinate in &self.coordinates {
            average_coordinate.x += coordinate.x;
            average_coordinate.y += coordinate.y;
        }
        average_coordinate.x = average_coordinate.x / self.coordinates.len() as i32;
        average_coordinate.y = average_coordinate.y / self.coordinates.len() as i32;
        average_coordinate
    }

    fn get_distances(&self) -> DistanceSet {
        let mut max_distance: i32 = 0;
        let mut min_distance: i32 = i32::MAX;
        for point in &self.coordinates {
            for other in &self.coordinates {
                if point != other {
                    let new_distance: i32 = point.distance(other);
                    if new_distance > max_distance {
                        max_distance = new_distance;
                    } else if new_distance < min_distance {
                        min_distance = new_distance;
                    }
                }
            }
        }
        DistanceSet { min: min_distance, max: max_distance, minimax: (min_distance+max_distance)/2 }
    }

    fn get_center_distances(&self) -> CenterDistanceSet {
        let center: Coordinate = self.find_center();
        let mut max_distance: i32 = 0;
        let mut min_distance: i32 = i32::MAX;
        let mut average: i32 = 0;
        let mut distances: Vec<i32> = Vec::new();
        for other in &self.coordinates {
            if center != *other {
                let new_distance: i32 = center.distance(other);
                average += new_distance;
                distances.push(new_distance);
                if new_distance > max_distance {
                    max_distance = new_distance;
                } if new_distance < min_distance {
                    min_distance = new_distance;
                }
            }
        }
        average = average / self.coordinates.len() as i32;
        let absolute_tolerance: i32 = (average as f32 * TOLERANCE) as i32;
        let mut above: i32 = 0;
        let mut below: i32 = 0;
        for distance in &distances {
            if distance - absolute_tolerance > average {
                above += 1;
            }
            if distance + absolute_tolerance < average {
                below += 1;
            }
        }
        let passed: i32 = self.coordinates.len() as i32 - (above + below);
        CenterDistanceSet { min: min_distance, max: max_distance, avg: average, above: above, below: below, values: self.coordinates.len() as i32, passes_percent: ((passed as f32) / (self.coordinates.len() as f32) * 100.0) as i32}
    }
}

fn main() {
    let mut recording: Recording = Recording::default();
    let mut shape_collection: Vec<Shape> = Vec::new();
    loop {
        loop {
            match recording.update() {
                RecordingStatus::Finished => break,
                _ => (),
            }
            thread::sleep(time::Duration::from_millis(1000/FRAMERATE_FPS));
        }
        shape_collection.push(Shape { coordinates: recording.coordinates.clone(), shape_type: ShapeName::Undefined, distances: DistanceSet {min: 0, max: 0, minimax: 0}});
        for shape in &mut shape_collection {
            if shape.shape_type == ShapeName::Undefined {
                shape.shape_type = shape.get_shape_name();
                shape.distances = shape.get_distances();
            }
        }
        recording = Recording::default();
    }
}

