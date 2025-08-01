// Movement-RS - More or less accurate shape recognition 
// Copyright (C) 2025  Linus Tibert
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use mouse_position::mouse_position::{Mouse};
use std::{thread, time};
use std::ops::{Sub, Add, Div, Mul};

const END_FIGURE_TIMEOUT: u8 = 5;
const FRAMERATE_FPS: u64 = 20;
const TOLERANCE_GENERAL: f32 = 0.25;
const CIRCLE_TOLERANCE: f32 = 0.25;
const LINE_TOLERANCE_PX: f32 = 10.0;
const ELLIPSE_CENTRUM_TOLERANCE_PX: i32 = 100;
const ELLIPSE_TOLERANCE: f32 = 0.5;

fn get_mouse_position() -> Vector2D {
    let position = Mouse::get_mouse_position();
    match position {
        Mouse::Position { x, y } => Vector2D { x: x, y: y },
        Mouse::Error => Vector2D { x: 0, y: 0 },
    }
}

#[derive(Clone, Default, PartialEq, Copy)]
struct Vector2D {
    x: i32,
    y: i32,
}

#[derive(Clone, Default)]
struct Recording {
    coordinates: Vec<Vector2D>,
    initialized: bool,
    running: bool,
    stop_coordinate: Vector2D,
    coordinate_unchanged_cycles: u8,
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
    coordinates: Vec<Vector2D>,
    shape_type: ShapeName,
 }

#[derive(Clone, PartialEq)]
struct DistanceSet {
    min: i32,
    max: i32,
    max_pair: [Vector2D; 2],
    min_pair: [Vector2D; 2],
}

#[derive(Clone, PartialEq)]
struct PointDistanceSet {
    min: i32,
    max: i32,
    avg: i32,
    above: i32,
    below: i32,
    values: i32,
    passes_percent: i32,
    max_pair: [Vector2D; 2],
    min_pair: [Vector2D; 2],
}

impl Recording {
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
        let current_mouse_coordinate: Vector2D = get_mouse_position();
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

impl Vector2D {
    fn abs(&self) -> f32 {
        (((self.x.pow(2))+(self.y.pow(2))) as f32 ).sqrt()
    }

    fn cross(self, other: Vector2D) -> i32 {
        self.x * other.y - self.y * other.x
    }

    fn distance(&self, other: &Vector2D) -> i32 {
        let mut distance = (((self.x-other.x).pow(2)+(self.y-other.y).pow(2)) as f32).sqrt() as i32;
        distance = distance;
        distance
    }

    fn distance_line(self, b: Vector2D, c: Vector2D) -> f32 {
        let a: Vector2D = self;
        let vector_ab: Vector2D = a-b;
        let vector_ac: Vector2D = a-c;
        (vector_ab.cross(vector_ac) as f32)/(b-c).abs() as f32
    }
}

impl Sub for Vector2D {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Add for Vector2D {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Div<i32> for Vector2D {
    type Output = Self;

