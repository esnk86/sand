use minifb::{Key, Window, WindowOptions, MouseButton, MouseMode};

use std::collections::HashMap;
use std::time;

const UNIT_WIDTH: usize = 16;
const UNITS_PER_ROW: usize = 39;
const WINDOW_WIDTH: usize = UNIT_WIDTH * UNITS_PER_ROW;

#[derive(Clone, Copy, PartialEq)]
enum Unit {
    Air,
    Rock,
    Sand,
}

struct Slice<'a> {
    slice: HashMap<usize, HashMap<usize, Unit>>,
    buffer: Vec<u32>,
    window: &'a mut Window,
    cursor_size: usize,
}

impl<'a> Slice<'a> {
    fn new(window: &'a mut Window) -> Self {
        let slice = HashMap::new();
        let buffer = vec![0; WINDOW_WIDTH * WINDOW_WIDTH];
        let cursor_size = 1;

        Self {
            slice,
            buffer,
            window,
            cursor_size,
        }
    }

    fn centre(&self) -> usize {
        f32::ceil(UNITS_PER_ROW as f32 / 2.0) as usize - 1
    }

    fn run(&mut self) {
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            if self.handle_input() {
                break;
            }
            self.update();
        }

        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            self.put_sand_unit();
        }
    }

    fn handle_input(&mut self) -> bool {
        if self.window.get_mouse_down(MouseButton::Left) {
            let (x, y) = self.mouse_pos_to_unit_pos();
            self.put_unit(x, y, self.cursor_size, Unit::Rock);
        } else if self.window.get_mouse_down(MouseButton::Right) {
            let (x, y) = self.mouse_pos_to_unit_pos();
            self.put_unit(x, y, self.cursor_size, Unit::Air);
        } else if self.window.get_mouse_down(MouseButton::Middle) {
            return true;
        } else if let Some(scroll) = self.window.get_scroll_wheel() {
            if scroll.1 > 0.0 {
                self.cursor_size += 1;
            } else {
                self.cursor_size -= 1;
            }
        }

        return false;
    }

    fn put_unit(&mut self, x: usize, y: usize, scale: usize, unit: Unit) {
        for y1 in 0 .. scale {
            for x1 in 0 .. scale {
                if let Some(row) = self.slice.get_mut(&(y + y1)) {
                    row.insert(x + x1, unit);
                } else {
                    let mut row = HashMap::new();
                    row.insert(x + x1, unit);
                    self.slice.insert(y + y1, row);
                }
            }
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

    fn buf_unit(&mut self, x: usize, y: usize) {
        let colour = match self.get_unit(x, y) {
            Unit::Air => 0x6b573d,
            Unit::Rock => 0x7a6e5e,
            Unit::Sand => 0xd8ccbb,
        };

        for py in 0 .. UNIT_WIDTH {
            for px in 0 .. UNIT_WIDTH {
                self.buffer[(y * UNIT_WIDTH + py) * WINDOW_WIDTH + (x * UNIT_WIDTH + px)] = colour;
            }
        }
    }

    fn mouse_pos_to_unit_pos(&self) -> (usize, usize) {
        let mouse_pos = self.window.get_mouse_pos(MouseMode::Clamp).unwrap();
        let mx = mouse_pos.0;
        let my = mouse_pos.1;
        let ux = mx as usize / UNIT_WIDTH;
        let uy = my as usize / UNIT_WIDTH;

        (ux, uy)
    }

    fn put_sand_unit(&mut self) {
        let mut x1 = self.centre();
        let mut y1 = 0;

        'GRAVITY: loop {
            self.handle_input();

            let y2 = y1 + 1;

            for x2 in [x1, x1-1, x1+1] {
                if self.get_unit(x2, y2) == Unit::Air {
                    self.put_unit(x1, y1, 1, Unit::Air);
                    x1 = x2;
                    y1 = y2;
                    self.put_unit(x1, y1, 1, Unit::Sand);
                    self.update();
                    if y1 + 1 == UNITS_PER_ROW {
                        break;
                    }
                    continue 'GRAVITY;
                }
            }

            self.put_unit(x1, y1, 1, Unit::Sand);
            self.update();
            return;
        }
    }

    fn update(&mut self) {
        for y in 0 .. UNITS_PER_ROW {
            for x in 0 .. UNITS_PER_ROW {
                self.buf_unit(x, y);
            }
        }

        let ms = self.cursor_size * UNIT_WIDTH;
        let (ux, uy) = self.mouse_pos_to_unit_pos();

        for py in 0 .. ms {
            for px in 0 .. ms {
                let i = (uy * UNIT_WIDTH + py) * WINDOW_WIDTH + (ux * UNIT_WIDTH + px);
                if i < self.buffer.len() {
                    self.buffer[i] = 0;
                }
            }
        }

        self.window
            .update_with_buffer(&self.buffer, WINDOW_WIDTH, WINDOW_WIDTH)
            .unwrap();
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

    window.set_cursor_visibility(false);

    window.limit_update_rate(Some(time::Duration::from_micros(16600)));

    let mut slice = Slice::new(&mut window);

    slice.run();
}
