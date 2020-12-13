use std::{cmp::Ordering, env, ffi::OsString, fs::File, io::Write, path::Path, process::Command};

fn main() {
    let version = Version::read().unwrap();

    let out = env::var_os("OUT_DIR").unwrap();
    let mut log = File::create(Path::new(&out).join("build.log")).unwrap();

    log.write_fmt(format_args!(
        "windows_by_handle = {}\n",
        enable_windows_by_handle(version)
    ))
    .unwrap();
}

fn enable_windows_by_handle(version: Version) -> bool {
    if cfg!(not(windows)) {
        return false;
    }

    if !version.nightly
        || version
            < (Version {
                major: 1,
                minor: 38,
                patch: 0,
                nightly: false,
                commit: "".to_string(),
                date: Date {
                    year: 2019,
                    month: 7,
                    day: 26,
                },
            })
    {
        return false;
    }

    println!("cargo:rustc-cfg=windows_by_handle");
    true
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
struct Date {
    year: u32,
    month: u32,
    day: u32,
}

impl Date {
    fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return Err(format!("unrecognised date string: {}", s));
        }

        let year = read_num(parts[0])?;
        let month = read_num(parts[1])?;
        let day = read_num(parts[2])?;

        Ok(Date { year, month, day })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    nightly: bool,
    commit: String,
    date: Date,
}

impl Version {
    fn read() -> Result<Self, String> {
        let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
        let output = Command::new(&rustc)
            .arg("--version")
            .output()
            .unwrap()
            .stdout;
        Version::parse(&String::from_utf8(output).unwrap())
    }

    fn parse(s: &str) -> Result<Self, String> {
        let words: Vec<&str> = s.split_whitespace().collect();
        if words.len() != 4 || words[0] != "rustc" {
            return Err(format!("unrecognised version string: {}", s));
        }

        let commit = match words[2].strip_prefix('(') {
            None => return Err(format!("unrecognised version string: {}", s)),
            Some(s) => s.to_string(),
        };
        let date = match words[3].strip_suffix(')') {
            None => return Err(format!("unrecognised version string: {}", s)),
            Some(s) => Date::parse(s)?,
        };

        let spec: Vec<&str> = words[1].split('-').collect();
        let nightly = spec.len() == 2 && spec[1] == "nightly";

        let parts: Vec<&str> = spec[0].split('.').collect();
        let major = read_num(parts[0])?;
        let minor = read_num(parts[1])?;
        let patch = read_num(parts[2])?;

        Ok(Version {
            major,
            minor,
            patch,
            nightly,
            commit,
            date,
        })
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut ord: Ordering;

        ord = self.major.cmp(&other.major);
        if ord != Ordering::Equal {
            return ord;
        }

        ord = self.minor.cmp(&other.minor);
        if ord != Ordering::Equal {
            return ord;
        }

        ord = self.patch.cmp(&other.patch);
        if ord != Ordering::Equal {
            return ord;
        }

        self.date.cmp(&other.date)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn read_num(s: &str) -> Result<u32, String> {
    let mut num = String::new();
    for c in s.chars() {
        if !c.is_digit(10) {
            break;
        }
        num.push(c);
    }
    num.parse::<u32>().map_err(|e| e.to_string())
}
