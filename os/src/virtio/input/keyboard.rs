//! # 键盘编码
//! 
//! 2021年1月11日 zg

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
#[repr(u16)]
pub enum Key
{
	NONE,
	ESC,
	ONE,
	TWO,
	THREE,
	FOUR,
	FIVE,
	SIX,
	SEVEN,
	EIGHT,
	NINE,
	ZERO,
	MINUS,
	EQUAL,
	BACKSPACE,
	TAB,
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
	ENTER,
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
	COLON,
	SINGLEPOINT,
	POINT,
	LSHIFT,
	BACKSLASH,
	Z,
	X,
	C,
	V,
	B,
	N,
	M,
	COMMA,
	DOT,
	SLASH,
	RSHIFT,
	DPSTAR,
	LALT,
	SPACE,
    CAPS,
    MouseLeft = 272,
    MouseRight,
    MouseMid,
}

impl Key {
    pub fn from(x : u16)->Self{
        match x {
            1 => {Key::ESC}
            2 => {Key::ONE}
            3 => {Key::TWO}
            4 => {Key::THREE}
            5 => {Key::FOUR}
            6 => {Key::FIVE}
            7 => {Key::SIX}
            8 => {Key::SEVEN}
            9 => {Key::EIGHT}
            10 => {Key::NINE}
            11 => {Key::ZERO}
            12 => {Key::MINUS}
            13 => {Key::EQUAL}
            14 => {Key::BACKSPACE}
            15 => {Key::TAB}
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
            28 => {Key::ENTER}
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
            39 => {Key::COLON}
            40 => {Key::SINGLEPOINT}
            41 => {Key::POINT}
            42 => {Key::LSHIFT}
            43 => {Key::BACKSLASH}
            44 => {Key::Z}
            45 => {Key::X}
            46 => {Key::C}
            47 => {Key::V}
            48 => {Key::B}
            49 => {Key::N}
            50 => {Key::M}
            51 => {Key::COMMA}
            52 => {Key::DOT}
            53 => {Key::SLASH}
            54 => {Key::RSHIFT}
            55 => {Key::DPSTAR}
            56 => {Key::LALT}
            57 => {Key::SPACE}
            58 => {Key::CAPS}
            272 => {Key::MouseLeft}
            273 => {Key::MouseRight}
            274 => {Key::MouseMid}
            _ => {Key::NONE}
        }
    }
    pub fn to_char(self)->char {
        match self {
            Key::A => {'a'}
            Key::B => {'b'}
            Key::C => {'c'}
            Key::D => {'d'}
            Key::E => {'e'}
            Key::F => {'f'}
            Key::G => {'g'}
            Key::H => {'h'}
            Key::I => {'i'}
            Key::J => {'j'}
            Key::K => {'k'}
            Key::L => {'l'}
            Key::M => {'m'}
            Key::N => {'n'}
            Key::O => {'o'}
            Key::P => {'p'}
            Key::Q => {'q'}
            Key::R => {'r'}
            Key::S => {'s'}
            Key::T => {'t'}
            Key::U => {'u'}
            Key::V => {'v'}
            Key::W => {'w'}
            Key::X => {'x'}
            Key::Y => {'y'}
            Key::Z => {'z'}
            Key::ENTER => {'\r'}
            Key::ONE => {'1'}
            Key::TWO => {'2'}
            Key::THREE => {'3'}
            Key::FOUR => {'4'}
            Key::FIVE => {'5'}
            Key::SIX => {'6'}
            Key::SEVEN => {'7'}
            Key::EIGHT => {'8'}
            Key::NINE => {'9'}
            Key::ZERO => {'0'}
            Key::DOT => {'.'}
            Key::COLON => {';'}
            Key::SINGLEPOINT => {'\''}
            Key::BACKSLASH => {'\\'}
            Key::COMMA => {','}
            Key::SLASH => {'/'}
            Key::SPACE => {' '}
            Key::MINUS => {'-'}
            Key::EQUAL => {'='}
            Key::TAB => {'\t'}
            _ => {' '}
        }
    }
}
