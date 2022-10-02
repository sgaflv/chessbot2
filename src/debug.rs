use crate::bboard::*;

pub trait AsString {
    fn as_string(&self) -> String;
}

pub trait Demo {
    fn demo(&self);
}

impl AsString for BBoard {
    fn as_string(&self) -> String {
        let mut result = String::new();

        for i in 0..8 {
            let b: u8 = ((*self >> (7 - i) * 8) & 255) as u8;

            result.push_str(format!("{:08b}\n", b.reverse_bits()).as_str());
        }

        result
    }
}

impl Demo for BBoard {
    fn demo(&self) {
        println!("{}", self.as_string());
    }
}
