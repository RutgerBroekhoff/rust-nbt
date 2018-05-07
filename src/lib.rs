/*

Copyright (c) 2018, Rutger Broekhoff
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:
    * Redistributions of source code must retain the above copyright
      notice, this list of conditions and the following disclaimer.
    * Redistributions in binary form must reproduce the above copyright
      notice, this list of conditions and the following disclaimer in the
      documentation and/or other materials provided with the distribution.
    * Neither the name of the <organization> nor the
      names of its contributors may be used to endorse or promote products
      derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL <COPYRIGHT HOLDER> BE LIABLE FOR ANY
DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

 */

#[macro_use]
extern crate nom;
extern crate byteorder;

use byteorder::{
    WriteBytesExt,
    BigEndian,
};

use nom::{
    be_i8,
    be_i16,
    le_i16,
    be_u16,
    le_u16,
    be_i32,
    le_i32,
    be_i64,
    le_i64,
    Endianness,
    ErrorKind,
    IResult,
};

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::str;
use std::vec::Vec;

macro_rules! f32 ( ($i:expr, $e:expr) => ( {if nom::Endianness::Big == $e { nom::be_f32($i) } else { nom::le_f32($i) } } ););
macro_rules! f64 ( ($i:expr, $e:expr) => ( {if nom::Endianness::Big == $e { nom::be_f64($i) } else { nom::le_f64($i) } } ););

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

#[derive(Debug, PartialEq, Clone)]
pub struct NBTFile {
    root_name: String,
    root: NBTTag,
}

impl NBTFile {
    pub fn from_tuple(tuple: (&str, NBTTag)) -> Option<NBTFile> {
        if let &NBTTag::TagCompound(_) = &tuple.1 {
            Some(NBTFile {
                root_name: tuple.0.clone().to_owned(),
                root: tuple.1
            })
        } else {
            None
        }
    }
    
    pub fn from_path(path: &str) -> Result<NBTFile, String> {
        let path = Path::new(path);
        let display = path.display();
        
        let mut file = match File::open(&path) {
            Err(msg) => return Err(format!("File {} could not be opened: {}", display, msg.description())),
            Ok(file) => file,
        };
        
        NBTFile::from_file(&mut file)
    }
    
    pub fn from_file(file: &mut File) -> Result<NBTFile, String> {
        let mut bytes: Vec<u8> = Vec::new();
    
        match file.read_to_end(&mut bytes) {
            Err(msg) => return Err(format!("Error reading file: {}", msg.description())),
            Ok(_) => (),
        };
        
        NBTFile::from_bytes(&bytes)
    }
    
    pub fn from_bytes(bytes: &Vec<u8>) -> Result<NBTFile, String> {
        let file_raw = read_nbt_file(bytes.as_slice());
        
        if let Ok(file) = file_raw {
            if let Some(file_root) = file.1 {
                return Ok(file_root)
            } else {
                return Err("File could not be read".to_owned())
            }  
        }
        
        Err("File could not be read".to_owned())
    }
    
    pub fn write_to_path(&self, path: &str) -> Result<(), String> {
        let path = Path::new(path);
        let display = path.display();
        
        let mut file = match File::open(&path) {
            Err(msg) => return Err(format!("File {} could not be opened: {}", display, msg.description())),
            Ok(file) => file,
        };
        
        self.write_to_file(&mut file)
    }
    
    pub fn write_to_file(&self, file: &mut File) -> Result<(), String> {
        match file.write_all(self.as_bytes()?.as_slice()) {
            Err(msg) => return Err(format!("Error writing to file: {}", msg.description())),
            Ok(_) => return Ok(()),
        }
    }
    
    pub fn as_bytes(&self) -> Result<Vec<u8>, String> {
        return write_tag(&self.root, true, true, Some(&self.root_name))
    }
}

named!(read_tag_name<&[u8], &str>,
    do_parse!(
        len:  u16!(nom::Endianness::Big) >>
        name: take!(len)                 >>
        (str::from_utf8(name).unwrap())
    )
);

named!(read_tag_byte<&[u8], NBTTag>,
    do_parse!(
        val: be_i8 >>
        (NBTTag::TagByte(val))
    )
);

named!(read_tag_short<&[u8], NBTTag>,
    do_parse!(
        val: i16!(nom::Endianness::Big) >>
        (NBTTag::TagShort(val))
    )
);

