use std::num::Wrapping;
use std::rc::Rc;

use arr_macro::arr;

use crate::bboard::{bb_coord_q, bb_get_q, BBoard, last_bit};
use crate::piece_moves::PieceMoveProvider;

pub struct Magic {
    move_provider: Rc<PieceMoveProvider>,
    rook_pop_bits: [i32; 64],
    bishop_pop_bits: [i32; 64],
    rook_shift_bits: [i32; 64],
    bishop_shift_bits: [i32; 64],
    rook_magic: [u64; 64],
    bishop_magic: [u64; 64],
    rook_attack_bits: [Vec<BBoard>; 64],
    bishop_attack_bits: [Vec<BBoard>; 64],
}

impl Magic {
    pub fn new(move_provider: Rc<PieceMoveProvider>) -> Magic {
        let mut result = Magic {
            move_provider,
            rook_pop_bits: [
                12, 11, 11, 11, 11, 11, 11, 12,
                11, 10, 10, 10, 10, 10, 10, 11,
                11, 10, 10, 10, 10, 10, 10, 11,
                11, 10, 10, 10, 10, 10, 10, 11,
                11, 10, 10, 10, 10, 10, 10, 11,
                11, 10, 10, 10, 10, 10, 10, 11,
                11, 10, 10, 10, 10, 10, 10, 11,
                12, 11, 11, 11, 11, 11, 11, 12
            ],
            bishop_pop_bits: [
                6, 5, 5, 5, 5, 5, 5, 6,
                5, 5, 5, 5, 5, 5, 5, 5,
                5, 5, 7, 7, 7, 7, 5, 5,
                5, 5, 7, 9, 9, 7, 5, 5,
                5, 5, 7, 9, 9, 7, 5, 5,
                5, 5, 7, 7, 7, 7, 5, 5,
                5, 5, 5, 5, 5, 5, 5, 5,
                6, 5, 5, 5, 5, 5, 5, 6,
            ],
            rook_shift_bits: [0; 64],
            bishop_shift_bits: [0; 64],
            rook_magic: [
                0xA8002C000108020u64,
                0x4440200140003000u64,
                0x8080200010011880u64,
                0x380180080141000u64,
                0x1A00060008211044u64,
                0x410001000A0C0008u64,
                0x9500060004008100u64,
                0x100024284A20700u64,
                0x802140008000u64,
                0x80C01002A00840u64,
                0x402004282011020u64,
                0x9862000820420050u64,
                0x1001448011100u64,
                0x6432800200800400u64,
                0x40100010002000Cu64,
                0x2800D0010C080u64,
                0x90C0008000803042u64,
                0x4010004000200041u64,
                0x3010010200040u64,
                0xA40828028001000u64,
                0x123010008000430u64,
                0x24008004020080u64,
                0x60040001104802u64,
                0x582200028400D1u64,
                0x4000802080044000u64,
                0x408208200420308u64,
                0x610038080102000u64,
                0x3601000900100020u64,
                0x80080040180u64,
                0xC2020080040080u64,
                0x80084400100102u64,
                0x4022408200014401u64,
                0x40052040800082u64,
                0xB08200280804000u64,
                0x8A80A008801000u64,
                0x4000480080801000u64,
                0x911808800801401u64,
                0x822A003002001894u64,
                0x401068091400108Au64,
                0x4A10A00004Cu64,
                0x2000800640008024u64,
                0x1486408102020020u64,
                0x100A000D50041u64,
                0x810050020B0020u64,
                0x204000800808004u64,
                0x20048100A000Cu64,
                0x112000831020004u64,
                0x9000040810002u64,
                0x440490200208200u64,
                0x8910401000200040u64,
                0x6404200050008480u64,
                0x4B824A2010010100u64,
                0x4080801810C0080u64,
                0x400802A0080u64,
                0x8224080110026400u64,
                0x40002C4104088200u64,
                0x1002100104A0282u64,
                0x1208400811048021u64,
                0x3201014A40D02001u64,
                0x5100019200501u64,
                0x101000208001005u64,
                0x2008450080702u64,
                0x1002080301D00Cu64,
                0x410201CE5C030092u64,
            ],
            bishop_magic: [
                0x40210414004040u64,
                0x2290100115012200u64,
                0xA240400A6004201u64,
                0x80A0420800480u64,
                0x4022021000000061u64,
                0x31012010200000u64,
                0x4404421051080068u64,
                0x1040882015000u64,
                0x8048C01206021210u64,
                0x222091024088820u64,
                0x4328110102020200u64,
                0x901CC41052000D0u64,
                0xA828C20210000200u64,
                0x308419004A004E0u64,
                0x4000840404860881u64,
                0x800008424020680u64,
                0x28100040100204A1u64,
                0x82001002080510u64,
                0x9008103000204010u64,
                0x141820040C00B000u64,
                0x81010090402022u64,
                0x14400480602000u64,
                0x8A008048443C00u64,
                0x280202060220u64,
                0x3520100860841100u64,
                0x9810083C02080100u64,
                0x41003000620C0140u64,
                0x6100400104010A0u64,
                0x20840000802008u64,
                0x40050A010900A080u64,
                0x818404001041602u64,
                0x8040604006010400u64,
                0x1028044001041800u64,
                0x80B00828108200u64,
                0xC000280C04080220u64,
                0x3010020080880081u64,
                0x10004C0400004100u64,
                0x3010020200002080u64,
                0x202304019004020Au64,
                0x4208A0000E110u64,
                0x108018410006000u64,
                0x202210120440800u64,
                0x100850C828001000u64,
                0x1401024204800800u64,
                0x41028800402u64,
                0x20642300480600u64,
                0x20410200800202u64,
                0xCA02480845000080u64,
                0x140C404A0080410u64,
                0x2180A40108884441u64,
                0x4410420104980302u64,
                0x1108040046080000u64,
                0x8141029012020008u64,
                0x894081818082800u64,
                0x40020404628000u64,
                0x804100C010C2122u64,
                0x8168210510101200u64,
                0x1088148121080u64,
                0x204010100C11010u64,
                0x1814102013841400u64,
                0xC00010020602u64,
                0x1045220C040820u64,
                0x12400808070840u64,
                0x2004012A040132u64,
            ],
            rook_attack_bits: arr![Vec::new(); 64],
            bishop_attack_bits: arr![Vec::new(); 64],
        };

        for i in 0..64 {
            result.rook_shift_bits[i] = 64 - result.rook_pop_bits[i];
            result.bishop_shift_bits[i] = 64 - result.bishop_pop_bits[i];
        }

        result.init_magic();

        result
    }

