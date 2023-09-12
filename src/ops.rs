use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u16", endian = "big")]
pub enum Op {
    #[deku(id = "0x00E0")]
    Clear,

    #[deku(id = "0x00EE")]
    Return,

    #[deku(id_pat = "0x1000..=0x1FFF")]
    AbsJump(#[deku(pad_bits_before = "4", bits = "12")] u16),

    #[deku(id_pat = "0x2000..=0x2FFF")]
    Call(#[deku(pad_bits_before = "4", bits = "12")] u16),

    #[deku(id_pat = "0xB000..=0xBFFF")]
    OffsetJump(#[deku(pad_bits_before = "4", bits = "12")] u16),

    #[deku(id_pat = "0x3000..=0x3FFF")]
    SkipEqVal(#[deku(pad_bits_before = "4", bits = "4")] u8, u8),

    #[deku(id_pat = "0x4000..=0x4FFF")]
    SkipNeqVal(#[deku(pad_bits_before = "4", bits = "4")] u8, u8),

    #[deku(id_pat = "0x5000..=0x5FFF")]
    SkipEqReg(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "0x6000..=0x6FFF")]
    SetVal(#[deku(pad_bits_before = "4", bits = "4")] u8, u8),

    #[deku(id_pat = "0x7000..=0x7FFF")]
    AddVal(#[deku(pad_bits_before = "4", bits = "4")] u8, u8),

    #[deku(id_pat = "0x9000..=0x9FFF")]
    SkipNeqReg(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "0xC000..=0xCFFF")]
    Rand(#[deku(pad_bits_before = "4", bits = "4")] u8, u8),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x0")]
    Mov(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x1")]
    Or(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x2")]
    And(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x3")]
    Xor(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x4")]
    Add(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x5")]
    Sub(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x6")]
    Shr(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x7")]
    SubN(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0xE")]
    Shl(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "0xD000..=0xDFFF")]
    Draw(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(bits = "4")] u8,
        #[deku(bits = "4")] u8,
    ),

    #[deku(
        id_pat = "0xE09E | 0xE19E | 0xE29E | 0xE39E | 0xE49E | 0xE59E | 0xE69E | 0xE79E | 0xE89E | 0xE99E | 0xEA9E | 0xEB9E  | 0xEC9E  | 0xED9E  | 0xEE9E  | 0xEF9E"
    )]
    SkipKey(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xE0A1 | 0xE1A1 | 0xE2A1 | 0xE3A1 | 0xE4A1 | 0xE5A1 | 0xE6A1 | 0xE7A1 | 0xE8A1 | 0xE9A1 | 0xEAA1 | 0xEBA1  | 0xECA1  | 0xEDA1  | 0xEEA1  | 0xEFA1"
    )]
    SkipNoKey(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF00A | 0xF10A | 0xF20A | 0xF30A | 0xF40A | 0xF50A | 0xF60A | 0xF70A | 0xF80A | 0xF90A | 0xFA0A | 0xFB0A  | 0xFC0A  | 0xFD0A  | 0xFE0A  | 0xFF0A"
    )]
    GetKey(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF007 | 0xF107 | 0xF207 | 0xF307 | 0xF407 | 0xF507 | 0xF607 | 0xF707 | 0xF807 | 0xF907 | 0xFA07 | 0xFB07  | 0xFC07  | 0xFD07  | 0xFE07  | 0xFF07"
    )]
    GetDelay(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF015 | 0xF115 | 0xF215 | 0xF315 | 0xF415 | 0xF515 | 0xF615 | 0xF715 | 0xF815 | 0xF915 | 0xFA15 | 0xFB15  | 0xFC15  | 0xFD15  | 0xFE15  | 0xFF15"
    )]
    SetDelay(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF018 | 0xF118 | 0xF218 | 0xF318 | 0xF418 | 0xF518 | 0xF618 | 0xF718 | 0xF818 | 0xF918 | 0xFA18 | 0xFB18  | 0xFC18  | 0xFD18  | 0xFE18  | 0xFF18"
    )]
    SetSoundTimer(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF01E | 0xF11E | 0xF21E | 0xF31E | 0xF41E | 0xF51E | 0xF61E | 0xF71E | 0xF81E | 0xF91E | 0xFA1E | 0xFB1E  | 0xFC1E  | 0xFD1E  | 0xFE1E  | 0xFF1E"
    )]
    IncrIndex(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(id_pat = "0xA000..=0xAFFF")]
    SetIndex(#[deku(pad_bits_before = "4", bits = "12")] u16),

    #[deku(
        id_pat = "0xF029 | 0xF129 | 0xF229 | 0xF329 | 0xF429 | 0xF529 | 0xF629 | 0xF729 | 0xF829 | 0xF929 | 0xFA29 | 0xFB29  | 0xFC29  | 0xFD29  | 0xFE29  | 0xFF29"
    )]
    SetSpriteI(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF033 | 0xF133 | 0xF233 | 0xF333 | 0xF433 | 0xF533 | 0xF633 | 0xF733 | 0xF833 | 0xF933 | 0xFA33 | 0xFB33  | 0xFC33  | 0xFD33  | 0xFE33  | 0xFF33"
    )]
    DecimalRepr(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF055 | 0xF155 | 0xF255 | 0xF553 | 0xF455 | 0xF555 | 0xF655 | 0xF755 | 0xF855 | 0xF955 | 0xFA55 | 0xFB55  | 0xFC55  | 0xFD55  | 0xFE55  | 0xFF55"
    )]
    DumpRegisters(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF065 | 0xF165 | 0xF265 | 0xF653 | 0xF465 | 0xF655 | 0xF665 | 0xF765 | 0xF865 | 0xF965 | 0xFA65 | 0xFB65  | 0xFC65  | 0xFD65  | 0xFE65  | 0xFF65"
    )]
    LoadRegisters(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(id_pat = "_")]
    Unknown(u16),
}
