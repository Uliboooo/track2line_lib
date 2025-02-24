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
            let line_path = match path.extension().unwrap().to_str().unwrap() {
                "wav" => path.with_extension("txt"),
                _ => continue,
            };
            if !line_path.exists() {
                panic!("line file is notfound!")
            }
            tmp_list.push(PathSet::new(path, line_path));
        }
        Self { list: tmp_list }
    }
    pub fn check(&self) -> Result<Vec<(String, String)>, Error> {
        let mut tmp = Vec::<(String, String)>::new();
        for i in &self.list {
            let tmp_line = fs::read_to_string(&i.line)
                .unwrap()
                .chars()
                .take(20)
                .collect::<String>()
                .trim()
                .to_string();

            let new_audio_path = i.audio.with_file_name(tmp_line).with_extension("wav");

            tmp.push((
                i.audio.file_name().unwrap().to_string_lossy().to_string(),
                new_audio_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            ));
        }
        Ok(tmp)
    }
}

// fn is_

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
            process::Command::new("powershell")
                .args(["-NoExit", "-File", windows_path.to_str().unwrap()])
                .status()
                .unwrap()
                .success()
        } else {
            process::Command::new("sh")
                .args([unix_path.to_str().unwrap()])
                .status()
                .unwrap()
                .success()
        }
    }

    #[test]
    fn test_ready_function_execution() {
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
    fn test_ready_and_list_assets() {
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

    #[test]
    fn test_check() {
        ready();
        let cud = env::current_dir()
            .unwrap()
            .join("assets_for_test")
            .join("assets");
        let a = PathSets::new(cud).check().unwrap();
        for i in a {
            println!("* {:width$} ---> {}", i.0, i.1, width = 20);
        }
    }
}
