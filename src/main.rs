use mouse_position::mouse_position::{Mouse};
use std::{thread, time};

const END_FIGURE_TIMEOUT: u8 = 5;

#[derive(Clone, Default, PartialEq)]
struct Coordinate {
    x: i32,
    y: i32,
}

fn get_mouse_position() -> Coordinate {
    let position = Mouse::get_mouse_position();
    match position {
        Mouse::Position { x, y } => Coordinate { x: x, y: y },
        Mouse::Error => Coordinate { x: 0, y: 0 },
    }
}

#[derive(Clone, Default)]
struct Recording {
    coordinates: Vec<Coordinate>,
    initialized: bool,
    running: bool,
    stop_coordinate: Coordinate,
    coordinate_unchanged_cycles: u8,
    length: u32,
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
}

#[derive(Clone, PartialEq)]
struct Shape {
    coordinates: Vec<Coordinate>,
    shape_type: ShapeName,
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
        self.length = 0;
        
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
            self.length += 1;
            self.stop_coordinate = current_mouse_coordinate;
            if self.length == 1 {
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

impl Shape {
    fn get_shape_name(&mut self) -> ShapeName {
        ShapeName::Unknown
    }

    fn find_centroid(&mut self) -> Coordinate {
        Coordinate { x: 0, y: 0 }
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
            thread::sleep(time::Duration::from_millis(100));
        }
        shape_collection.push(Shape { coordinates: recording.coordinates.clone(), shape_type: ShapeName::Unknown });
        for shape in &mut shape_collection {
            if shape.shape_type == ShapeName::Unknown {
                shape.get_shape_name();
            }
        }
    }
}

