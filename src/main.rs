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

#[derive(Clone, PartialEq, Debug)]
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle() {
        let example_circle_coordinates: Vec<Vector2D> = vec![Vector2D { x: 2623, y: 359 }, Vector2D { x: 2637, y: 340 }, Vector2D { x: 2665, y: 313 }, Vector2D { x: 2688, y: 292 }, Vector2D { x: 2722, y: 267 }, Vector2D { x: 2762, y: 243 }, Vector2D { x: 2800, y: 228 }, Vector2D { x: 2849, y: 222 }, Vector2D { x: 2896, y: 227 }, Vector2D { x: 2941, y: 239 }, Vector2D { x: 2983, y: 262 }, Vector2D { x: 3015, y: 296 }, Vector2D { x: 3038, y: 335 }, Vector2D { x: 3050, y: 381 }, Vector2D { x: 3045, y: 421 }, Vector2D { x: 3031, y: 454 }, Vector2D { x: 3008, y: 498 }, Vector2D { x: 2983, y: 528 }, Vector2D { x: 2942, y: 555 }, Vector2D { x: 2904, y: 574 }, Vector2D { x: 2864, y: 586 }, Vector2D { x: 2818, y: 591 }, Vector2D { x: 2782, y: 590 }, Vector2D { x: 2752, y: 581 }, Vector2D { x: 2722, y: 568 }, Vector2D { x: 2692, y: 547 }, Vector2D { x: 2671, y: 518 }, Vector2D { x: 2658, y: 476 }, Vector2D { x: 2665, y: 427 }, Vector2D { x: 2677, y: 376 }, Vector2D { x: 2684, y: 344 }];
        let test_shape: Shape = Shape { coordinates: example_circle_coordinates, shape_type: ShapeName::Undefined };
        assert_eq!(test_shape.get_shape_name(), ShapeName::Circle);
    }

    #[test]
    fn test_ellipse() {
        let example_ellipse_coordinates: Vec<Vector2D> = vec![Vector2D { x: 2909, y: 424 }, Vector2D { x: 2886, y: 424 }, Vector2D { x: 2856, y: 423 }, Vector2D { x: 2822, y: 422 }, Vector2D { x: 2779, y: 427 }, Vector2D { x: 2697, y: 439 }, Vector2D { x: 2595, y: 461 }, Vector2D { x: 2557, y: 474 }, Vector2D { x: 2535, y: 488 }, Vector2D { x: 2522, y: 507 }, Vector2D { x: 2516, y: 529 }, Vector2D { x: 2524, y: 551 }, Vector2D { x: 2551, y: 574 }, Vector2D { x: 2638, y: 594 }, Vector2D { x: 2787, y: 602 }, Vector2D { x: 2946, y: 600 }, Vector2D { x: 3085, y: 600 }, Vector2D { x: 3253, y: 596 }, Vector2D { x: 3343, y: 580 }, Vector2D { x: 3396, y: 552 }, Vector2D { x: 3404, y: 533 }, Vector2D { x: 3405, y: 520 }, Vector2D { x: 3394, y: 504 }, Vector2D { x: 3362, y: 484 }, Vector2D { x: 3324, y: 469 }, Vector2D { x: 3281, y: 456 }, Vector2D { x: 3243, y: 446 }, Vector2D { x: 3183, y: 433 }, Vector2D { x: 3122, y: 425 }, Vector2D { x: 3033, y: 421 }, Vector2D { x: 2979, y: 421 }, Vector2D { x: 2936, y: 424 }, Vector2D { x: 2926, y: 425 }, Vector2D { x: 2923, y: 425 }, Vector2D { x: 2915, y: 425 }, Vector2D { x: 2909, y: 425 }, Vector2D { x: 2906, y: 425 }, Vector2D { x: 2903, y: 425 }, Vector2D { x: 2902, y: 425 }];
        let test_shape: Shape = Shape { coordinates: example_ellipse_coordinates, shape_type: ShapeName::Undefined };
        assert_eq!(test_shape.get_shape_name(), ShapeName::Ellipse);
    }

    #[test]
    fn test_line() {
        let example_line_coordinates: Vec<Vector2D> = vec![Vector2D { x: 3659, y: 919 }, Vector2D { x: 3655, y: 919 }, Vector2D { x: 3654, y: 919 }, Vector2D { x: 3651, y: 919 }, Vector2D { x: 3645, y: 919 }, Vector2D { x: 3625, y: 919 }, Vector2D { x: 3609, y: 920 }, Vector2D { x: 3587, y: 920 }, Vector2D { x: 3565, y: 921 }, Vector2D { x: 3538, y: 923 }, Vector2D { x: 3520, y: 924 }, Vector2D { x: 3496, y: 924 }, Vector2D { x: 3475, y: 924 }, Vector2D { x: 3448, y: 924 }, Vector2D { x: 3418, y: 922 }, Vector2D { x: 3391, y: 921 }, Vector2D { x: 3361, y: 921 }, Vector2D { x: 3333, y: 921 }, Vector2D { x: 3305, y: 921 }, Vector2D { x: 3283, y: 920 }, Vector2D { x: 3258, y: 919 }, Vector2D { x: 3237, y: 919 }, Vector2D { x: 3211, y: 918 }, Vector2D { x: 3190, y: 917 }, Vector2D { x: 3164, y: 916 }, Vector2D { x: 3138, y: 916 }, Vector2D { x: 3111, y: 918 }, Vector2D { x: 3084, y: 919 }, Vector2D { x: 3052, y: 922 }, Vector2D { x: 3029, y: 923 }, Vector2D { x: 2999, y: 924 }, Vector2D { x: 2970, y: 925 }, Vector2D { x: 2935, y: 926 }, Vector2D { x: 2911, y: 927 }, Vector2D { x: 2892, y: 927 }, Vector2D { x: 2883, y: 928 }, Vector2D { x: 2878, y: 928 }];
        let test_shape: Shape = Shape { coordinates: example_line_coordinates, shape_type: ShapeName::Undefined };
        assert_eq!(test_shape.get_shape_name(), ShapeName::Line);
    }

    #[test]
    fn test_junk() {
        let example_junk_coordinates_vec: Vec<Vec<Vector2D>> = vec![vec![Vector2D { x: 3166, y: 539 }, Vector2D { x: 3121, y: 497 }, Vector2D { x: 3030, y: 432 }, Vector2D { x: 2939, y: 397 }, Vector2D { x: 2854, y: 400 }, Vector2D { x: 2767, y: 449 }, Vector2D { x: 2712, y: 503 }, Vector2D { x: 2673, y: 554 }, Vector2D { x: 2639, y: 590 }, Vector2D { x: 2592, y: 631 }, Vector2D { x: 2543, y: 655 }, Vector2D { x: 2495, y: 669 }, Vector2D { x: 2435, y: 678 }, Vector2D { x: 2378, y: 683 }, Vector2D { x: 2358, y: 678 }, Vector2D { x: 2338, y: 648 }, Vector2D { x: 2332, y: 638 }, Vector2D { x: 2332, y: 637 }], vec![Vector2D { x: 2878, y: 667 }, Vector2D { x: 2867, y: 654 }, Vector2D { x: 2810, y: 601 }, Vector2D { x: 2780, y: 574 }, Vector2D { x: 2730, y: 520 }, Vector2D { x: 2699, y: 485 }, Vector2D { x: 2679, y: 459 }, Vector2D { x: 2676, y: 453 }, Vector2D { x: 2675, y: 453 }, Vector2D { x: 2675, y: 454 }, Vector2D { x: 2674, y: 481 }, Vector2D { x: 2667, y: 523 }, Vector2D { x: 2660, y: 563 }, Vector2D { x: 2649, y: 628 }, Vector2D { x: 2641, y: 665 }, Vector2D { x: 2637, y: 687 }, Vector2D { x: 2634, y: 701 }, Vector2D { x: 2632, y: 708 }, Vector2D { x: 2632, y: 709 }, Vector2D { x: 2626, y: 702 }, Vector2D { x: 2602, y: 664 }, Vector2D { x: 2572, y: 629 }, Vector2D { x: 2534, y: 591 }, Vector2D { x: 2493, y: 557 }, Vector2D { x: 2410, y: 508 }, Vector2D { x: 2360, y: 466 }, Vector2D { x: 2358, y: 464 }, Vector2D { x: 2359, y: 487 }, Vector2D { x: 2354, y: 560 }, Vector2D { x: 2332, y: 655 }, Vector2D { x: 2315, y: 723 }, Vector2D { x: 2304, y: 755 }, Vector2D { x: 2303, y: 758 }], vec![Vector2D { x: 2513, y: 753 }, Vector2D { x: 2515, y: 699 }, Vector2D { x: 2523, y: 589 }, Vector2D { x: 2527, y: 527 }, Vector2D { x: 2533, y: 472 }, Vector2D { x: 2534, y: 447 }, Vector2D { x: 2537, y: 431 }, Vector2D { x: 2538, y: 430 }, Vector2D { x: 2547, y: 436 }, Vector2D { x: 2580, y: 462 }, Vector2D { x: 2669, y: 532 }, Vector2D { x: 2745, y: 580 }, Vector2D { x: 2808, y: 622 }, Vector2D { x: 2837, y: 642 }, Vector2D { x: 2869, y: 663 }, Vector2D { x: 2878, y: 669 }, Vector2D { x: 2881, y: 671 }], vec![Vector2D { x: 3114, y: 661 }, Vector2D { x: 3082, y: 650 }, Vector2D { x: 3045, y: 638 }, Vector2D { x: 2985, y: 617 }, Vector2D { x: 2923, y: 595 }, Vector2D { x: 2855, y: 574 }, Vector2D { x: 2782, y: 551 }, Vector2D { x: 2743, y: 539 }, Vector2D { x: 2691, y: 521 }, Vector2D { x: 2649, y: 508 }, Vector2D { x: 2631, y: 504 }, Vector2D { x: 2632, y: 502 }, Vector2D { x: 2646, y: 491 }, Vector2D { x: 2678, y: 473 }, Vector2D { x: 2713, y: 457 }, Vector2D { x: 2757, y: 437 }, Vector2D { x: 2781, y: 425 }, Vector2D { x: 2803, y: 412 }, Vector2D { x: 2830, y: 399 }, Vector2D { x: 2878, y: 374 }, Vector2D { x: 2922, y: 350 }, Vector2D { x: 2974, y: 318 }, Vector2D { x: 3023, y: 288 }, Vector2D { x: 3051, y: 267 }, Vector2D { x: 3064, y: 258 }, Vector2D { x: 3064, y: 257 }, Vector2D { x: 3065, y: 272 }, Vector2D { x: 3076, y: 318 }, Vector2D { x: 3082, y: 351 }, Vector2D { x: 3089, y: 401 }, Vector2D { x: 3091, y: 426 }, Vector2D { x: 3094, y: 458 }, Vector2D { x: 3094, y: 492 }, Vector2D { x: 3095, y: 539 }, Vector2D { x: 3095, y: 577 }, Vector2D { x: 3095, y: 604 }, Vector2D { x: 3095, y: 618 }, Vector2D { x: 3097, y: 631 }, Vector2D { x: 3098, y: 640 }, Vector2D { x: 3099, y: 646 }], vec![Vector2D { x: 3285, y: 753 }, Vector2D { x: 3301, y: 748 }, Vector2D { x: 3308, y: 744 }, Vector2D { x: 3253, y: 742 }, Vector2D { x: 2817, y: 768 }, Vector2D { x: 2535, y: 785 }, Vector2D { x: 2748, y: 681 }, Vector2D { x: 3094, y: 563 }, Vector2D { x: 3280, y: 478 }, Vector2D { x: 3009, y: 481 }, Vector2D { x: 2599, y: 521 }, Vector2D { x: 2611, y: 498 }, Vector2D { x: 2817, y: 360 }, Vector2D { x: 3081, y: 208 }, Vector2D { x: 3099, y: 190 }, Vector2D { x: 2864, y: 240 }, Vector2D { x: 2566, y: 294 }, Vector2D { x: 2450, y: 311 }, Vector2D { x: 2449, y: 311 }], vec![Vector2D { x: 2646, y: 757 }, Vector2D { x: 2603, y: 753 }, Vector2D { x: 2568, y: 745 }, Vector2D { x: 2519, y: 737 }, Vector2D { x: 2486, y: 731 }, Vector2D { x: 2419, y: 718 }, Vector2D { x: 2376, y: 708 }, Vector2D { x: 2330, y: 690 }, Vector2D { x: 2297, y: 675 }, Vector2D { x: 2274, y: 660 }, Vector2D { x: 2258, y: 645 }, Vector2D { x: 2246, y: 627 }, Vector2D { x: 2237, y: 596 }, Vector2D { x: 2237, y: 565 }, Vector2D { x: 2251, y: 541 }, Vector2D { x: 2266, y: 529 }, Vector2D { x: 2288, y: 525 }, Vector2D { x: 2309, y: 524 }, Vector2D { x: 2348, y: 538 }, Vector2D { x: 2391, y: 553 }, Vector2D { x: 2429, y: 568 }, Vector2D { x: 2455, y: 577 }, Vector2D { x: 2487, y: 591 }, Vector2D { x: 2514, y: 599 }, Vector2D { x: 2547, y: 608 }, Vector2D { x: 2567, y: 614 }, Vector2D { x: 2586, y: 617 }, Vector2D { x: 2597, y: 617 }, Vector2D { x: 2608, y: 615 }, Vector2D { x: 2627, y: 603 }, Vector2D { x: 2658, y: 586 }, Vector2D { x: 2665, y: 577 }, Vector2D { x: 2668, y: 570 }, Vector2D { x: 2672, y: 555 }, Vector2D { x: 2671, y: 529 }, Vector2D { x: 2667, y: 478 }, Vector2D { x: 2668, y: 448 }, Vector2D { x: 2679, y: 425 }, Vector2D { x: 2694, y: 412 }, Vector2D { x: 2738, y: 398 }, Vector2D { x: 2776, y: 395 }, Vector2D { x: 2791, y: 406 }, Vector2D { x: 2793, y: 430 }, Vector2D { x: 2775, y: 480 }, Vector2D { x: 2740, y: 556 }, Vector2D { x: 2724, y: 597 }, Vector2D { x: 2712, y: 624 }, Vector2D { x: 2706, y: 640 }, Vector2D { x: 2703, y: 655 }, Vector2D { x: 2714, y: 676 }, Vector2D { x: 2731, y: 688 }, Vector2D { x: 2753, y: 696 }, Vector2D { x: 2789, y: 709 }, Vector2D { x: 2831, y: 717 }, Vector2D { x: 2888, y: 727 }, Vector2D { x: 2940, y: 734 }, Vector2D { x: 3019, y: 747 }, Vector2D { x: 3083, y: 765 }, Vector2D { x: 3169, y: 799 }, Vector2D { x: 3201, y: 825 }, Vector2D { x: 3213, y: 846 }, Vector2D { x: 3208, y: 861 }, Vector2D { x: 3193, y: 874 }, Vector2D { x: 3120, y: 886 }, Vector2D { x: 2991, y: 878 }, Vector2D { x: 2862, y: 854 }, Vector2D { x: 2807, y: 835 }, Vector2D { x: 2738, y: 809 }, Vector2D { x: 2682, y: 790 }, Vector2D { x: 2601, y: 760 }, Vector2D { x: 2567, y: 750 }], vec![Vector2D { x: 2980, y: 421 }, Vector2D { x: 2931, y: 415 }, Vector2D { x: 2871, y: 426 }, Vector2D { x: 2815, y: 455 }, Vector2D { x: 2767, y: 505 }, Vector2D { x: 2738, y: 569 }, Vector2D { x: 2733, y: 633 }, Vector2D { x: 2748, y: 687 }, Vector2D { x: 2794, y: 740 }, Vector2D { x: 2859, y: 793 }, Vector2D { x: 2943, y: 821 }, Vector2D { x: 3051, y: 814 }, Vector2D { x: 3138, y: 782 }, Vector2D { x: 3192, y: 753 }, Vector2D { x: 3234, y: 715 }, Vector2D { x: 3259, y: 658 }, Vector2D { x: 3262, y: 605 }, Vector2D { x: 3233, y: 556 }, Vector2D { x: 3194, y: 513 }, Vector2D { x: 3129, y: 477 }, Vector2D { x: 3048, y: 454 }, Vector2D { x: 2962, y: 442 }, Vector2D { x: 2882, y: 435 }, Vector2D { x: 2813, y: 428 }, Vector2D { x: 2757, y: 423 }, Vector2D { x: 2701, y: 420 }, Vector2D { x: 2662, y: 420 }, Vector2D { x: 2634, y: 424 }, Vector2D { x: 2626, y: 425 }, Vector2D { x: 2625, y: 425 }, Vector2D { x: 2625, y: 426 }]];
        for junk_coordinates in example_junk_coordinates_vec {
            let test_shape: Shape = Shape { coordinates: junk_coordinates, shape_type: ShapeName::Undefined };
            assert_eq!(test_shape.get_shape_name(), ShapeName::Unknown);
        }
    }
}