named!(read_tag_int<&[u8], NBTTag>,
    do_parse!(
        val: i32!(nom::Endianness::Big) >>
        (NBTTag::TagInt(val))
    )
);

named!(read_tag_long<&[u8], NBTTag>,
    do_parse!(
        val: i64!(nom::Endianness::Big) >>
        (NBTTag::TagLong(val))
    )
);

named!(read_tag_float<&[u8], NBTTag>,
    do_parse!(
        val: f32!(nom::Endianness::Big) >>
        (NBTTag::TagFloat(val))
    )
);

named!(read_tag_double<&[u8], NBTTag>,
    do_parse!(
        val: f64!(nom::Endianness::Big) >>
        (NBTTag::TagDouble(val))
    )
);

named!(read_tag_byte_array<&[u8], NBTTag>,
    do_parse!(
        len: i32!(nom::Endianness::Big)        >>
        val: many_m_n!(1, len as usize, be_i8) >>
        (NBTTag::TagByteArray(val))
    )
);

named!(read_tag_string<&[u8], NBTTag>,
    do_parse!(
        len: u16!(nom::Endianness::Big) >>
        val: take!(len)                 >>
        (NBTTag::TagString(str::from_utf8(val).unwrap().to_owned()))
    )
);

named!(read_tag_list<&[u8], NBTTag>,
    do_parse!(
        elems_type: take!(1) >>
        len: i32!(nom::Endianness::Big) >>
        elems: many_m_n!(1, len as usize, apply!(read_tag_known, elems_type[0])) >>
        (NBTTag::TagList(elems))
    )
);

named!(read_tag_compound<&[u8], NBTTag>,
    do_parse!(
        elems: many_till!(read_tag, tag!([0x00])) >>
        (NBTTag::TagCompound(tuple_vector_to_hashmap(elems.0)))
    )
);

named!(read_tag_int_array<&[u8], NBTTag>,
    do_parse!(
        len: i32!(nom::Endianness::Big)         >>
        val: many_m_n!(1, len as usize, be_i32) >>
        (NBTTag::TagIntArray(val))
    )
);

named!(read_tag_long_array<&[u8], NBTTag>,
    do_parse!(
        len: i32!(nom::Endianness::Big)         >>
        val: many_m_n!(1, len as usize, be_i64) >>
        (NBTTag::TagLongArray(val))
    )
);

named!(read_tag<&[u8], (&str, NBTTag)>,
    do_parse!(
        tag_type: take!(1)                          >>
        name: read_tag_name                         >>
        output: apply!(read_tag_known, tag_type[0]) >>
        (name, output)
    )
);

named!(read_nbt_file<&[u8], Option<NBTFile>>,
    do_parse!(
        root: read_tag >>
        (NBTFile::from_tuple(root))
    )
);

pub fn read_tag_known(input: &[u8], tag_type: u8) -> IResult<&[u8], NBTTag> {
    match tag_type {
        1  => read_tag_byte(input),
        2  => read_tag_short(input),
        3  => read_tag_int(input),
        4  => read_tag_long(input),
        5  => read_tag_float(input),
        6  => read_tag_double(input),
        7  => read_tag_byte_array(input),
        8  => read_tag_string(input),
        9  => read_tag_list(input),
        10 => read_tag_compound(input),
        11 => read_tag_int_array(input),
        12 => read_tag_long_array(input),
        _  => Err(nom::Err::Error(error_position!(input, ErrorKind::Custom(0)))),
    }
}

fn tuple_vector_to_hashmap(input: Vec<(&str, NBTTag)>) -> HashMap<String, NBTTag> {
    let mut map = HashMap::new();

    for item in input.iter() {
        map.insert(item.0.clone().to_owned(), item.1.clone());
    }

    return map;
}

pub fn write_tag_byte(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagByte(tag_value) = input {
        let mut output: Vec<u8> = Vec::new();
        
        output.write_i8(tag_value);
        
        return Ok(output)
    }
    
    Err("Tag is not of type TagByte".to_owned())
}

pub fn write_tag_short(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagShort(tag_value) = input {
        let mut output: Vec<u8> = Vec::new();
        
        output.write_i16::<BigEndian>(tag_value);
        
        return Ok(output)
    }
    
    Err("Tag is not of type TagShort".to_owned())
}

