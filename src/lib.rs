#[macro_use]
extern crate nom;
extern crate byteorder;

use std::collections::HashMap;
use std::vec::Vec;

pub mod file;
mod read;
mod write;

#[derive(Debug, PartialEq, Clone)]
pub enum NBTTag {
    TagEnd,
    TagByte(i8),
    TagShort(i16),
    TagInt(i32),
    TagLong(i64),
    TagFloat(f32),
    TagDouble(f64),
    TagByteArray(Vec<i8>),
    TagString(String),
    TagList(Vec<NBTTag>),
    TagCompound(HashMap<String, NBTTag>),
    TagIntArray(Vec<i32>),
    TagLongArray(Vec<i64>),
}
