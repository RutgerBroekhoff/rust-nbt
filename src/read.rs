use file::NBTFile;
use NBTTag;
use nom;
use nom::{
    be_i16,
    be_i32,
    be_i64,
    be_i8,
    be_u16,
    Endianness,
    ErrorKind,
    IResult,
    le_i16,
    le_i32,
    le_i64,
    le_u16,
};
use std::collections::HashMap;
use std::str;

macro_rules! f32 ( ($i:expr, $e:expr) => ( {if nom::Endianness::Big == $e { nom::be_f32($i) } else { nom::le_f32($i) } } ););
macro_rules! f64 ( ($i:expr, $e:expr) => ( {if nom::Endianness::Big == $e { nom::be_f64($i) } else { nom::le_f64($i) } } ););

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
        (NBTTag::TagCompound(tuple_vector_to_hash_map(elems.0)))
    )
);

named!(read_tag_int_array<&[u8], NBTTag>,
    do_parse!(
        len: i32!(nom::Endianness::Big)         >>
        val: many_m_n!(1, len as usize, i32!(nom::Endianness::Big)) >>
        (NBTTag::TagIntArray(val))
    )
);

named!(read_tag_long_array<&[u8], NBTTag>,
    do_parse!(
        len: i32!(nom::Endianness::Big)         >>
        val: many_m_n!(1, len as usize, i64!(nom::Endianness::Big)) >>
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

named!(pub read_nbt_file<&[u8], Option<NBTFile>>,
    do_parse!(
        root: read_tag >>
        (file_from_tuple(root))
    )
);

// Reads tag of which the type is already known
fn read_tag_known(input: &[u8], tag_type: u8) -> IResult<&[u8], NBTTag> {
    match tag_type {
        1 => read_tag_byte(input),
        2 => read_tag_short(input),
        3 => read_tag_int(input),
        4 => read_tag_long(input),
        5 => read_tag_float(input),
        6 => read_tag_double(input),
        7 => read_tag_byte_array(input),
        8 => read_tag_string(input),
        9 => read_tag_list(input),
        10 => read_tag_compound(input),
        11 => read_tag_int_array(input),
        12 => read_tag_long_array(input),
        _ => Err(nom::Err::Error(error_position!(input, ErrorKind::Custom(0)))),
    }
}

fn file_from_tuple(tuple: (&str, NBTTag)) -> Option<NBTFile> {
    if let &NBTTag::TagCompound(_) = &tuple.1 {
        Some(NBTFile {
            root_name: tuple.0.clone().to_owned(),
            root: tuple.1,
        })
    } else {
        None
    }
}

fn tuple_vector_to_hash_map(input: Vec<(&str, NBTTag)>) -> HashMap<String, NBTTag> {
    let mut map = HashMap::new();

    for item in input.iter() {
        map.insert(item.0.clone().to_owned(), item.1.clone());
    }

    return map;
}

#[test]
fn test_tuple_vec_to_hash_map() {
    let input = vec![
        ("Hello World!", NBTTag::TagString("Test".to_owned())),
        ("Bye World!", NBTTag::TagInt(3))
    ];

    let mut expected = HashMap::new();

    expected.insert("Hello World!".to_owned(), NBTTag::TagString("Test".to_owned()));
    expected.insert("Bye World!".to_owned(), NBTTag::TagInt(3));

    assert_eq!(tuple_vector_to_hash_map(input), expected);
}

#[test]
fn test_read_name() {
    assert_eq!(read_tag_name(vec![0x00, 0x05, 0x48, 0x65, 0x6C, 0x6C, 0x6F].as_slice()), Ok((&b""[..], "Hello")))
}

#[test]
fn test_read_nbt_file() {
    let input = vec![
        0x0A, 0x00, 0x01, 0x65, 0x08, 0x00, 0x05, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x00, 0x05, 0x48, 0x65, 0x6C, 0x6C, 0x6f, 0x00
    ];

    let mut compound_contents = HashMap::new();
    compound_contents.insert("Hello".to_owned(), NBTTag::TagString("Hello".to_owned()));

    assert_eq!(read_nbt_file(input.as_slice()), Ok((&b""[..],
                                                    Some(NBTFile {
                                                        root_name: "e".to_owned(),
                                                        root: NBTTag::TagCompound(compound_contents),
                                                    }))));
}
