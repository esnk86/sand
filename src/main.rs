use minifb::{Key, Window, WindowOptions};

use std::collections::HashMap;

const UNIT_WIDTH: usize = 64;
const UNITS_PER_ROW: usize = 9;
const WINDOW_WIDTH: usize = UNIT_WIDTH * UNITS_PER_ROW;

#[derive(Clone, Copy)]
enum Unit {
    Air,
    Rock,
    Sand,
}

struct Slice<'a> {
    slice: HashMap<usize, HashMap<usize, Unit>>,
    bottom: usize,
    buffer: Vec<u32>,
    window: &'a mut Window,
}

impl<'a> Slice<'a> {
    fn new(window: &'a mut Window) -> Self {
        let slice = HashMap::new();
        let bottom = 0;
        let buffer = vec![0; WINDOW_WIDTH * WINDOW_WIDTH];

        Self {
            slice,
            bottom,
            buffer,
            window,
        }
    }

    fn centre(&self) -> usize {
        f32::ceil(UNITS_PER_ROW as f32 / 2.0) as usize - 1
    }

    fn run(&mut self) {
        let mut row = HashMap::new();

        row.insert(0, Unit::Rock);
        row.insert(self.centre(), Unit::Sand);

        self.slice.insert(0, row);

        for y in 0 .. UNITS_PER_ROW {
            for x in 0 .. UNITS_PER_ROW {
                self.put_unit(x, y);
            }
        }

        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            self.window
                .update_with_buffer(&self.buffer, WINDOW_WIDTH, WINDOW_WIDTH)
                .unwrap();
        }
    }

    fn get_unit(&self, x: usize, y: usize) -> Unit {
        match self.slice.get(&y) {
            None => Unit::Air,
            Some(row) => match row.get(&x) {
                None => Unit::Air,
                Some(&u) => u,
            }
        }
    }

    fn put_unit(&mut self, x: usize, y: usize) {
        let colour = match self.get_unit(x, y) {
            Unit::Air => 0x5368a0,
            Unit::Rock => 0x5a3e36,
            Unit::Sand => 0xe5d68e,
        };

        for py in 0 .. UNIT_WIDTH {
            for px in 0 .. UNIT_WIDTH {
                self.buffer[(y * UNIT_WIDTH + py) * WINDOW_WIDTH + (x * UNIT_WIDTH + px)] = colour;
            }
        }
    }
}

fn main() {
    let mut window = Window::new(
        "Sand",
        WINDOW_WIDTH,
        WINDOW_WIDTH,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut slice = Slice::new(&mut window);

    slice.run();
}
