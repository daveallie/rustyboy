use std::sync::mpsc;

pub struct Input {
    up: Key,
    down: Key,
    left: Key,
    right: Key,
    a: Key,
    b: Key,
    select: Key,
    start: Key,
    io_register: u8,
    pub interrupt: u8,
    key_data_receiver: mpsc::Receiver<Key>,
}

#[derive(Debug)]
pub struct Key {
    pub key_type: KeyType,
    pub is_down: bool,
}

#[derive(Debug)]
pub enum KeyType {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

impl Input {
    pub fn new(key_data_receiver: mpsc::Receiver<Key>) -> Self {
        Self {
            up: Key { key_type: KeyType::Right, is_down: false },
            down: Key { key_type: KeyType::Left, is_down: false },
            left: Key { key_type: KeyType::Up, is_down: false },
            right: Key { key_type: KeyType::Down, is_down: false },
            a: Key { key_type: KeyType::A, is_down: false },
            b: Key { key_type: KeyType::B, is_down: false },
            select: Key { key_type: KeyType::Select, is_down: false },
            start: Key { key_type: KeyType::Start, is_down: false },
            io_register: 0,
            interrupt: 0,
            key_data_receiver,
        }
    }

    pub fn read(&self) -> u8 {
        self.io_register
    }

    pub fn write(&mut self, value: u8) {
        self.io_register = value;
        self.update_io_register();
    }

    pub fn run_cycle(&mut self) {
        match self.key_data_receiver.try_recv() {
            Ok(key) => {
                match key.key_type {
                    KeyType::Up => self.up.is_down = key.is_down,
                    KeyType::Down => self.down.is_down = key.is_down,
                    KeyType::Left => self.left.is_down = key.is_down,
                    KeyType::Right => self.right.is_down = key.is_down,
                    KeyType::A => self.a.is_down = key.is_down,
                    KeyType::B => self.b.is_down = key.is_down,
                    KeyType::Select => self.select.is_down = key.is_down,
                    KeyType::Start => self.start.is_down = key.is_down,
                }

                if key.is_down {
                    self.interrupt |= 0x10;
                }

                self.update_io_register();
            },
            _ => ()
        }
    }

    fn update_io_register(&mut self) {
        // filter to bit 4 & 5 as they are the only valid input bits
        self.io_register &= 0x30;

        if self.io_register & 0x10 == 0 {
            // bit 4 is low, check R L U D keys
            let row_res: u8 = self.row_0_keys().iter()
                .filter(|key| !key.is_down)
                .map(|key| key.key_type.value())
                .fold(0, |acc, key_value| acc | key_value);

            self.io_register |= row_res;
        }

        if self.io_register & 0x20 == 0 {
            // bit 5 is low, check A B Se St keys
            let row_res: u8 = self.row_1_keys().iter()
                .filter(|key| !key.is_down)
                .map(|key| key.key_type.value())
                .fold(0, |acc, key_value| acc | key_value);

            self.io_register |= row_res;
        }
    }

    fn row_0_keys(&self) -> [&Key; 4] {
        [
            &self.up,
            &self.down,
            &self.left,
            &self.right,
        ]
    }

    fn row_1_keys(&self) -> [&Key; 4] {
        [
            &self.a,
            &self.b,
            &self.select,
            &self.start,
        ]
    }
}

impl KeyType {
    pub fn value(&self) -> u8 {
        match *self {
            KeyType::Right | KeyType::A => 0x01,
            KeyType::Left | KeyType::B => 0x02,
            KeyType::Up | KeyType::Select => 0x04,
            KeyType::Down | KeyType::Start => 0x08,
        }
    }

//    pub fn row(&self) -> u8 {
//        match *self {
//            KeyType::Right | KeyType::Left | KeyType::Down | KeyType::Up => 1,
//            KeyType::A | KeyType::B | KeyType::Select | KeyType::Start => 2,
//        }
//    }
}