pub fn write_tag_int(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagInt(tag_value) = input {
        let mut output: Vec<u8> = Vec::new();
        
        output.write_i32::<BigEndian>(tag_value);
        
        return Ok(output)
    }
    
    Err("Tag is not of type TagInt".to_owned())
}

pub fn write_tag_long(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagLong(tag_value) = input {
        let mut output: Vec<u8> = Vec::new();
        
        output.write_i64::<BigEndian>(tag_value);
        
        return Ok(output)
    }
    
    Err("Tag is not of type TagLong".to_owned())
}

pub fn write_tag_float(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagFloat(tag_value) = input {
        let mut output: Vec<u8> = Vec::new();
        
        output.write_f32::<BigEndian>(tag_value);
        
        return Ok(output)
    }
    
    Err("Tag is not of type TagFloat".to_owned())
}

pub fn write_tag_double(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagDouble(tag_value) = input {
        let mut output: Vec<u8> = Vec::new();
        
        output.write_f64::<BigEndian>(tag_value);
        
        return Ok(output)
    }
    
    Err("Tag is not of type TagDouble".to_owned())
}

pub fn write_tag_byte_array(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagByteArray(ref tag_value) = input {
        let mut output: Vec<u8> = Vec::new();
        
        output.write_i32::<BigEndian>(tag_value.len() as i32);
        
        for byte in tag_value {
            output.write_i8(*byte);
        }
        
        return Ok(output)
    }
    
    Err("Tag is not of type TagByteArray".to_owned())
}

pub fn write_tag_string(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagString(ref tag_value) = input {
        let mut output: Vec<u8> = Vec::new();
        
        output.write_u16::<BigEndian>(tag_value.len() as u16);
        
        output.extend_from_slice(tag_value.as_bytes());
        
        return Ok(output)
    }
    
    Err("Tag is not of type TagString".to_owned())
}

pub fn write_tag_compound(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagCompound(ref tag_value) = input {
        let mut output: Vec<u8> = Vec::new();
        
        for tag in tag_value {
            match write_tag(tag.1, true, true, Some(tag.0)) {
                Ok(mut result) => output.append(&mut result),
                Err(msg)       => return Err(msg),
            }
        }
        
        output.push(0);
        
        return Ok(output)
    }
    
    Err("Tag is not of type TagCompound".to_owned())
}

pub fn write_tag_list(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagList(ref tag_value) = input {
        let mut output: Vec<u8> = Vec::new();
        
        if tag_value.len() <= 0 {
            return Err("Size of TagList is required to be bigger than 0".to_owned())
        }
        
        if let Some(tag_id) = get_tag_id(&tag_value[0]) {
            output.push(tag_id);   
        } else {
            return Err("Tag id not recognized".to_owned())
        }
        
        output.write_i32::<BigEndian>(tag_value.len() as i32);
        
        for tag in tag_value {
            match write_tag(tag, false, false, None) {
                Ok(mut result) => output.append(&mut result),
                Err(msg)       => return Err(msg),
            }
        }
    }
    
    Err("Tag is not of type TagList".to_owned())
}

pub fn write_tag_int_array(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagIntArray(ref tag_value) = input {
        let mut output: Vec<u8> = Vec::new();
        
        output.write_i32::<BigEndian>(tag_value.len() as i32);
        
        for int in tag_value {
            output.write_i32::<BigEndian>(*int);
        }
        
        return Ok(output)
    }
    
    Err("Tag is not of type TagIntArray".to_owned())
}

pub fn write_tag_long_array(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagLongArray(ref tag_value) = input {
        let mut output: Vec<u8> = Vec::new();
        
        output.write_i32::<BigEndian>(tag_value.len() as i32);
        
        for long in tag_value {
            output.write_i64::<BigEndian>(*long);
        }
        
        return Ok(output)
    }
    
    Err("Tag is not of type TagLongArray".to_owned())
}

