use std::{
    fs,
    io::{self, Read, Write}
};

pub fn get(file_path: &str) -> io::Result<String> {
    let mut buf = String::new();
    fs::File::open(file_path)?.read_to_string(&mut buf)?;
    Ok(buf.trim().to_string())
}

pub fn set(file_path: &str, value: &str) -> io::Result<()> {
    let mut file = fs::File::create(file_path)?;
    file.write_all(value.as_bytes())
}
