use std::io::Write;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use serial::{SerialPort, SystemPort};

const SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate: serial::Baud2400,
    char_size: serial::Bits8,
    parity: serial::ParityNone,
    stop_bits: serial::Stop1,
    flow_control: serial::FlowNone,
};

pub struct Board {
    serial: SystemPort,
    rows: [Row; 4],
}

impl Board {
    pub fn new(port: String) -> Self {
        let mut serial = serial::open(&port).expect("Failed to connect to serial!");

        serial
            .configure(&SETTINGS)
            .expect("Failed to configure serial.");
        serial.set_timeout(Duration::from_secs(10)).unwrap();

        let rows = [
            Row::new('A', 6),
            Row::new('B', 7),
            Row::new('C', 7),
            Row::new('D', 7),
        ];

        let mut board = Board { serial, rows };

        board.write(0, "      ".to_string());
        board.write(1, "       ".to_string());
        board.write(2, "       ".to_string());
        board.write(3, "       ".to_string());

        thread::sleep(Duration::from_millis(1000));

        board.write(0, "TELAND".to_string());
        thread::sleep(Duration::from_millis(500));
        board.write(1, "TER ZEE".to_string());
        thread::sleep(Duration::from_millis(500));
        board.write(2, "DELUCHT".to_string());
        thread::sleep(Duration::from_millis(500));
        board.write(3, "LSTRM15".to_string());

        thread::sleep(Duration::from_millis(3000));

        board.write(0, "      ".to_string());
        board.write(1, "       ".to_string());
        board.write(2, "       ".to_string());
        board.write(3, "       ".to_string());

        thread::sleep(Duration::from_millis(1500));

        board
    }

    pub fn tick(&mut self) {
        let serial = &mut self.serial;

        self.rows[0].tick(serial);
        self.rows[1].tick(serial);
        self.rows[2].tick(serial);
        self.rows[3].tick(serial);
    }

    pub fn write(&mut self, row: usize, text: String) {
        let row = &mut self.rows[row];

        row.write(&mut self.serial, text)
    }
}

struct Row {
    id: char,
    length: usize,
    last_update: Instant,
    current_text: String,
    next_queued: Option<String>,
}

impl Row {
    pub fn new(id: char, length: usize) -> Self {
        Self {
            id,
            length,
            last_update: Instant::now().checked_sub(Duration::from_secs(5)).unwrap(),
            current_text: String::new(),
            next_queued: None,
        }
    }

    pub fn tick(&mut self, port: &mut SystemPort) {
        if self.last_update.elapsed().as_secs() < 1 {
            return;
        }

        let text = match &self.next_queued {
            Some(next) => next.clone(),
            None => return,
        };

        self.write(port, text.to_string());
    }

    pub fn write(&mut self, port: &mut SystemPort, msg: String) {
        if self.last_update.elapsed().as_secs() < 1 {
            self.next_queued = Some(msg);
            return;
        }

        let cmd = self.format_string(msg);
        self.last_update = Instant::now();
        self.next_queued = None;

        if self.current_text == cmd {
            return;
        }

        let _ = port.write_all(cmd.as_bytes());
        self.current_text = cmd;
    }

    fn format_string(&self, input: String) -> String {
        let length: usize = self.length.into();
        assert!(input.len() == length);

        let mut cmd = String::with_capacity(18);

        for i in 0..18 {
            let rel_index = i - (i / 3);

            if i % 3 == 2 || rel_index > self.length - 1 {
                cmd.push(' ');
                continue;
            }

            cmd.push(input.chars().nth(self.length - rel_index - 1).unwrap());
        }

        format!("{} {} X\r", self.id, cmd.chars().rev().collect::<String>())
    }
}
