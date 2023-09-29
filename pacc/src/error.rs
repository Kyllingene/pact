use std::error::Error;
use std::fmt::Display;

pub type RimResult<T> = Result<T, RimError>;

#[derive(Debug)]
pub enum RimError {
    LineTooShort(usize),
    LineTooLong(usize),
    InvalidInstruction(usize, String),
    InvalidRegister(usize, String),
    InvalidDevice(usize, String),
    InvalidInteger(usize, String),
    IntegerTooLarge(usize, u8, u8),
    IoError(std::io::Error),
}

impl Display for RimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LineTooShort(lno) => write!(f, "Line {lno} too short"),
            Self::LineTooLong(lno) => write!(f, "Line {lno} too long"),
            Self::InvalidInstruction(lno, instruction) => write!(f, "Invalid instruction `{instruction}` on line {lno}"),
            Self::InvalidRegister(lno, register) => write!(f, "Invalid register `{register}` on line {lno}"),
            Self::InvalidDevice(lno, device) => write!(f, "Invalid device `{device}` on line {lno}"),
            Self::InvalidInteger(lno, integer) => write!(f, "Invalid integer `{integer}` on line {lno}"),
            Self::IntegerTooLarge(lno, val, max) => write!(f, "Integer `{val}` on line {lno} too large (max is {max})"),
            Self::IoError(e) => e.fmt(f),
        }
    }
}

impl Error for RimError {}

impl From<std::io::Error> for RimError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}
