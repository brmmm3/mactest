use std::fs;
use std::io::{Error, ErrorKind};
use std::time::SystemTime;
use std::{
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread,
};

use flume::Receiver;
use speedy::{Readable, Writable};

#[cfg(unix)]
use expanduser::expanduser;

#[macro_use]
extern crate serde_derive;

pub type ErrorsType = Vec<(String, String)>; // Tuple with file path and error message

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ReturnType {
    Base,
    Ext,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub root_path: PathBuf,
    pub sorted: bool,
    pub skip_hidden: bool,
    pub max_depth: usize,
    pub max_file_cnt: usize,
    pub dir_include: Option<Vec<String>>,
    pub dir_exclude: Option<Vec<String>>,
    pub file_include: Option<Vec<String>>,
    pub file_exclude: Option<Vec<String>>,
    pub case_sensitive: bool,
    pub return_type: ReturnType,
}

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[cfg_attr(
    any(feature = "bincode", feature = "json"),
    derive(Deserialize, Serialize)
)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DirEntry {
    pub path: String,
    pub is_symlink: bool,
    pub is_dir: bool,
    pub is_file: bool,
    pub st_ctime: Option<SystemTime>,
    pub st_mtime: Option<SystemTime>,
    pub st_atime: Option<SystemTime>,
    pub st_size: u64,
}

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[cfg_attr(
    any(feature = "bincode", feature = "json"),
    derive(Deserialize, Serialize)
)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DirEntryExt {
    pub path: String,
    pub is_symlink: bool,
    pub is_dir: bool,
    pub is_file: bool,
    /// Creation time in seconds as float
    pub st_ctime: Option<SystemTime>,
    /// Modification time in seconds as float
    pub st_mtime: Option<SystemTime>,
    /// Access time in seconds as float
    pub st_atime: Option<SystemTime>,
    /// Size of file / entry
    pub st_size: u64,
    /// File system block size
    pub st_blksize: u64,
    /// Number of used blocks on device / file system
    pub st_blocks: u64,
    /// File access mode / rights
    pub st_mode: u32,
    /// Number of hardlinks
    pub st_nlink: u64,
    /// User ID (Unix only)
    pub st_uid: u32,
    /// Group ID (Unix only)
    pub st_gid: u32,
    /// I-Node number (Unix only)
    pub st_ino: u64,
    /// Device number (Unix only)
    pub st_dev: u64,
    /// Device number (for character and block devices on Unix).
    pub st_rdev: u64,
}

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[cfg_attr(
    any(feature = "bincode", feature = "json"),
    derive(Deserialize, Serialize)
)]
#[derive(Debug, Clone, PartialEq)]
pub enum ScandirResult {
    DirEntry(DirEntry),
    DirEntryExt(DirEntryExt),
    Error((String, String)),
}

#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[cfg_attr(
    any(feature = "bincode", feature = "json"),
    derive(Deserialize, Serialize)
)]
#[derive(Debug, Clone, PartialEq)]
pub struct ScandirResults {
    pub results: Vec<ScandirResult>,
    pub errors: ErrorsType,
}

impl ScandirResults {
    pub fn new() -> Self {
        ScandirResults {
            results: Vec::new(),
            errors: Vec::new(),
        }
    }
}

pub fn check_and_expand_path<P: AsRef<Path>>(path_str: P) -> Result<PathBuf, Error> {
    #[cfg(unix)]
    let path_result = fs::canonicalize(expanduser(path_str.as_ref().to_str().unwrap()).unwrap());
    #[cfg(not(unix))]
    let path_result = fs::canonicalize(&path_str);
    let path = match path_result {
        Ok(p) => {
            if !p.exists() {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    path_str.as_ref().to_str().unwrap().to_string(),
                ));
            }
            p
        }
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };
    Ok(path)
}

#[derive(Debug)]
pub struct Scandir {
    // Options
    options: Options,
    store: bool,
    // Results
    entries: ScandirResults,
    duration: Arc<Mutex<f64>>,
    finished: Arc<AtomicBool>,
    // Internal
    thr: Option<thread::JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    rx: Option<Receiver<ScandirResult>>,
}

impl Scandir {
    pub fn new<P: AsRef<Path>>(root_path: P, store: Option<bool>) -> Result<Self, Error> {
        Ok(Scandir {
            options: Options {
                root_path: check_and_expand_path(root_path)?,
                sorted: false,
                skip_hidden: false,
                max_depth: usize::MAX,
                max_file_cnt: usize::MAX,
                dir_include: None,
                dir_exclude: None,
                file_include: None,
                file_exclude: None,
                case_sensitive: false,
                return_type: ReturnType::Base,
            },
            store: store.unwrap_or(true),
            entries: ScandirResults::new(),
            duration: Arc::new(Mutex::new(0.0)),
            finished: Arc::new(AtomicBool::new(false)),
            thr: None,
            stop: Arc::new(AtomicBool::new(false)),
            rx: None,
        })
    }

    pub fn duration(&mut self) -> f64 {
        *self.duration.lock().unwrap()
    }

    pub fn finished(&mut self) -> bool {
        *self.duration.lock().unwrap() > 0.0
    }

    pub fn finished2(&mut self) -> bool {
        *self.duration.lock().unwrap() != 0.0
    }
}
