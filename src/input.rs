use std::collections::HashSet;
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::keyboard::{PhysicalKey, KeyCode};

pub struct InputState {
    keys_pressed: HashSet<KeyCode>,
    keys_just_pressed: HashSet<KeyCode>,
    pub mouse_delta: (f64, f64),
    pub cursor_pos: (f64, f64),
    pub mouse_left_pressed: bool,
    pub mouse_left_just_pressed: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            keys_just_pressed: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            cursor_pos: (0.0, 0.0),
            mouse_left_pressed: false,
            mouse_left_just_pressed: false,
        }
    }

    pub fn handle_key_event(&mut self, event: &KeyEvent) {
        if let PhysicalKey::Code(key) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    if !self.keys_pressed.contains(&key) {
                        self.keys_just_pressed.insert(key);
                    }
                    self.keys_pressed.insert(key);
                }
                ElementState::Released => {
                    self.keys_pressed.remove(&key);
                    self.keys_just_pressed.remove(&key);
                }
            }
        }
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    pub fn end_frame(&mut self) {
        self.keys_just_pressed.clear();
        self.mouse_delta = (0.0, 0.0);
        self.mouse_left_just_pressed = false;
    }

    pub fn handle_cursor_moved(&mut self, x: f64, y: f64) {
        self.cursor_pos = (x, y);
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        if button == MouseButton::Left {
            match state {
                ElementState::Pressed => {
                    if !self.mouse_left_pressed {
                        self.mouse_left_just_pressed = true;
                    }
                    self.mouse_left_pressed = true;
                }
                ElementState::Released => {
                    self.mouse_left_pressed = false;
                }
            }
        }
    }
}
