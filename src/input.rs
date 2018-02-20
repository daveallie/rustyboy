pub struct Input {
    keys: [Key; 8],
    io_register: u8,
}

impl Input {
    pub fn new() -> Self {
        let keys = [
            Key { key_type: KeyType::Right, is_down: false },
            Key { key_type: KeyType::Left, is_down: false },
            Key { key_type: KeyType::Up, is_down: false },
            Key { key_type: KeyType::Down, is_down: false },
            Key { key_type: KeyType::A, is_down: false },
            Key { key_type: KeyType::B, is_down: false },
            Key { key_type: KeyType::Select, is_down: false },
            Key { key_type: KeyType::Start, is_down: false },
        ];

        Self {
            keys,
            io_register: 0,
        }
    }

    pub fn read(&self) -> u8 {
        self.io_register
    }

    pub fn write(&mut self, value: u8) {
        self.io_register = value;
        self.update_io_register();
    }

    fn update_io_register(&mut self) {
        // filter to bit 4 & 5 as they are the only valid input bits
        self.io_register &= 0x30;

        if self.io_register & 0x10 == 0 {
            // bit 4 is low, check R L U D keys
            let row_res: u8 = self.keys[..4].iter()
                .filter(|key| !key.is_down)
                .map(|key| key.key_type.value())
                .fold(0, |acc, key_value| acc | key_value);

            self.io_register |= row_res;
        }

        if self.io_register & 0x20 == 0 {
            // bit 5 is low, check A B Se St keys
            let row_res: u8 = self.keys[4..].iter()
                .filter(|key| !key.is_down)
                .map(|key| key.key_type.value())
                .fold(0, |acc, key_value| acc | key_value);

            self.io_register |= row_res;
        }
    }
}

#[derive(Debug)]
struct Key {
    key_type: KeyType,
    is_down: bool,
}

#[derive(Debug)]
enum KeyType {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
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