    #[inline]
    pub fn get_bishop_attack_bits(&self, idx: usize, board: BBoard) -> BBoard {
        let attack_bits = board & (self.move_provider.inner_bishop_attack_bits[idx]);
        let index = (Wrapping(attack_bits) * Wrapping(self.bishop_magic[idx]))
            >> (self.bishop_shift_bits[idx] as usize);

        self.bishop_attack_bits[idx][index.0 as usize]
    }

    #[inline]
    pub fn get_rook_attack_bits(&self, idx: usize, board: BBoard) -> BBoard {
        let attack_bits = board & self.move_provider.inner_rook_attack_bits[idx];
        let index = Wrapping(attack_bits) * Wrapping(self.rook_magic[idx])
            >> (self.rook_shift_bits[idx] as usize);

        self.rook_attack_bits[idx][index.0 as usize]
    }

    fn init_magic(&mut self) {
        init_rq_magic(
            &self.rook_shift_bits,
            &self.move_provider.inner_rook_attack_bits,
            &self.rook_magic,
            &mut self.rook_attack_bits,
            &compute_rook_attack_bits,
        );

        init_rq_magic(
            &self.bishop_shift_bits,
            &self.move_provider.inner_bishop_attack_bits,
            &self.bishop_magic,
            &mut self.bishop_attack_bits,
            &compute_bishop_attack_bits,
        );
    }
}

fn init_rq_magic(
    shift_bits: &[i32; 64],
    inner_attack_bits: &[BBoard; 64],
    magic: &[u64; 64],
    result_attack_bits: &mut [Vec<BBoard>; 64],
    attack_function: &dyn Fn(i32, i32, BBoard) -> BBoard,
) {
    // init attack bits
    for y in 0i32..8 {
        for x in 0i32..8 {
            let idx: usize = (y * 8 + x) as usize;
            let rp = 1 << (64 - shift_bits[idx as usize]);

            result_attack_bits[idx].reserve(rp as usize);
            for _ in 0..rp {
                result_attack_bits[idx].push(0u64);
            }

            for i in 0..rp {
                let mut j = i;
                let mut attack_bits: BBoard = inner_attack_bits[idx];
                let mut field: BBoard = 0u64;

                while attack_bits > 0 {
                    let last_bit = last_bit(attack_bits);

                    if j & 1 > 0 {
                        field |= last_bit;
                    }

                    j >>= 1;

                    attack_bits ^= last_bit;
                }

                field &= inner_attack_bits[idx];
                let mut index = (Wrapping(field) * Wrapping(magic[idx])).0;
                index >>= shift_bits[idx];

                result_attack_bits[idx][index as usize] = attack_function(x, y, field);
            }
        }
    }
}

fn compute_rook_attack_bits(x: i32, y: i32, field: BBoard) -> BBoard {
    let (mut dir1, mut dir2, mut dir3, mut dir4) = (true, true, true, true);
    let mut attack: BBoard = 0u64;
    for k in 1..8 {
        if dir1 {
            attack |= bb_coord_q(x + k, y);
        }
        if dir2 {
            attack |= bb_coord_q(x - k, y);
        }
        if dir3 {
            attack |= bb_coord_q(x, y + k);
        }
        if dir4 {
            attack |= bb_coord_q(x, y - k);
        }

        dir1 = dir1 && !bb_get_q(field, x + k, y);
        dir2 = dir2 && !bb_get_q(field, x - k, y);
        dir3 = dir3 && !bb_get_q(field, x, y + k);
        dir4 = dir4 && !bb_get_q(field, x, y - k);
    }

    attack
}

fn compute_bishop_attack_bits(x: i32, y: i32, field: BBoard) -> BBoard {
    let (mut dir1, mut dir2, mut dir3, mut dir4) = (true, true, true, true);
    let mut attack: BBoard = 0u64;
    for k in 1..8 {
        if dir1 {
            attack |= bb_coord_q(x + k, y + k);
        }
        if dir2 {
            attack |= bb_coord_q(x - k, y + k);
        }
        if dir3 {
            attack |= bb_coord_q(x + k, y - k);
        }
        if dir4 {
            attack |= bb_coord_q(x - k, y - k);
        }

        dir1 = dir1 && !bb_get_q(field, x + k, y + k);
        dir2 = dir2 && !bb_get_q(field, x - k, y + k);
        dir3 = dir3 && !bb_get_q(field, x + k, y - k);
        dir4 = dir4 && !bb_get_q(field, x - k, y - k);
    }

    attack
}
