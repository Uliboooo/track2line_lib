use std::{
    fmt,
    fs::{self, ReadDir},
    io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum T2LError {
    IoError(io::Error),
}
impl fmt::Display for T2LError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            T2LError::IoError(e) => write!(f, "{}", e),
        }
    }
}

struct PathSet {
    old: PathBuf,
    new: PathBuf,
}

pub struct PathSets {
    list: Vec<PathSet>,
}
impl PathSets {
    pub fn new<P: AsRef<Path>>(dir: P) -> Result<Self, T2LError> {
        let mut list = Self { list: Vec::new() };
        let file_list = get_file_list(dir)?;

        for f in file_list {
            let entry_path = f.map_err(T2LError::IoError)?.path();
            let ext = match entry_path.extension() {
                Some(e) => match e.to_str() {
                    Some(ee) => ee,
                    None => continue,
                },
                None => continue,
            };
            match ext {
                "wav" => {
                    let text_path = entry_path.parent().unwrap().join("text.txt"); // TODO: unwrap
                    let text_content = fs::read_to_string(&text_path).map_err(T2LError::IoError)?;
                    let audio_path = entry_path.join(format!("{}.mp3", &text_content[0..=20]));
                    list.list.push(PathSet {
                        old: entry_path,
                        new: audio_path,
                    });
                }
                _ => continue,
            }
        }
        Ok(list)
    }

    pub fn get_check_list(&self) -> Vec<(String, String)> {
        let mut tmp_list = Vec::<(String, String)>::new();
        for set in &self.list {
            tmp_list.push((
                set.old.to_string_lossy().to_string(),
                set.new.to_string_lossy().to_string(),
            ));
        }
        tmp_list
    }

    /// Rename all paths in the list.
    pub fn rename(&self) -> Result<(), T2LError> {
        for set in &self.list {
            fs::rename(&set.old, &set.new).map_err(T2LError::IoError)?;
        }
        Ok(())
    }
}

// fn rename_audio(&mut )

fn get_file_list<P: AsRef<Path>>(dir: P) -> Result<ReadDir, T2LError> {
    Ok(fs::read_dir(dir).map_err(T2LError::IoError)?)
}

#[test]
fn test_get_file_list() {
    eprintln!("{:?}", get_file_list("assets_for_test/assets/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    #[test]
    fn global_test() {
        assert!(process::Command::new("sh")
            .arg("ready_test_files.sh")
            .status()
            .unwrap()
            .success());
        let a = match PathSets::new("assets_for_test/assets/") {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{}", e);
                panic!("Failed to create PathSets: {}", e)
            }
        };
        let b = a.get_check_list();
        for i in b {
            println!("* {:width$} ---> {}", i.0, i.1, width = 20);
        }
        assert!(a.rename().is_ok());
    }
}
