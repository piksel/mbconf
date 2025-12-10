use core::{fmt::Debug, prelude::rust_2024::*};

#[cfg(feature = "defmt")]
pub trait Index: Sized + Copy + Eq + defmt::Format{}

#[cfg(not(feature = "defmt"))]
pub trait Index: Sized + Copy + Eq{}

pub trait ActionIndex: Sized + Copy + Eq + Debug {
    fn as_index(self) -> usize;
    fn from_byte(byte: u8) -> Option<Self>;
}

pub trait PropIndex: Sized + Copy + Eq + Debug {
    fn as_index(self) -> usize;
    fn from_byte(byte: u8) -> Option<Self>;
}

pub trait SectionIndex: Sized + Copy + Eq + Debug  {
    fn as_index(self) -> usize;
    fn from_byte(byte: u8) -> Option<Self>;
}

pub trait InfoIndex: Sized + Copy + Eq + Debug {
    fn as_index(self) -> usize;
    fn from_byte(byte: u8) -> Option<Self>;
}