mod theme;
mod unit;

use crate::theme::{Theme, ThemeId};
use crate::unit::Unit;
use std::collections::HashMap;
use std::time;

use minifb::{Key, Window, WindowOptions, MouseButton, MouseMode};

const UNIT_WIDTH: usize = 16;
const UNITS_PER_ROW: usize = 39;
const WINDOW_WIDTH: usize = UNIT_WIDTH * UNITS_PER_ROW;

enum State {
    Playing,
    Paused,
    Stopped,
}

struct Slice<'a> {
    slice: HashMap<usize, HashMap<usize, Unit>>,
    buffer: Vec<u32>,
    window: &'a mut Window,
    cursor_pos: (usize, usize),
    cursor_size: usize,
    theme: Theme,
    state: State,
    x: Option<usize>,
    y: Option<usize>,
    emitter: usize,
}

impl<'a> Slice<'a> {
    fn new(window: &'a mut Window) -> Self {
        let slice = HashMap::new();
        let buffer = vec![0; WINDOW_WIDTH * WINDOW_WIDTH];
        let cursor_pos = (0, 0);
        let cursor_size = 8;
        let theme = Theme::get(ThemeId::Sandshell);
        let state = State::Stopped;
        let x = None;
        let y = None;
        let emitter = Self::centre();

        Self {
            slice,
            buffer,
            window,
            cursor_pos,
            cursor_size,
            theme,
            state,
            x,
            y,
            emitter,
        }
    }

    fn centre() -> usize {
        f32::ceil(UNITS_PER_ROW as f32 / 2.0) as usize - 1
    }

    fn running(&self) -> bool {
        self.window.is_open()
    }

    fn run(&mut self) {
        self.buf_units();
        while self.running() {
            self.update();
            match self.state {
                State::Playing => self.playing(),
                State::Paused  => self.paused(),
                State::Stopped => self.stopped(),
            }
        }
    }

    fn handle_input(&mut self) -> bool {
        if self.window.get_mouse_down(MouseButton::Left) {
            let (x, y) = self.mouse_pos_to_unit_pos();
            self.put_unit(x, y, self.cursor_size, Unit::Rock);
        } else if self.window.get_mouse_down(MouseButton::Right) {
            let (x, y) = self.mouse_pos_to_unit_pos();
            self.put_unit(x, y, self.cursor_size, Unit::Air);
        } else if let Some(scroll) = self.window.get_scroll_wheel() {
            if scroll.1 > 0.0 {
                self.cursor_size += 2;
            } else if self.cursor_size > 2 {
                self.cursor_size -= 2;
            }
        }

        if self.window.is_key_released(Key::S) {
            self.update();
            self.play();
            return false;
        } else if self.window.is_key_released(Key::P) {
            self.update();
            self.pause();
            return false;
        } else if self.window.is_key_released(Key::M) {
            self.update();
            self.stop();
            return false;
        } else if self.window.is_key_down(Key::Left) && self.emitter > 0 {
            self.emitter -= 1;
        } else if self.window.is_key_down(Key::Right) && self.emitter < UNITS_PER_ROW - 1 {
            self.emitter += 1;
        }

        self.update();
        return true;
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
                self.buf_unit(x + x1, y + y1);
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
            Unit::Air => self.theme.0,
            Unit::Rock => self.theme.1,
            Unit::Sand => self.theme.2,
        };

        for py in 0 .. UNIT_WIDTH {
            for px in 0 .. UNIT_WIDTH {
                self.buffer[(y * UNIT_WIDTH + py) * WINDOW_WIDTH + (x * UNIT_WIDTH + px)] = colour;
            }
        }
    }

    fn buf_units(&mut self) {
        for y in 0 .. UNITS_PER_ROW {
            for x in 0 .. UNITS_PER_ROW {
                self.buf_unit(x, y);
            }
        }
    }

    fn mouse_pos_to_unit_pos(&mut self) -> (usize, usize) {
        let mouse_pos = self.window.get_mouse_pos(MouseMode::Clamp).unwrap();
        let mx = mouse_pos.0;
        let my = mouse_pos.1;
        let ux = mx as usize / UNIT_WIDTH;
        let uy = my as usize / UNIT_WIDTH;

        self.unbuf_cursor();

        self.cursor_pos = (ux, uy);
        self.cursor_pos
    }

    fn gravity(&mut self) {
        if self.x.is_none() {
            self.x = Some(self.emitter);
            self.y = Some(0);
        }

        let x1 = self.x.unwrap();
        let y1 = self.y.unwrap();
        let y2 = y1 + 1;

        for x2 in [x1, if x1>0{x1-1}else{x1}, if x1<UNITS_PER_ROW-1{x1+1}else{x1}] {
            if self.get_unit(x2, y2) == Unit::Air {
                self.put_unit(x1, y1, 1, Unit::Air);
                self.put_unit(x2, y2, 1, Unit::Sand);
                self.x = Some(x2);
                self.y = Some(y2);
                if y2 >= UNITS_PER_ROW - 1 {
                    self.land();
                }
                return;
            }
        }

        self.land();
    }

    fn land(&mut self) {
        self.x = None;
        self.y = None;
    }

    fn clear_sand(&mut self) {
        for (_, row) in self.slice.iter_mut() {
            for (_, p) in row.iter_mut() {
                if *p == Unit::Sand {
                    *p = Unit::Air;
                }
            }
        }
    }

    fn buf_emitter(&mut self) {
        for py in 0 .. UNIT_WIDTH {
            for px in 0 .. UNIT_WIDTH {
                self.buffer[py * WINDOW_WIDTH + self.emitter * UNIT_WIDTH + px] = 0;
            }
        }
    }

    fn unbuf_cursor(&mut self) {
        for y in self.cursor_pos.1 .. self.cursor_pos.1 + self.cursor_size {
            for x in self.cursor_pos.0 .. self.cursor_pos.0 + self.cursor_size {
                if y < UNITS_PER_ROW && x < UNITS_PER_ROW {
                    self.buf_unit(x, y);
                }
            }
        }
    }

    fn buf_cursor(&mut self) {
        let (mx, my) = self.mouse_pos_to_unit_pos();
        let cs = self.cursor_size;
        let bs = 2;

        for uy in my .. my + cs {
            for ux in mx .. mx + cs {
                for py in 0 .. UNIT_WIDTH {
                    for px in 0 .. UNIT_WIDTH {
                        if uy == my && py < bs
                        || uy == my + cs - 1 && py >= UNIT_WIDTH - bs
                        || ux == mx && px < bs
                        || ux == mx + cs - 1 && px >= UNIT_WIDTH - bs
                        { 
                            let ky = uy * UNIT_WIDTH + py;
                            let mut kx = ux * UNIT_WIDTH + px;
                            kx = usize::min(kx, WINDOW_WIDTH-1);
                            let i = ky * WINDOW_WIDTH + kx;
                            if i < self.buffer.len() {
                                self.buffer[i] = 0;
                            }
                        }
                    }
                }
            }
        }
    }

    fn play(&mut self) {
        self.state = State::Playing;
    }

    fn pause(&mut self) {
        self.state = State::Paused;
    }

    fn stop(&mut self) {
        self.x = None;
        self.y = None;
        self.clear_sand();
        self.state = State::Stopped;
    }

    fn playing(&mut self) {
        while self.running() && self.handle_input() {
            self.gravity();
        }
    }

    fn paused(&mut self) {
        while self.running() && self.handle_input() {
        }
    }

    fn stopped(&mut self) {
        while self.running() && self.handle_input() {
        }
    }

    fn update(&mut self) {
        self.buf_emitter();
        self.buf_cursor();

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
