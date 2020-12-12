mod file;
mod path;

use std::fs::{self, Metadata};

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

pub fn purge(path: &str) -> program::Result {
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
        file::shred(path).map_err(|e| format!("'{}': {}", path, e))?;
    }

    file::erase(path).map_err(|e| format!("'{}': {}", path, e))?;

    Ok(0)
}
