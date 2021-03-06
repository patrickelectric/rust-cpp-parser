use super::lexer::{Lexer, Token};

#[derive(Clone, Debug, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
pub(crate) enum Kind {
    NON, // nothing
    SEL, // self: ', ", ...
    AAA, // a, b, ...
    BBB,
    FFF,
    NNN,
    RRR,
    TTT,
    VVV,
    OCT, // octal
    HEX, // x...
    UNS, // u...
    UNL, // U...
}

#[rustfmt::skip]
const ECHARS: [Kind; 256] = [
    // 0 NUL   1 SOH      2 STX      3 ETX      4 EOT      5 ENQ      6 ACK      7 BEL
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    // 8 BS    9 HT       A NL       B VT       C NP       D CR       E SO       F SI
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    // 10 DLE  11 DC1     12 DC2     13 DC3     14 DC4     15 NAK     16 SYN     17 ETB
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    // 18 CAN  19 EM      1A SUB     1B ESC     1C FS      1D GS      1E RS      1F US
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    // 20 SP   21  !      22  "      23  #      24  $      25  %      26  &      27  '
    Kind::NON, Kind::NON, Kind::SEL, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::SEL, //
    // 28  (   29  )      2A  *      2B  +      2C  ,      2D  -      2E  .      2F   /
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    // 30  0   31  1      32  2      33  3      34  4      35  5      36  6      37  7
    Kind::OCT, Kind::OCT, Kind::OCT, Kind::OCT, Kind::OCT, Kind::OCT, Kind::OCT, Kind::OCT, //
    // 38  8   39  9      3A  :      3B  ;      3C  <      3D  =      3E  >      3F  ?
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::SEL, //
    // 40  @   41  A      42  B      43  C      44  D      45  E      46  F      47  G
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    // 48  H   49  I      4A  J      4B  K      4C  L      4D  M      4E  N      4F  O
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    // 50  P   51  Q      52  R      53  S      54  T      55  U      56  V      57  W
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::UNL, Kind::NON, Kind::NON, //
    // 58  X   59  Y      5A  Z      5B  [      5C  \      5D  ]      5E  ^      5F  _
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::SEL, Kind::NON, Kind::NON, Kind::NON, //
    // 60  `   61  a      62  b      63  c      64  d      65  e      66  f      67  g
    Kind::NON, Kind::AAA, Kind::BBB, Kind::NON, Kind::NON, Kind::NON, Kind::FFF, Kind::NON, //
    // 68  h   69  i      6A  j      6B  k      6C  l      6D  m      6E  n      6F  o
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NNN, Kind::NON, //
    // 70  p   71  q      72  r      73  s      74  t      75  u      76  v      77  w
    Kind::NON, Kind::NON, Kind::RRR, Kind::NON, Kind::TTT, Kind::UNS, Kind::VVV, Kind::NON, //
    // 78  x   79  y      7A  z      7B  {      7C  |      7D  }      7E  ~      7F DEL
    Kind::HEX, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, // 
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
    Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, Kind::NON, //
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CharType {
    L,
    UU,
    U,
    U8,
}

impl<'a> Lexer<'a> {
    #[inline(always)]
    pub(crate) fn get_oct_char(&mut self, start: u32) -> u32 {
        let mut num = start;
        loop {
            if self.pos < self.len {
                let c = self.next_char(0);
                if b'0' <= c && c <= b'7' {
                    self.pos += 1;
                    num = 8 * num + u32::from(c - b'0');
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        num
    }

    #[inline(always)]
    pub(crate) fn get_hex_char(&mut self) -> u32 {
        let mut num = 0;
        for _ in 0..3 {
            if self.pos < self.len {
                let c = self.next_char(0);
                let n = Self::get_hex_num(c);
                if n < 16 {
                    self.pos += 1;
                    num = 16 * num + n;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        num as u32
    }

    #[inline(always)]
    pub(crate) fn get_universal_short(&mut self) -> u32 {
        // it has 4 digits
        let rem = self.len - self.pos;
        if rem >= 4 {
            let c1 = self.next_char(0);
            let c2 = self.next_char(1);
            let c3 = self.next_char(2);
            let c4 = self.next_char(3);
            self.pos += 4;
            // TODO: maybe check if we've hex digits...
            (0x1000 * Self::get_hex_num(c1)
                + 0x100 * Self::get_hex_num(c2)
                + 0x10 * Self::get_hex_num(c3)
                + Self::get_hex_num(c4)) as u32
        } else {
            0
        }
    }

    #[inline(always)]
    pub(crate) fn get_universal_long(&mut self) -> u32 {
        // it has 8 digits
        let rem = self.len - self.pos;
        if rem >= 8 {
            let c1 = self.next_char(0);
            let c2 = self.next_char(1);
            let c3 = self.next_char(2);
            let c4 = self.next_char(3);
            let c5 = self.next_char(4);
            let c6 = self.next_char(5);
            let c7 = self.next_char(6);
            let c8 = self.next_char(7);
            self.pos += 8;
            // TODO: maybe check if we've hex digits...
            (0x10000000 * Self::get_hex_num(c1)
                + 0x1000000 * Self::get_hex_num(c2)
                + 0x100000 * Self::get_hex_num(c3)
                + 0x10000 * Self::get_hex_num(c4)
                + 0x1000 * Self::get_hex_num(c5)
                + 0x100 * Self::get_hex_num(c6)
                + 0x10 * Self::get_hex_num(c7)
                + Self::get_hex_num(c8)) as u32
        } else {
            0
        }
    }

    #[inline(always)]
    pub(crate) fn get_escape(&mut self) -> u32 {
        if self.pos < self.len {
            let c = self.next_char(0);
            self.pos += 1;
            let kind = unsafe { ECHARS.get_unchecked(c as usize) };
            match kind {
                Kind::SEL => u32::from(c),
                Kind::AAA => 0x07,
                Kind::BBB => 0x08,
                Kind::FFF => 0x0C,
                Kind::NNN => 0x0A,
                Kind::RRR => 0x0D,
                Kind::TTT => 0x09,
                Kind::VVV => 0x0B,
                Kind::OCT => {
                    let first = u32::from(c - b'0');
                    self.get_oct_char(first)
                }
                Kind::HEX => self.get_hex_char(),
                Kind::UNS => self.get_universal_short(),
                Kind::UNL => self.get_universal_long(),
                _ => unreachable!(),
            }
        } else {
            0
        }
    }

    #[inline(always)]
    fn get_shift(c: u32) -> u32 {
        match c {
            0..=0xFF => 0x100,
            0xFF..=0xFFFF => 0x10000,
            _ => 0,
        }
    }

    #[inline(always)]
    pub(crate) fn get_c_char_u32(&mut self) -> u32 {
        let mut val: u32 = 0;
        loop {
            if self.pos < self.len {
                let c = self.next_char(0);
                if c == b'\\' {
                    self.pos += 1;
                    let e = self.get_escape();
                    // TODO: not sure that's correct
                    // e.g. \x12\x0034 == 1234 or 120034 ?
                    val = val * Self::get_shift(e) + e;
                } else if c == b'\'' {
                    self.pos += 1;
                    break;
                } else {
                    self.pos += 1;
                    val = val * 0x100 + u32::from(c);
                }
            } else {
                break;
            }
        }
        val
    }

    #[inline(always)]
    pub(crate) fn get_char(&mut self) -> Token<'a> {
        Token::LiteralChar(self.get_c_char_u32())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_char() {
        let mut p = Lexer::new(b"'a' 'b' 'c' '\\t' '\\n' '\\\'' '\\\"' '\\12' '\\1' '\\x12' '\\x12\\x34' 'abcd' '\\u1a2b' '\\U1a2B3c4D'");
        assert_eq!(p.next(), Token::LiteralChar(u32::from('a')));
        assert_eq!(p.next(), Token::LiteralChar(u32::from('b')));
        assert_eq!(p.next(), Token::LiteralChar(u32::from('c')));
        assert_eq!(p.next(), Token::LiteralChar(u32::from('\t')));
        assert_eq!(p.next(), Token::LiteralChar(u32::from('\n')));
        assert_eq!(p.next(), Token::LiteralChar(u32::from('\'')));
        assert_eq!(p.next(), Token::LiteralChar(u32::from('\"')));
        assert_eq!(p.next(), Token::LiteralChar(0o12));
        assert_eq!(p.next(), Token::LiteralChar(0o1));
        assert_eq!(p.next(), Token::LiteralChar(0x12));
        assert_eq!(p.next(), Token::LiteralChar(0x1234));
        assert_eq!(p.next(), Token::LiteralChar(0x61626364));
        assert_eq!(p.next(), Token::LiteralChar(0x1a2b));
        assert_eq!(p.next(), Token::LiteralChar(0x1a2b3c4d));
    }

    #[test]
    fn test_special_char() {
        let mut p = Lexer::new(b"u'a' U'b' u8'c' L'\\t'");
        assert_eq!(p.next(), Token::LiteralUChar(u32::from('a')));
        assert_eq!(p.next(), Token::LiteralUUChar(u32::from('b')));
        assert_eq!(p.next(), Token::LiteralU8Char(u32::from('c')));
        assert_eq!(p.next(), Token::LiteralLChar(u32::from('\t')));
    }
}
