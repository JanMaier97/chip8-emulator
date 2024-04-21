pub trait Keypad {
    fn is_key_down(&self, key: u8) -> bool;
    fn get_pressed_key(&self) -> Option<u8>;
}

pub struct MockKeypad {
    pub value: Option<u8>,
}

impl MockKeypad {
    pub fn from_value(value: u8) -> Self {
        Self { value: Some(value) }
    }
}

impl Default for MockKeypad {
    fn default() -> Self {
        Self { value: None }
    }
}

impl Keypad for MockKeypad {
    fn is_key_down(&self, key: u8) -> bool {
        let Some(value) = self.value else {
            return false;
        };
        value == key
    }

    fn get_pressed_key(&self) -> Option<u8> {
        self.value
    }
}
