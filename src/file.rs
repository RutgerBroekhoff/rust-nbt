use NBTTag;
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use std::path::Path;
use write;
use read;

#[derive(Debug, PartialEq, Clone)]
pub struct NBTFile {
    pub root_name: String,
    pub root: NBTTag,
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
        let file_raw = read::read_nbt_file(bytes.as_slice());
        
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
        return write::write_tag(&self.root, true, true, Some(&self.root_name))
    }
}