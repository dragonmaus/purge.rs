#![cfg_attr(windows_by_handle, feature(windows_by_handle))]

mod file;
mod path;

use std::{
    cmp::Ordering,
    fs::{self, DirEntry, Metadata},
    io,
};

cfg_if::cfg_if! {
    if #[cfg(windows_by_handle)] {
        use std::os::windows::fs::MetadataExt;

        fn hardlinks(meta: &Metadata) -> u64 {
            match meta.number_of_links() {
                None => 1,
                Some(n) => n as u64,
            }
        }
    } else if #[cfg(target_os = "linux")] {
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

pub fn purge(path: &str, verbosity: u8) -> program::Result {
    let attrs = fs::symlink_metadata(path).map_err(|e| format!("'{}': {}", path, e))?;

    let mut perms = attrs.permissions();
    if perms.readonly() {
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(false);
        fs::set_permissions(path, perms).map_err(|e| format!("'{}': {}", path, e))?;
    }

    if attrs.is_dir() {
        let mut entries = fs::read_dir(path)
            .map_err(|e| format!("'{}': {}", path, e))?
            .collect::<Vec<io::Result<DirEntry>>>();

        entries.sort_by(|a, b| {
            if a.is_err() || b.is_err() {
                return Ordering::Equal;
            }

            let na = a.as_ref().unwrap().file_name();
            let nb = b.as_ref().unwrap().file_name();

            let za = na.to_string_lossy().chars().all(|c| c == '0');
            let zb = nb.to_string_lossy().chars().all(|c| c == '0');

            if za && !zb {
                return Ordering::Less;
            }

            if zb && !za {
                return Ordering::Greater;
            }

            na.cmp(&nb)
        });

        for entry in entries {
            let entry = entry.map_err(|e| format!("'{}': {}", path, e))?;
            purge(&entry.path().to_string_lossy(), verbosity)?;
        }
    } else if attrs.is_file() && hardlinks(&attrs) == 1 {
        file::shred(path, verbosity).map_err(|e| format!("'{}': {}", path, e))?;
    }

    file::erase(path, verbosity).map_err(|e| format!("'{}': {}", path, e))?;

    Ok(0)
}
