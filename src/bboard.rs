use std::num::Wrapping;

pub type BBoard = u64;

#[inline]
pub fn last_bit(b: BBoard) -> BBoard {
    b & (-Wrapping(b)).0
}

#[inline]
pub fn remove_last_bit(b: BBoard) -> BBoard {
    assert_ne!(b, 0);
    b & (b - 1)
}

pub fn bb_coord(x: u8, y: u8) -> BBoard {
    assert!(x < 8);
    assert!(y < 8);
    1u64 << (y * 8 + x) as u64
}

pub fn bb_coord_q(x: i32, y: i32) -> BBoard {
    if x > 7 || x < 0 {
        return 0;
    }
    if y > 7 || y < 0 {
        return 0;
    }

    1u64 << (y * 8 + x) as u64
}

pub fn bb_get_q(board: BBoard, x: i32, y: i32) -> bool {
    if x > 7 || x < 0 {
        return false;
    }

    if y > 7 || y < 0 {
        return false;
    }

    board & 1u64 << (y * 8 + x) as u64 > 0
}

pub fn bb_to_coord(board: BBoard) -> String {
    if board.count_ones() == 1 {
        let idx: u32 = board.trailing_zeros();
        let y = idx / 8;
        let x = idx % 8;
        String::from(format!(
            "{}{}",
            (x as u8 + b'a') as char,
            (y as u8 + b'1') as char
        ))
    } else {
        String::from("-")
    }
}
