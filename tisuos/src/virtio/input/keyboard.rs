//! # 键盘编码
//! 
//! 2021年1月11日 zg

#![allow(dead_code)]

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum Key
{
	None,
	ESC,
	One,
	Two,
	Three,
	Four,
	Five,
	Six,
	Seven,
	Eight,
	Nine,
	Zero,
	Minus,
	Equal,
	BackSpace,
	Tab,
	Q,
	W,
	E,
	R,
	T,
	Y,
	U,
	I,
	O,
	P,
	LSB,
	RSB,
	Enter,
	LCTRL,
	A,
	S,
	D,
	F,
	G,
	H,
	J,
	K,
	L,
	Colon,
	SineglePoint,
	Point,
	LSHIFT,
	BackSlash,
	Z,
	X,
	C,
	V,
	B,
	N,
	M,
	Comma,
	Dot,
	Slash,
	RSHIFT,
	DPSTAR,
	LALT,
	Space,
    CAPS,
    MouseLeft = 272,
    MouseRight,
    MouseMid,
}

impl Key {
    pub fn from(x : u16)->Self{
        match x {
            1 => {Key::ESC}
            2 => {Key::One}
            3 => {Key::Two}
            4 => {Key::Three}
            5 => {Key::Four}
            6 => {Key::Five}
            7 => {Key::Six}
            8 => {Key::Seven}
            9 => {Key::Eight}
            10 => {Key::Nine}
            11 => {Key::Zero}
            12 => {Key::Minus}
            13 => {Key::Equal}
            14 => {Key::BackSpace}
            15 => {Key::Tab}
            16 => {Key::Q}
            17 => {Key::W}
            18 => {Key::E}
            19 => {Key::R}
            20 => {Key::T}
            21 => {Key::Y}
            22 => {Key::U}
            23 => {Key::I}
            24 => {Key::O}
            25 => {Key::P}
            26 => {Key::LSB}
            27 => {Key::RSB}
            28 => {Key::Enter}
            29 => {Key::LCTRL}
            30 => {Key::A}
            31 => {Key::S}
            32 => {Key::D}
            33 => {Key::F}
            34 => {Key::G}
            35 => {Key::H}
            36 => {Key::J}
            37 => {Key::K}
            38 => {Key::L}
            39 => {Key::Colon}
            40 => {Key::SineglePoint}
            41 => {Key::Point}
            42 => {Key::LSHIFT}
            43 => {Key::BackSlash}
            44 => {Key::Z}
            45 => {Key::X}
            46 => {Key::C}
            47 => {Key::V}
            48 => {Key::B}
            49 => {Key::N}
            50 => {Key::M}
            51 => {Key::Comma}
            52 => {Key::Dot}
            53 => {Key::Slash}
            54 => {Key::RSHIFT}
            55 => {Key::DPSTAR}
            56 => {Key::LALT}
            57 => {Key::Space}
            58 => {Key::CAPS}
            272 => {Key::MouseLeft}
            273 => {Key::MouseRight}
            274 => {Key::MouseMid}
            _ => {Key::None}
        }
    }
    pub fn to_char(self)->Option<char> {
        match self {
            Key::A => {Some('a')}
            Key::B => {Some('b')}
            Key::C => {Some('c')}
            Key::D => {Some('d')}
            Key::E => {Some('e')}
            Key::F => {Some('f')}
            Key::G => {Some('g')}
            Key::H => {Some('h')}
            Key::I => {Some('i')}
            Key::J => {Some('j')}
            Key::K => {Some('k')}
            Key::L => {Some('l')}
            Key::M => {Some('m')}
            Key::N => {Some('n')}
            Key::O => {Some('o')}
            Key::P => {Some('p')}
            Key::Q => {Some('q')}
            Key::R => {Some('r')}
            Key::S => {Some('s')}
            Key::T => {Some('t')}
            Key::U => {Some('u')}
            Key::V => {Some('v')}
            Key::W => {Some('w')}
            Key::X => {Some('x')}
            Key::Y => {Some('y')}
            Key::Z => {Some('z')}
            Key::Enter => {Some('\r')}
            Key::One => {Some('1')}
            Key::Two => {Some('2')}
            Key::Three => {Some('3')}
            Key::Four => {Some('4')}
            Key::Five => {Some('5')}
            Key::Six => {Some('6')}
            Key::Seven => {Some('7')}
            Key::Eight => {Some('8')}
            Key::Nine => {Some('9')}
            Key::Zero => {Some('0')}
            Key::Dot => {Some('.')}
            Key::Colon => {Some(';')}
            Key::SineglePoint => {Some('\'')}
            Key::BackSlash => {Some('\\')}
            Key::Comma => {Some(',')}
            Key::Slash => {Some('/')}
            Key::Space => {Some(' ')}
            Key::Minus => {Some('-')}
            Key::Equal => {Some('=')}
            Key::Tab => {Some('\t')}
            _ => {None}
        }
    }
}
