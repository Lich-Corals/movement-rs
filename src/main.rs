use mouse_position::mouse_position::{Mouse};
use std::path::absolute;
use std::{thread, time};
use std::ops::{Sub, Add};

const END_FIGURE_TIMEOUT: u8 = 5;
const FRAMERATE_FPS: u64 = 20;
const CIRCLE_TOLERANCE: f32 = 0.25;
const TOLERANCE_GENERAL: f32 = 0.25;
const LINE_TOLERANCE_PX: f32 = 15.0;

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

#[derive(Clone, Default, PartialEq, Copy)]
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
    Line,
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

    fn abs(&self) -> f32 {
        (((self.x.pow(2))+(self.y.pow(2))) as f32 ).sqrt()
    }

    fn distance_line(self, b: Coordinate, c: Coordinate) -> f32 {
        let a: Coordinate = self;
        let vector_ab: Coordinate = a-b;
        let vector_ac: Coordinate = a-c;
        ((vector_ab.x*vector_ac.y-vector_ab.y*vector_ac.x) as f32).abs()/(b-c).abs() as f32
    }
}

impl Sub for Coordinate {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Add for Coordinate {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Shape {
    fn get_shape_name(&self) -> ShapeName {
        
        let passes_percent_circle: i32 = self.get_center_distances().passes_percent;
        let passed_percent_line: f32;
        let max_distance: i32 = self.get_distances().max;
        let start_end_distance: i32 = self.coordinates[0].distance(&self.coordinates[self.coordinates.len()-1]);
        if self.get_center_distances().passes_percent >= 100 - (TOLERANCE_GENERAL * 100.0) as i32 {
            println!("CIRCLE ({}%)", passes_percent_circle);
            ShapeName::Circle
        } else if max_distance == start_end_distance {
            let zero: Coordinate = self.coordinates[0];
            let zfro: Coordinate = self.coordinates[self.coordinates.len()-1];
            let mut passed_coordinates: Vec<&Coordinate> = Vec::new();
            for coordinate in &self.coordinates {
                let distance: f32 = coordinate.distance_line(self.coordinates[0], self.coordinates[self.coordinates.len()-1]);
                if distance <= LINE_TOLERANCE_PX {
                    passed_coordinates.push(coordinate);
                }
            }
            passed_percent_line = (passed_coordinates.len() as f32) / (self.coordinates.len() as f32) * 100.0;
            if passed_percent_line >= 100.0 - (100.0 * TOLERANCE_GENERAL) {
                println!("LINE ({}%)", passed_percent_line as i32);
                ShapeName::Line
            } else {
                println!("UNKNOWN ({}% Line)", passed_percent_line as i32);
                ShapeName::Unknown
            }
        } else {
            println!("UNKNOWN ({}% Circle)", passes_percent_circle);
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
        let absolute_tolerance: i32 = (average as f32 * CIRCLE_TOLERANCE) as i32;
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

