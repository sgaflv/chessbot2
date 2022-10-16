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

pub fn bb_print(board: BBoard) {
    // print board
    for y in 0..8 {
        for x in 0..8 {
            if has_bit(&board, x, 7-y) {
                print!("*");
            } else {
                print!(".");
            }
        }

        println!();
    }
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

#[inline]
pub fn add_bit(board: &mut BBoard, x: u32, y: u32) {
    if x > 7 {
        return;
    }
    
    if y > 7 {
        return;
    }

    (*board) |= 1u64 << (x + y * 8) as u64;
}

#[inline]
pub fn has_bit(board: &BBoard, x: u32, y: u32) -> bool {
    if x > 7 {
        return false;
    }
    if y > 7 {
        return false;
    }

    return (*board & (1u64 << (x + y * 8) as u64)) > 0;
}


#[cfg(test)]
mod tests {
    use std::num::Wrapping;

    #[test]
    fn test_pop_count() {
        let test3 = 1u64 << 5 | 1 << 10 | 1 << 20;
        assert_eq!(test3.count_ones(), 3);
    }

    #[test]
    fn test_pop_count_side_bits() {
        let test64 = (Wrapping(1u64) << 63) + Wrapping(1);
        assert_eq!(test64.0.count_ones(), 2);
    }

    #[test]
    fn test_pop_count_all_bits() {
        let test64 = Wrapping(0u64) - Wrapping(1);
        assert_eq!(test64.0.count_ones(), 64);
    }

    #[test]
    fn test_pop_count_0() {
        assert_eq!(0u64.count_ones(), 0);
    }
}
