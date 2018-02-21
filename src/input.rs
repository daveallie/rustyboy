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
                let changed = match key.key_type {
                    KeyType::Up => if self.up.is_down != key.is_down { self.up.is_down = key.is_down; true } else { false },
                    KeyType::Down => if self.down.is_down != key.is_down { self.down.is_down = key.is_down; true } else { false },
                    KeyType::Left => if self.left.is_down != key.is_down { self.left.is_down = key.is_down; true } else { false },
                    KeyType::Right => if self.right.is_down != key.is_down { self.right.is_down = key.is_down; true } else { false },
                    KeyType::A => if self.a.is_down != key.is_down { self.a.is_down = key.is_down; true } else { false },
                    KeyType::B => if self.b.is_down != key.is_down { self.b.is_down = key.is_down; true } else { false },
                    KeyType::Select => if self.select.is_down != key.is_down { self.select.is_down = key.is_down; true } else { false },
                    KeyType::Start => if self.start.is_down != key.is_down { self.start.is_down = key.is_down; true } else { false },
                };

                if changed {
                    if key.is_down {
                        self.interrupt |= 0x10;
                        println!("KEY DOWN: {:?}", key.key_type);
                    }

                    self.update_io_register();
                }
            },
            _ => ()
        }
    }

    fn update_io_register(&mut self) {
        // filter to bit 4 & 5 as they are the only valid input bits
        self.io_register &= 0x30;

        if self.io_register & 0x10 == 0 {
            // bit 4 is low, check R L U D keys
            let row_res: u8 = self.col_1_keys().iter()
                .filter(|key| !key.is_down)
                .map(|key| key.key_type.value())
                .fold(0, |acc, key_value| acc | key_value);

            self.io_register |= row_res;
        }

        if self.io_register & 0x20 == 0 {
            // bit 5 is low, check A B Se St keys
            let row_res: u8 = self.col_0_keys().iter()
                .filter(|key| !key.is_down)
                .map(|key| key.key_type.value())
                .fold(0, |acc, key_value| acc | key_value);

            self.io_register |= row_res;
        }
    }

    fn col_0_keys(&self) -> [&Key; 4] {
        [
            &self.a,
            &self.b,
            &self.select,
            &self.start,
        ]
    }

    fn col_1_keys(&self) -> [&Key; 4] {
        [
            &self.right,
            &self.left,
            &self.up,
            &self.down,
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
