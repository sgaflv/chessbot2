use crate::bboard::BBoard;

pub struct PieceMoveProvider {
    pub black_pawn_move: [BBoard; 64],
    pub black_pawn_capture: [BBoard; 64],
    pub white_pawn_move: [BBoard; 64],
    pub white_pawn_capture: [BBoard; 64],

    pub rook_move: [BBoard; 64],
    pub knight_move: [BBoard; 64],
    pub bishop_move: [BBoard; 64],
    pub queen_move: [BBoard; 64],

    pub king_move: [BBoard; 64],

    pub inner_rook_attack_bits: [BBoard; 64],
    pub inner_bishop_attack_bits: [BBoard; 64],
}

impl PieceMoveProvider {
    pub fn new() -> PieceMoveProvider {
        let mut result = PieceMoveProvider {
            black_pawn_move: generate_moves(&black_pawn_move),
            black_pawn_capture: generate_moves(&black_pawn_capture),
            white_pawn_move: generate_moves(&white_pawn_move),
            white_pawn_capture: generate_moves(&white_pawn_capture),

            rook_move: generate_moves(&rook_move),
            knight_move: generate_moves(&knight_move),
            bishop_move: generate_moves(&bishop_move),
            queen_move: generate_moves(&queen_move),
            king_move: generate_moves(&king_move),

            inner_rook_attack_bits: [0; 64],
            inner_bishop_attack_bits: [0; 64],
        };

        let inner = inner_bits(&result.bishop_move, &result.rook_move);

        result.inner_bishop_attack_bits = inner.0;
        result.inner_rook_attack_bits = inner.1;

        result
    }
}

/* helpful boards */
static LEFT_BORDER: BBoard = 0x0101010101010101u64;
static RIGHT_BORDER: BBoard = 0x8080808080808080u64;
static TOP_BORDER: BBoard = 0xff00000000000000u64;
static BOTTOM_BORDER: BBoard = 0x00000000000000ffu64;

fn inner_bits(
    bishop_moves: &[BBoard; 64],
    rook_moves: &[BBoard; 64],
) -> ([BBoard; 64], [BBoard; 64]) {
    let mut r_bishop = [0; 64];
    let mut r_rook = [0; 64];

    for i in 0usize..64 {
        let mut bboard = bishop_moves[i];
        let mut rboard = rook_moves[i];
        let x = i % 8;
        let y = i / 8;

        if x > 0 {
            bboard &= !LEFT_BORDER;
            rboard &= !LEFT_BORDER;
        }

        if x < 7 {
            bboard &= !RIGHT_BORDER;
            rboard &= !RIGHT_BORDER;
        }

        if y > 0 {
            bboard &= !BOTTOM_BORDER;
            rboard &= !BOTTOM_BORDER;
        }

        if y < 7 {
            bboard &= !TOP_BORDER;
            rboard &= !TOP_BORDER;
        }

        r_rook[i] = rboard;
        r_bishop[i] = bboard;
    }

    (r_bishop, r_rook)
}

fn generate_moves(func: &dyn Fn(u8) -> BBoard) -> [BBoard; 64] {
    let mut result: [BBoard; 64] = [0u64; 64];

    for i in 0u8..64 {
        result[i as usize] = func(i);
    }

    result
}

fn black_pawn_move(idx: u8) -> BBoard {
    if idx < 8 {
        return 0u64;
    }
    let mut result = 1u64 << (idx - 8) as u64;

    if idx / 8 == 6 {
        result |= 1u64 << (idx - 16) as u64;
    }

    result
}

fn black_pawn_capture(idx: u8) -> BBoard {
    if idx < 8 {
        return 0u64;
    }

    let mut result = 0;

    if idx % 8 > 0 {
        result |= 1u64 << (idx - 9) as u64
    }
    if idx % 8 < 7 {
        result |= 1u64 << (idx - 7) as u64
    }

    result
}

fn white_pawn_move(idx: u8) -> BBoard {
    if idx > 55 {
        return 0u64;
    }

    let mut result = 1u64 << (idx + 8) as u64;

    if idx / 8 == 1 {
        result |= 1u64 << (idx + 16) as u64;
    }

    result
}

fn white_pawn_capture(idx: u8) -> BBoard {
    if idx > 55 {
        return 0u64;
    }

    let mut result = 0;

    if idx % 8 > 0 {
        result |= 1u64 << (idx + 7) as u64
    }
    if idx % 8 < 7 {
        result |= 1u64 << (idx + 9) as u64
    }

    result
}

fn rook_move(idx: u8) -> BBoard {
    let x1 = idx % 8;
    let y1 = idx / 8;

    let mut result = 0;

    for i in 0..64 {
        if idx == i {
            continue;
        }

        let x2 = i % 8;
        let y2 = i / 8;

        if x1 == x2 || y1 == y2 {
            result |= 1u64 << i as u64
        }
    }

    result
}

fn bishop_move(idx: u8) -> BBoard {
    let x1 = (idx % 8) as i32;
    let y1 = (idx / 8) as i32;

    let mut result = 0u64;

    for i in 0..64 {
        if idx == i {
            continue;
        }

        let x2 = (i % 8) as i32;
        let y2 = (i / 8) as i32;

        if x1 - y1 == x2 - y2 || x1 + y1 == x2 + y2 {
            result |= 1u64 << i as u64
        }
    }

    result
}

fn queen_move(idx: u8) -> BBoard {
    let x1 = (idx % 8) as i32;
    let y1 = (idx / 8) as i32;

    let mut result = 0u64;

    for i in 0..64 {
        if idx == i {
            continue;
        }

        let x2 = (i % 8) as i32;
        let y2 = (i / 8) as i32;

        if x1 == x2 || y1 == y2 || x1 - y1 == x2 - y2 || x1 + y1 == x2 + y2 {
            result |= 1u64 << i as u64
        }
    }

    result
}

fn king_move(idx: u8) -> BBoard {
    let x1 = (idx % 8) as i32;
    let y1 = (idx / 8) as i32;

    let mut result = 0u64;

    for i in 0..64 {
        if idx == i {
            continue;
        }

        let x2 = (i % 8) as i32;
        let y2 = (i / 8) as i32;

        let dx = (x1 - x2).abs();
        let dy = (y1 - y2).abs();

        if dx < 2 && dy < 2 {
            result |= 1u64 << i as u64
        }
    }

    result
}

fn knight_move(idx: u8) -> BBoard {
    let x1 = (idx % 8) as i32;
    let y1 = (idx / 8) as i32;

    let mut result = 0u64;

    for i in 0..64 {
        if idx == i {
            continue;
        }

        let x2 = (i % 8) as i32;
        let y2 = (i / 8) as i32;

        let dx = (x1 - x2).abs();
        let dy = (y1 - y2).abs();

        if dx + dy == 3 && dx > 0 && dy > 0 {
            result |= 1u64 << i as u64
        }
    }

    result
}
