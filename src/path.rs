use std::{fs, io, path::Path};

pub fn exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

pub fn join(dir: &Option<String>, name: &str) -> io::Result<String> {
    Ok(match dir {
        None => name.to_string(),
        Some(p) => Path::new(p).join(name).to_string_lossy().into_owned(),
    })
}

pub fn split(path: &str) -> io::Result<(Option<String>, String)> {
    let path = Path::new(path);
    let head = path.parent().map(|p| p.to_string_lossy().into_owned());
    let tail = match path.file_name() {
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("bad path: '{}'", path.display()),
            ))
        }
        Some(p) => p.to_string_lossy().into_owned(),
    };

    Ok((head, tail))
}
