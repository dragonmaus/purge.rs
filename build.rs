extern crate regex;

use regex::Regex;
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
                nightly: true,
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
        let output = Command::new(rustc)
            .arg("--version")
            .output()
            .unwrap()
            .stdout;
        Version::parse(&String::from_utf8(output).unwrap())
    }

    fn parse(s: &str) -> Result<Self, String> {
        let re = Regex::new(r"^rustc (\d+)\.(\d+)\.(\d+)(?:-([^)]+))?(?: \(([[:xdigit:]]{9}) (\d{4}-\d{2}-\d{2})\))?").unwrap();

        if let Some(c) = re.captures(s) {
            let major = read_num(c.get(1).unwrap().as_str())?;
            let minor = read_num(c.get(2).unwrap().as_str())?;
            let patch = read_num(c.get(3).unwrap().as_str())?;
            let nightly = match c.get(4) {
                None => false,
                Some(m) => m.as_str() == "nightly",
            };
            let commit = match c.get(5) {
                None => "",
                Some(m) => m.as_str(),
            }
            .to_string();
            let date = Date::parse(match c.get(6) {
                None => "0-0-0",
                Some(m) => m.as_str(),
            })?;

            Ok(Version {
                major,
                minor,
                patch,
                nightly,
                commit,
                date,
            })
        } else {
            Err(format!("unrecognised version string: {}", s))
        }
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
        if !c.is_ascii_digit() {
            break;
        }
        num.push(c);
    }
    num.parse::<u32>().map_err(|e| e.to_string())
}