pub fn write_tag(input: &NBTTag, write_id: bool, write_name: bool, name: Option<&String>) -> Result<Vec<u8>, String> {
    let mut output: Vec<u8> = Vec::new();
    
    if write_id {
        if let Some(tag_id) = get_tag_id(&input) {
            output.push(tag_id);   
        }
    }
    
    if write_name {
        if let Some(name_val) = name {
            if name_val.len() == 0 {
                output.write_u16::<BigEndian>(0 as u16);
            } else {
                output.write_u16::<BigEndian>(name_val.len() as u16);   
            }
            
            output.extend_from_slice(name_val.as_bytes());
        }
    }
    
    let mut tag_result: Vec<u8>;
    
    match input {
        &NBTTag::TagByte(_)      => tag_result = write_tag_byte(&input)?,
        &NBTTag::TagShort(_)     => tag_result = write_tag_short(&input)?,
        &NBTTag::TagInt(_)       => tag_result = write_tag_int(&input)?,
        &NBTTag::TagLong(_)      => tag_result = write_tag_long(&input)?,
        &NBTTag::TagFloat(_)     => tag_result = write_tag_float(&input)?,
        &NBTTag::TagDouble(_)    => tag_result = write_tag_double(&input)?,
        &NBTTag::TagByteArray(_) => tag_result = write_tag_byte_array(&input)?,
        &NBTTag::TagString(_)    => tag_result = write_tag_string(&input)?,
        &NBTTag::TagList(_)      => tag_result = write_tag_list(&input)?,
        &NBTTag::TagCompound(_)  => tag_result = write_tag_compound(&input)?,
        &NBTTag::TagIntArray(_)  => tag_result = write_tag_int_array(&input)?,
        &NBTTag::TagLongArray(_) => tag_result = write_tag_long_array(&input)?,
        _                        => return Err("Tag type not matched".to_owned())
    }
    
    output.append(&mut tag_result);
    
    Ok(output)
}

pub fn get_tag_id(tag: &NBTTag) -> Option<u8> {
    match tag {
        &NBTTag::TagByte(_)      => Some(1),
        &NBTTag::TagShort(_)     => Some(2),
        &NBTTag::TagInt(_)       => Some(3),
        &NBTTag::TagLong(_)      => Some(4),
        &NBTTag::TagFloat(_)     => Some(5),
        &NBTTag::TagDouble(_)    => Some(6),
        &NBTTag::TagByteArray(_) => Some(7),
        &NBTTag::TagString(_)    => Some(8),
        &NBTTag::TagList(_)      => Some(9),
        &NBTTag::TagCompound(_)  => Some(10),
        &NBTTag::TagIntArray(_)  => Some(11),
        &NBTTag::TagLongArray(_) => Some(12),
        _                        => None,
    }
}

#[cfg(test)]
mod tests {
    use NBTFile;
    use NBTTag;
    use read_nbt_file;
    use read_tag_name;
    use get_tag_id;
    use std::collections::HashMap;
    use tuple_vector_to_hashmap;

    #[test]
    fn check_read_name() {
        assert_eq!(read_tag_name(vec![0x00, 0x05, 0x48, 0x65, 0x6C, 0x6C, 0x6F].as_slice()), Ok((&b""[..], "Hello")))
    }

    #[test]
    fn check_tuple_vec_to_hashmap() {
        let input = vec![
            ("Hello World!", NBTTag::TagString("Test".to_owned())),
            ("Bye World!", NBTTag::TagInt(3))
        ];

        let mut expected = HashMap::new();

        expected.insert("Hello World!".to_owned(), NBTTag::TagString("Test".to_owned()));
        expected.insert("Bye World!".to_owned(),   NBTTag::TagInt(3));

        assert_eq!(tuple_vector_to_hashmap(input), expected);
    }

    #[test]
    fn check_nbt_file() {
        let input = vec![
            0x0A, 0x00, 0x01, 0x65, 0x08, 0x00, 0x05, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x00, 0x05, 0x48, 0x65, 0x6C, 0x6C, 0x6f, 0x00
        ];

        let mut compound_contents = HashMap::new();
        compound_contents.insert("Hello".to_owned(), NBTTag::TagString("Hello".to_owned()));

        assert_eq!(read_nbt_file(input.as_slice()), Ok((&b""[..],
        Some(NBTFile {
            root_name: "e".to_owned(),
            root: NBTTag::TagCompound(compound_contents)
        }))));
    }
    
    #[test]
    fn check_tag_matcher() {
        let input = NBTTag::TagString("Hello World!".to_owned());
        let result = get_tag_id(&input);
        let expected = Some(8);
        
        assert_eq!(result, expected);
    }
}