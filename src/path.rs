use std::{error::Error, io, path::Path};

pub fn join(dir: &Option<String>, name: &str) -> Result<String, Box<dyn Error>> {
    Ok(match dir {
        None => name.to_string(),
        Some(p) => Path::new(p).join(name).to_string_lossy().into_owned(),
    })
}

pub fn split(path: &str) -> Result<(Option<String>, String), Box<dyn Error>> {
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
