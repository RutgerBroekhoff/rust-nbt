use file::NBTFile;
use NBTTag;
use read::read_nbt_file;
use std::collections::HashMap;

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
                                                        root: NBTTag::TagCompound(compound_contents),
                                                    }))));
}
