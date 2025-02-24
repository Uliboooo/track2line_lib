use std::{
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
}

#[derive(Debug)]
struct PathSet {
    audio: PathBuf,
    line: PathBuf,
}
impl PathSet {
    fn new<P: AsRef<Path>>(audio: P, line: P) -> Self {
        Self {
            audio: audio.as_ref().to_path_buf(),
            line: line.as_ref().to_path_buf(),
        }
    }
}

pub struct PathSets {
    list: Vec<PathSet>,
}
impl PathSets {
    pub fn new<P: AsRef<Path>>(dir: P) -> Self {
        let path_list = get_file_list(dir).unwrap();
        let mut tmp_list = Vec::<PathSet>::new();
        for i in path_list {
            let path = i.path();
            // TODO: remove unwrap.
            let text_path = match path.extension().unwrap().to_str().unwrap() {
                "wav" => path.with_extension("txt"),
                _ => continue,
            };
            if !text_path.exists() {
                panic!("line file is notfound!")
            }
            tmp_list.push(PathSet::new(path, text_path));
        }
        Self { list: tmp_list }
    }
    // pub fn check(&self, )
}

fn get_file_list<P: AsRef<Path>>(dir: P) -> Result<Vec<DirEntry>, Error> {
    let filtered_list: Vec<_> = fs::read_dir(dir)
        .map_err(Error::IoError)?
        .filter_map(|e| {
            e.ok().filter(|ee| {
                ee.path()
                    .extension()
                    .is_some_and(|n| n.to_str().unwrap() == "wav" || n.to_str().unwrap() == "txt")
            })
        })
        .collect();
    Ok(filtered_list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, process};

    fn ready() -> bool {
        let p = env::current_dir().unwrap().to_path_buf();
        let windows_path = p.join("ready_test_files.ps1");
        let unix_path = p.join("ready_test_files.sh");
        if cfg!(target_os = "windows") {
            if process::Command::new("powershell")
                .args(["-NoExit", "-File", windows_path.to_str().unwrap()])
                .status()
                .unwrap()
                .success()
            {
                println!("Ready!(windows)");
                true
            } else {
                println!("Not ready!(windows)");
                false
            }
        } else {
            if process::Command::new("sh")
                .args([unix_path.to_str().unwrap()])
                .status()
                .unwrap()
                .success()
            {
                println!("Ready!(unix)");
                true
            } else {
                println!("Not ready!(unix)");
                false
            }
        }
    }

    #[test]
    fn a() {
        ready();
        println!(
            "{:?}",
            get_file_list(
                env::current_dir()
                    .unwrap()
                    .join("assets_for_test")
                    .join("assets")
            )
            .unwrap()
        );
    }

    #[test]
    fn b() {
        ready();
        let cud = env::current_dir()
            .unwrap()
            .join("assets_for_test")
            .join("assets");
        let a = PathSets::new(cud);
        for i in a.list {
            println!("{:?}", i);
        }
    }
}
