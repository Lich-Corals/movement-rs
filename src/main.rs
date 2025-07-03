use mouse_position::mouse_position::{Mouse};
use std::{thread, time};

const END_FIGURE_TIMEOUT: u8 = 5;

fn get_mouse_position() -> Coordinate {
    let position = Mouse::get_mouse_position();
    match position {
        Mouse::Position { x, y } => Coordinate { x: x, y: y },
        Mouse::Error => Coordinate { x: 0, y: 0 },
    }
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
    fn distance(self, other: Coordinate) -> f32 {
        (((self.x-other.x).pow(2)+(self.y-other.y).pow(2)) as f32).sqrt()
    }
}

impl Shape {
    fn get_shape_name(&mut self) -> ShapeName {
        ShapeName::Unknown
    }

    fn find_center(self) -> Coordinate {
        fn summation(start: i32, end: i32) -> Result {
            if start <= end {
                let mut sum: i32 = 0;
                let mut i: i32 = start.clone();
                while i <= end {
                    sum += 3*i-1;
                    i += 1;
                }
                Result {value: sum, ok: true}
            } else {
                Result {value: 0, ok: false}
            }
        }

        let mut a: Vec<Coordinate> = Vec::new();
        let mut b: Vec<Coordinate> = Vec::new();
        let mut c: Vec<Coordinate> = Vec::new();
        let mut n: u8 = 1;
        for coordinate in self.coordinates.clone() {
            if a.len()+b.len()+c.len()+3 <= self.coordinates.len() {
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

        

        struct Result {
            value: i32,
            ok: bool,
        }

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

