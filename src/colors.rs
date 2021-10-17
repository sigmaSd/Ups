#![allow(missing_docs)]

use std::fmt::Display;

const DEL1: &str = "\x1b[";
const DEL2: &str = "m";
const DEL3: &str = "\x1b[0m";
const FG: &str = "38";

enum Palette {
    Red,
    Green,
    Yellow,
    Blue,
    LightBlue,
    Rgb(u8, u8, u8),
}

impl std::fmt::Display for Palette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Palette::*;
        match self {
            Red => write!(f, "255;0;0"),
            Yellow => write!(f, "255;255;0"),
            Green => write!(f, "0;255;0"),
            Blue => write!(f, "0;0;255"),
            LightBlue => write!(f, "0;150;255"),
            Rgb(r, g, b) => write!(f, "{};{};{}", r, g, b),
        }
    }
}

pub trait Color {
    fn red(&self) -> String;
    fn green(&self) -> String;
    fn yellow(&self) -> String;
    fn blue(&self) -> String;
    fn light_blue(&self) -> String;
    fn rgb(&self, r: u8, g: u8, b: u8) -> String;
}

impl<T> Color for T
where
    T: Display,
{
    fn red(&self) -> String {
        color(self, Palette::Red)
    }
    fn green(&self) -> String {
        color(self, Palette::Green)
    }
    fn yellow(&self) -> String {
        color(self, Palette::Yellow)
    }
    fn blue(&self) -> String {
        color(self, Palette::Blue)
    }
    fn light_blue(&self) -> String {
        color(self, Palette::LightBlue)
    }
    fn rgb(&self, r: u8, g: u8, b: u8) -> String {
        color(self, Palette::Rgb(r, g, b))
    }
}

fn color(string: impl Display, color: Palette) -> String {
    //"\x1b[$(fg/bg);2;$r;$g;$bm$string\x1b[0m"
    format!("{}{};2;{}{}{}{}", DEL1, FG, color, DEL2, string, DEL3)
}
