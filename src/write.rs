use byteorder::{
    BigEndian,
    WriteBytesExt,
};
use NBTTag;
use std::vec::Vec;

fn write_tag_byte(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagByte(tag_value) = input {
        let mut output: Vec<u8> = Vec::new();

        output.write_i8(tag_value);

        return Ok(output);
    }

    Err("Tag is not of type TagByte".to_owned())
}

fn write_tag_short(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagShort(tag_value) = input {
        let mut output: Vec<u8> = Vec::new();

        output.write_i16::<BigEndian>(tag_value);

        return Ok(output);
    }

    Err("Tag is not of type TagShort".to_owned())
}

fn write_tag_int(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagInt(tag_value) = input {
        let mut output: Vec<u8> = Vec::new();

        output.write_i32::<BigEndian>(tag_value);

        return Ok(output);
    }

    Err("Tag is not of type TagInt".to_owned())
}

fn write_tag_long(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagLong(tag_value) = input {
        let mut output: Vec<u8> = Vec::new();

        output.write_i64::<BigEndian>(tag_value);

        return Ok(output);
    }

    Err("Tag is not of type TagLong".to_owned())
}

fn write_tag_float(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagFloat(tag_value) = input {
        let mut output: Vec<u8> = Vec::new();

        output.write_f32::<BigEndian>(tag_value);

        return Ok(output);
    }

    Err("Tag is not of type TagFloat".to_owned())
}

fn write_tag_double(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagDouble(tag_value) = input {
        let mut output: Vec<u8> = Vec::new();

        output.write_f64::<BigEndian>(tag_value);

        return Ok(output);
    }

    Err("Tag is not of type TagDouble".to_owned())
}

fn write_tag_byte_array(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagByteArray(ref tag_value) = input {
        let mut output: Vec<u8> = Vec::new();

        output.write_i32::<BigEndian>(tag_value.len() as i32);

        for byte in tag_value {
            output.write_i8(*byte);
        }

        return Ok(output);
    }

    Err("Tag is not of type TagByteArray".to_owned())
}

fn write_tag_string(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagString(ref tag_value) = input {
        let mut output: Vec<u8> = Vec::new();

        output.write_u16::<BigEndian>(tag_value.len() as u16);

        output.extend_from_slice(tag_value.as_bytes());

        return Ok(output);
    }

    Err("Tag is not of type TagString".to_owned())
}

fn write_tag_compound(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagCompound(ref tag_value) = input {
        let mut output: Vec<u8> = Vec::new();

        for tag in tag_value {
            match write_tag(tag.1, true, true, Some(tag.0)) {
                Ok(mut result) => output.append(&mut result),
                Err(msg) => return Err(msg),
            }
        }

        output.push(0);

        return Ok(output);
    }

    Err("Tag is not of type TagCompound".to_owned())
}

fn write_tag_list(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagList(ref tag_value) = input {
        let mut output: Vec<u8> = Vec::new();

        if tag_value.len() <= 0 {
            return Err("Size of TagList is required to be bigger than 0".to_owned());
        }

        if let Some(tag_id) = get_tag_id(&tag_value[0]) {
            output.push(tag_id);
        } else {
            return Err("Tag id not recognized".to_owned());
        }

        output.write_i32::<BigEndian>(tag_value.len() as i32);

        for tag in tag_value {
            match write_tag(tag, false, false, None) {
                Ok(mut result) => output.append(&mut result),
                Err(msg) => return Err(msg),
            }
        }
    }

    Err("Tag is not of type TagList".to_owned())
}

fn write_tag_int_array(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagIntArray(ref tag_value) = input {
        let mut output: Vec<u8> = Vec::new();

        output.write_i32::<BigEndian>(tag_value.len() as i32);

        for int in tag_value {
            output.write_i32::<BigEndian>(*int);
        }

        return Ok(output);
    }

    Err("Tag is not of type TagIntArray".to_owned())
}

fn write_tag_long_array(input: &NBTTag) -> Result<Vec<u8>, String> {
    if let &NBTTag::TagLongArray(ref tag_value) = input {
        let mut output: Vec<u8> = Vec::new();

        output.write_i32::<BigEndian>(tag_value.len() as i32);

        for long in tag_value {
            output.write_i64::<BigEndian>(*long);
        }

        return Ok(output);
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
        &NBTTag::TagByte(_) => tag_result = write_tag_byte(&input)?,
        &NBTTag::TagShort(_) => tag_result = write_tag_short(&input)?,
        &NBTTag::TagInt(_) => tag_result = write_tag_int(&input)?,
        &NBTTag::TagLong(_) => tag_result = write_tag_long(&input)?,
        &NBTTag::TagFloat(_) => tag_result = write_tag_float(&input)?,
        &NBTTag::TagDouble(_) => tag_result = write_tag_double(&input)?,
        &NBTTag::TagByteArray(_) => tag_result = write_tag_byte_array(&input)?,
        &NBTTag::TagString(_) => tag_result = write_tag_string(&input)?,
        &NBTTag::TagList(_) => tag_result = write_tag_list(&input)?,
        &NBTTag::TagCompound(_) => tag_result = write_tag_compound(&input)?,
        &NBTTag::TagIntArray(_) => tag_result = write_tag_int_array(&input)?,
        &NBTTag::TagLongArray(_) => tag_result = write_tag_long_array(&input)?,
        _ => return Err("Tag type not matched".to_owned())
    }

    output.append(&mut tag_result);

    Ok(output)
}

pub fn get_tag_id(tag: &NBTTag) -> Option<u8> {
    match tag {
        &NBTTag::TagByte(_) => Some(1),
        &NBTTag::TagShort(_) => Some(2),
        &NBTTag::TagInt(_) => Some(3),
        &NBTTag::TagLong(_) => Some(4),
        &NBTTag::TagFloat(_) => Some(5),
        &NBTTag::TagDouble(_) => Some(6),
        &NBTTag::TagByteArray(_) => Some(7),
        &NBTTag::TagString(_) => Some(8),
        &NBTTag::TagList(_) => Some(9),
        &NBTTag::TagCompound(_) => Some(10),
        &NBTTag::TagIntArray(_) => Some(11),
        &NBTTag::TagLongArray(_) => Some(12),
        _ => None,
    }
}