    fn div(self, other: i32) -> Self::Output {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl Mul<i32> for Vector2D {
    type Output = Self;

    fn mul(self, other: i32) -> Self::Output {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Mul for Vector2D {
    type Output = i32;

    fn mul(self, other: Self) -> Self::Output {
        self.x * other.x + self.y * self.y
    }

}

impl Shape {
    fn get_shape_name(&self) -> ShapeName {
        let passes_percent_circle: i32 = self.get_point_distances(self.find_center()).passes_percent;
        let passed_percent_line: f32;
        let max_distance: i32 = self.get_distances().max;
        let start_end_distance: i32 = self.coordinates[0].distance(&self.coordinates[self.coordinates.len()-1]);
        if self.get_point_distances(self.find_center()).passes_percent >= 100 - (TOLERANCE_GENERAL * 100.0) as i32 {
            println!("CIRCLE ({}%)", passes_percent_circle);
            ShapeName::Circle
        } else if max_distance == start_end_distance {
            let mut passed_coordinates: Vec<&Vector2D> = Vec::new();
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
            let max_pair: [Vector2D; 2] = self.get_distances().max_pair;
            let longest_vector: Vector2D = max_pair[0] + max_pair[1];
            let vector_centrum: Vector2D = longest_vector / 2;
            let calculated_centrum: Vector2D = self.find_center();
            if (calculated_centrum - vector_centrum).abs() as i32 <= ELLIPSE_CENTRUM_TOLERANCE_PX {
                let check_point_amount: i32 = (self.coordinates.len()/2) as i32;
                let mut last_distance: f32 = f32::MAX;
                let mut grow: f32 = 0.0;
                let mut shrink: f32 = 0.0;
                let check_vectors: [Vector2D; 2] = [(calculated_centrum - max_pair[1]) / check_point_amount, (calculated_centrum - max_pair[0]) / check_point_amount];
                let mut distance_errors: f32 = 0.0;
                let mut distance_passed: f32 = 0.0;
                let mut current_check_vector: Vector2D;
                for ii in 0..1 {
                    for i in 1..check_point_amount {
                        current_check_vector = vector_centrum + (check_vectors[ii] * i);
                        let distance_min: f32 = self.get_closest_to_point(current_check_vector).0.distance_line(max_pair[0], max_pair[1]).abs();
                        let point_min: Vector2D = self.get_closest_to_point(current_check_vector).0;
                        let mirrored_min: Vector2D = current_check_vector + (current_check_vector - point_min) * 2;
                        let mirrored_min_distance: f32 = self.get_closest_to_point(mirrored_min).0.distance_line(max_pair[0], max_pair[1]).abs();
                        if mirrored_min_distance - ELLIPSE_TOLERANCE * distance_min > distance_min || mirrored_min_distance + ELLIPSE_TOLERANCE * distance_min < distance_min {
                            distance_errors += 1.0;
                        } else {
                            distance_passed += 1.0;
                        }
                        if distance_min > last_distance {
                            
                            grow += 1.0;
                        } else {
                            shrink += 1.0;
                        }
                        last_distance = self.get_point_distances(check_vectors[ii] * i).min as f32;
                    }
                }
                let grow_factor: f32 = grow / (shrink + grow);
                let distance_error_factor: f32 = distance_errors / (distance_passed + distance_errors);
                let perfection: f32 = (grow_factor + distance_error_factor) / 2.0;
                if perfection > TOLERANCE_GENERAL {
                    println!("UNKNOWN ({}% Ellipse)", ((1.0 - perfection) * 100.0) as i32);
                    ShapeName::Unknown
                } else {
                    println!("ELLIPSE ({}%)", ((1.0 - perfection) * 100.0) as i32);
                    ShapeName::Ellipse
                }
            } else {
                println!("UNKNOWN ({}% Circle)", passes_percent_circle);
                ShapeName::Unknown
            }
        }
    }

    fn find_center(&self) -> Vector2D {
        let mut average_coordinate: Vector2D = Vector2D { x: 0, y: 0 };
        for coordinate in &self.coordinates {
            average_coordinate.x += coordinate.x;
            average_coordinate.y += coordinate.y;
        }
        average_coordinate.x = average_coordinate.x / self.coordinates.len() as i32;
        average_coordinate.y = average_coordinate.y / self.coordinates.len() as i32;
        average_coordinate
    }

    fn get_closest_to_point(&self, point: Vector2D) -> (Vector2D, i32) {
        let mut min_distance: i32 = i32::MAX;
        let mut min_distance_point: Vector2D = Vector2D::default();
        for other in &self.coordinates {
            if other != &point {
                if point.distance(other) < min_distance {
                    min_distance = point.distance(other);
                    min_distance_point = *other;
                }
            }
        }
        (min_distance_point, min_distance)
    }

    fn get_distances(&self) -> DistanceSet {
        let mut max_distance: i32 = 0;
        let mut min_distance: i32 = i32::MAX;
        let mut max_pair: [Vector2D; 2] = [Vector2D::default(), Vector2D::default()];
        let mut min_pair:[Vector2D; 2] = max_pair.clone();
        for point in &self.coordinates {
            for other in &self.coordinates {
                if point != other {
                    let new_distance: i32 = point.distance(other);
                    if new_distance > max_distance {
                        max_distance = new_distance;
                        max_pair = [*point, *other];
                    } else if new_distance < min_distance {
                        min_distance = new_distance;
                        min_pair = [*point, *other];
                    }
                }
            }
        }
        DistanceSet { min: min_distance, max: max_distance, max_pair: max_pair, min_pair: min_pair }
    }

    fn get_point_distances(&self, point: Vector2D) -> PointDistanceSet {
        let mut max_distance: i32 = 0;
        let mut min_distance: i32 = i32::MAX;
        let mut average: i32 = 0;
        let mut distances: Vec<i32> = Vec::new();
        let mut max_pair: [Vector2D; 2] = [Vector2D::default(), Vector2D::default()];
        let mut min_pair:[Vector2D; 2] = max_pair.clone();
        for other in &self.coordinates {
            if point != *other {
                let new_distance: i32 = point.distance(other);
                average += new_distance;
                distances.push(new_distance);
                if new_distance > max_distance {
                    max_distance = new_distance;
                    max_pair = [point, *other];
                } if new_distance < min_distance {
                    min_distance = new_distance;
                    min_pair = [point, *other];
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
        PointDistanceSet { min: min_distance, max: max_distance, avg: average, above: above, below: below, values: self.coordinates.len() as i32, passes_percent: ((passed as f32) / (self.coordinates.len() as f32) * 100.0) as i32, max_pair: max_pair, min_pair: min_pair}
    }
}

fn main() {
    println!("                   .-'''-.                                                                                        ");
    println!("                  '   _    \\                                                                                      ");
    println!(" __  __   ___   /   /` '.   .----.     .----.  __.....__     __  __   ___        __.....__       _..._            ");
    println!("|  |/  `.'   `..   |     \\  '\\    \\   /    .-''         '.  |  |/  `.'   `.  .-''         '.   .'     '.          ");
    println!("|   .-.  .-.   |   '      |  ''   '. /'   /     .-''''-.  `.|   .-.  .-.   '/     .-''''-.  `..   .-.   .    .|   ");
    println!("|  |  |  |  |  \\    \\     / / |    |'    /     /________\\   |  |  |  |  |  /     /________\\   |  '   '  |  .' |_  ");
    println!("|  |  |  |  |  |`.   ` ..' /  |    ||    |                  |  |  |  |  |  |                  |  |   |  |.'     | ");
    println!("|  |  |  |  |  |   '-...-'`   '.   `'   .\\    .-------------|  |  |  |  |  \\    .-------------|  |   |  '--.  .-' ");
    println!("|  |  |  |  |  |               \\        / \\    '-.____...---|  |  |  |  |  |\\    '-.____...---|  |   |  |  |  |   ");
    println!("|__|  |__|  |__|                \\      /   `.             .'|__|  |__|  |__| `.             .'|  |   |  |  |  |   ");
    println!("                                 '----'      `''-...... -'                     `''-...... -'  |  |   |  |  |  '.' ");
    println!("                                                                                              |  |   |  |  |   /  ");
    println!("More or less accurate shape recognition                                                       '--'   '--'  `'-'   ");
    println!("");
    println!("Movement-RS  Copyright (C) 2025  Linus Tibert\nThis program comes with ABSOLUTELY NO WARRANTY.\nThis is free software, and you are welcome to redistribute it\nunder certain conditions.\nView https://github.com/Lich-Corals/movement-rs/blob/main/LICENSE for more information.\n");
    println!("Move your cursor to start a recording. Stop moving to evaluate.\n");
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
        shape_collection.push(Shape { coordinates: recording.coordinates.clone(), shape_type: ShapeName::Undefined});
        for shape in &mut shape_collection {
            if shape.shape_type == ShapeName::Undefined {
                shape.shape_type = shape.get_shape_name();
            }
        }
        recording = Recording::default();
    }
}

