use getopt::Opt;
use rand::random;
use std::{
    error::Error,
    fs::{self, File, Metadata},
    io::{self, Seek, SeekFrom, Write},
    path::Path,
};

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        use std::os::linux::fs::MetadataExt;

        fn hardlinks(meta: &Metadata) -> u64 {
            meta.st_nlink()
        }
    } else if #[cfg(unix)] {
        use std::os::unix::fs::MetadataExt;

        fn hardlinks(meta: &Metadata) -> u64 {
            meta.nlink()
        }
    } else {
        fn hardlinks(_meta: &Metadata) -> u64 {
            1
        }
    }
}

const SHRED_BUFFER_MAX: usize = 262_144;

program::main!("purge");

fn usage_line() -> String {
    format!("Usage: {} [-h] path [path ...]", program::name("purge"))
}

fn print_usage() -> program::Result {
    println!("{}", usage_line());
    println!("  -h   display this help");
    Ok(0)
}

fn log(msg: &str) {
    eprintln!("{}: {}", program::name("purge"), msg)
}

fn program() -> program::Result {
    let mut args = program::args();
    let mut opts = getopt::Parser::new(&args, "h");

    loop {
        match opts.next().transpose()? {
            None => break,
            Some(opt) => match opt {
                Opt('h', None) => return print_usage(),
                _ => unreachable!(),
            },
        }
    }

    let args = args.split_off(opts.index());
    if args.is_empty() {
        eprintln!("{}", usage_line());
        return Ok(1);
    }

    for arg in args {
        purge(&arg)?;
    }

    Ok(0)
}

fn purge(path: &str) -> program::Result {
    let attrs = fs::symlink_metadata(path).map_err(|e| format!("'{}': {}", path, e))?;

    let mut perms = attrs.permissions();
    if perms.readonly() {
        perms.set_readonly(false);
        fs::set_permissions(path, perms).map_err(|e| format!("'{}': {}", path, e))?;
    }

    if attrs.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| format!("'{}': {}", path, e))? {
            let entry = entry.map_err(|e| format!("'{}': {}", path, e))?;
            purge(&entry.path().to_string_lossy())?;
        }
    } else if attrs.is_file() && hardlinks(&attrs) == 1 {
        shred(path).map_err(|e| format!("'{}': {}", path, e))?;
    }

    erase(path).map_err(|e| format!("'{}': {}", path, e))?;

    Ok(0)
}

fn shred(path: &str) -> program::Result {
    let size = fs::symlink_metadata(&path)?.len();
    let file = fs::OpenOptions::new().write(true).open(path)?;

    for pass in 1..=3 {
        let r = random::<u8>();
        log(&format!("{}: pass {}/4 (random)...", path, pass));
        shred_with(r, &file, size)?;
    }

    log(&format!("{}: pass 4/4 (000000)...", path));
    shred_with(0, &file, size)?;

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

fn erase(path: &str) -> program::Result {
    let (dir, mut oldname) = split_path(&path)?;
    let mut newname = String::from_utf8(oldname.as_bytes().iter().map(|_| b'0').collect())?;

    if newname != oldname {
        rename(&dir, &oldname, &newname)?;
    }

    while newname != "0" {
        oldname = newname;
        newname = oldname.strip_suffix("0").unwrap().to_string();

        rename(&dir, &oldname, &newname)?;
    }

    let newpath = join_path(&dir, &newname)?;
    if fs::symlink_metadata(&newpath)?.is_dir() {
        fs::remove_dir(newpath)?;
    } else {
        fs::remove_file(newpath)?;
    }

    log(&format!("{}: removed", path));

    Ok(0)
}

fn rename(dir: &Option<String>, from: &str, to: &str) -> program::Result {
    let from = join_path(&dir, &from)?;
    let to = join_path(&dir, &to)?;

    fs::rename(&from, &to)?;

    log(&format!("{}: renamed to {}", from, to));

    Ok(0)
}

fn split_path(path: &str) -> Result<(Option<String>, String), Box<dyn Error>> {
    let path = Path::new(path);
    let head = path.parent().map(|p| p.to_string_lossy().into_owned());
    let tail = match path.file_name() {
        None => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("bad path: '{}'", path.display()),
            )))
        }
        Some(p) => p.to_string_lossy().into_owned(),
    };

    Ok((head, tail))
}

fn join_path(dir: &Option<String>, name: &str) -> Result<String, Box<dyn Error>> {
    Ok(match dir {
        None => name.to_string(),
        Some(p) => Path::new(p).join(name).to_string_lossy().into_owned(),
    })
}
