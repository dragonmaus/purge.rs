extern crate rand;

use self::rand::random;
use crate::path;
use std::{
    fs::{self, File},
    io::{self, Seek, SeekFrom, Write},
};

const SHRED_BUFFER_MAX: usize = 262_144;

pub fn erase(path: &str, verbosity: u8) -> program::Result {
    let (dir, mut oldname) = path::split(path)?;
    let mut newname = String::from_utf8(oldname.as_bytes().iter().map(|_| b'0').collect())?;

    if newname != oldname {
        rename(&dir, &oldname, &newname, verbosity)?;
    }

    while newname != "0" {
        oldname = newname;
        newname = oldname.strip_suffix('0').unwrap().to_string();

        rename(&dir, &oldname, &newname, verbosity)?;
    }

    let newpath = path::join(&dir, &newname)?;
    if fs::symlink_metadata(&newpath)?.is_dir() {
        fs::remove_dir(newpath)?;
    } else {
        fs::remove_file(newpath)?;
    }

    if verbosity > 0 {
        log(&format!("{}: removed", path));
    }

    Ok(0)
}

pub fn shred(path: &str, verbosity: u8) -> program::Result {
    let size = fs::symlink_metadata(path)?.len();
    let file = fs::OpenOptions::new().write(true).open(path)?;

    for pass in 1..=3 {
        let r = random::<u8>();
        if verbosity > 1 {
            log(&format!("{}: pass {}/4 (random)...", path, pass));
        }
        shred_with(r, &file, size)?;
    }

    if verbosity > 1 {
        log(&format!("{}: pass 4/4 (000000)...", path));
    }
    shred_with(0, &file, size)?;

    Ok(0)
}

fn log(msg: &str) {
    eprintln!("{}: {}", program::name("purge"), msg)
}

fn rename(dir: &Option<String>, from: &str, to: &str, verbosity: u8) -> program::Result {
    let from = path::join(dir, from)?;
    let to = path::join(dir, to)?;

    if path::exists(&to) {
        return Err(io::Error::new(io::ErrorKind::Other, format!("file exists: {}", to)).into());
    }

    fs::rename(&from, &to)?;

    if verbosity > 1 {
        log(&format!("{}: renamed to {}", from, to));
    }

    Ok(0)
}

fn shred_with(byte: u8, mut file: &File, size: u64) -> program::Result {
    let mut remaining = size as usize;

    file.seek(SeekFrom::Start(0))?;

    while remaining >= SHRED_BUFFER_MAX {
        let buffer = [byte; SHRED_BUFFER_MAX];
        file.write_all(&buffer)?;
        remaining -= SHRED_BUFFER_MAX;
    }

    if remaining > 0 {
        let buffer = vec![byte; remaining];
        file.write_all(&buffer)?;
    }

    file.sync_data()?;

    Ok(0)
}
