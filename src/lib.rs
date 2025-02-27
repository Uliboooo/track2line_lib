use std::{
    fmt,
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ExtensionError,
    FailedCreateRenamedFolder,
    NoParent,
}

#[derive(Debug)]
struct PathSet {
    audio: PathBuf,
    changed_audio: Option<PathBuf>,
    line: PathBuf,
}
impl PathSet {
    fn new<P: AsRef<Path>>(audio: P, line: P) -> Self {
        Self {
            audio: audio.as_ref().to_path_buf(),
            changed_audio: None,
            line: line.as_ref().to_path_buf(),
        }
    }
}

pub struct ListForCheck(Vec<(Option<String>, Option<String>)>);

impl ListForCheck {
    fn new() -> Self {
        Self(Vec::<(Option<String>, Option<String>)>::new())
    }
}

impl fmt::Display for ListForCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (old, new) in &self.0 {
            // writeln!(f, "Audio: {}", old.as_ref().unwrap_or(&String::from("None")))?;
            // writeln!(f, "Line: {}", new.as_ref().unwrap_or(&String::from("None")))?;
            //         "* {:width$} ---> {:?}",
            //         i.0.unwrap(),
            //         i.1.unwrap(),
            //         width = 20
            writeln!(
                f,
                "* {:width$} ---> {}",
                old.as_ref().unwrap_or(&"None".to_string()),
                new.as_ref().unwrap_or(&"None".to_string()),
                width = 20
            )?;
        }
        Ok(())
    }
}

pub struct PathSets {
    work_dir: PathBuf,
    list: Vec<PathSet>,
}
impl PathSets {
    /// Create a new instance of PathSets.
    pub fn new<P: AsRef<Path>>(dir: &P) -> Result<Self, Error> {
        let path_list = get_file_list(dir)?;
        let mut tmp_list = Vec::<PathSet>::new();
        for i in path_list {
            let path = i.path();
            let line_path = match path
                .extension()
                .and_then(|ext| ext.to_str())
                .filter(|&ext| ext == "wav")
                .map(|_| path.with_extension("txt"))
            {
                Some(v) => v,
                None => continue,
            };
            if !line_path.exists() {
                panic!("line file is notfound!")
            }
            tmp_list.push(PathSet::new(path, line_path));
        }
        let mut new = PathSets {
            work_dir: dir.as_ref().to_path_buf(),
            list: tmp_list,
        };
        new.ready_rename();
        Ok(new)
    }

    fn ready_rename(&mut self) {
        for i in &mut self.list {
            let tmp_line = fs::read_to_string(&i.line)
                .map(|file_content| {
                    file_content
                        .chars()
                        .take(20)
                        .collect::<String>()
                        .trim()
                        .to_string()
                })
                .ok();

            // let line = "renamed".to_string().push_str(tmp_line.map_or(""));

            // セリフファイルから読み込んだwavファイルのパスを生成する
            // let new_audio_path = tmp_line.map(|p| i.audio.with_file_name(p).with_extension("wav"));
            i.changed_audio =
                tmp_line.map(|p| self.work_dir.join("renamed").join(p).with_extension("wav"));
        }
    }

    // pub type ListForCheck = Vec<(Option<String>, Option<String>)>;

    // TODO: Integrate the logic for generating new paths, which is included in `check()` and `rename()` later.
    pub fn check(&self) -> Result<ListForCheck, Error> {
        let mut tmp = ListForCheck::new();
        // for i in &self.list {
        //     let tmp_line = fs::read_to_string(&i.line)
        //         .map(|s| s.chars().take(20).collect::<String>().trim().to_string())
        //         .ok();

        //     let new_audio_path = tmp_line.map(|p| i.audio.with_file_name(p).with_extension("wav"));

        //     tmp.push((
        //         i.audio
        //             .file_name()
        //             .and_then(|f| Some(f.to_string_lossy().to_string())),
        //         new_audio_path.and_then(|f| {
        //             f.file_name()
        //                 .and_then(|f| Some(f.to_string_lossy().to_string()))
        //         }),
        //     ));
        // }
        for i in &self.list {
            tmp.0.push((
                i.audio.file_name().map(|f| f.to_string_lossy().to_string()),
                i.changed_audio
                    .as_ref()
                    .and_then(|f| f.file_name().map(|f| f.to_string_lossy().to_string())),
            ));
        }
        Ok(tmp)
    }

    pub fn rename(&mut self) -> Result<(), Error> {
        create_renamed_folder(&self.work_dir)?;
        // self.ready_rename();
        // for i in &self.list {
        //     let tmp_line = match fs::read_to_string(&i.line) {
        //         Ok(n) => n.chars().take(20).collect::<String>().trim().to_string(),
        //         Err(_) => continue,
        //     };

        //     let new_audio_path = i.audio.with_file_name(tmp_line).with_extension("wav");

        //     fs::rename(&i.audio, &new_audio_path).unwrap();
        // }
        for i in &self.list {
            // let new = &i.changed_audio.unwrap();
            fs::rename(&i.audio, i.changed_audio.as_ref().unwrap()).unwrap();
        }
        Ok(())
    }
}

fn get_file_list<P: AsRef<Path>>(dir: P) -> Result<Vec<DirEntry>, Error> {
    let filtered_list: Vec<_> = fs::read_dir(dir)
        .map_err(Error::IoError)?
        .filter_map(|e| {
            e.ok().filter(|ee| {
                ee.path().extension().is_some_and(|n| {
                    n.to_str().is_some_and(|f| f == "wav") || n.to_str().is_some_and(|f| f == "txt")
                })
            })
        })
        .collect();
    Ok(filtered_list)
}

fn create_renamed_folder<P: AsRef<Path>>(dir: P) -> Result<(), Error> {
    fs::create_dir(dir.as_ref().join("renamed")).map_err(|_| Error::FailedCreateRenamedFolder)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, process};

    #[test]
    fn ready_foo() {
        ready();
        let a = env::current_dir().unwrap().join("assets_for_test/assets");
        create_renamed_folder(a).unwrap();
    }

    /// run ready_test_files.ps1 or ready_test_files.sh for test ready
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
    fn ready_function() {
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
    fn list_assets() {
        ready();
        let cud = env::current_dir()
            .unwrap()
            .join("assets_for_test")
            .join("assets");
        let a = PathSets::new(&cud).unwrap();
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
        let a = PathSets::new(&cud).unwrap().check().unwrap();
        println!("{}", a);
        // for i in a.0 {
        //     println!(
        //         "* {:width$} ---> {:?}",
        //         i.0.unwrap(),
        //         i.1.unwrap(),
        //         width = 20
        //     );
        // }
    }

    #[test]
    fn test_rename() {
        ready();
        let cud = env::current_dir()
            .unwrap()
            .join("assets_for_test")
            .join("assets");
        PathSets::new(&cud).unwrap().rename().unwrap();
        for i in fs::read_dir(cud).unwrap() {
            println!("{:?}", i);
        }
    }

    #[test]
    fn test_init_rename_prep() {
        ready();
        let cud = env::current_dir()
            .unwrap()
            .join("assets_for_test")
            .join("assets");
        let mut b = PathSets::new(&cud).unwrap();
        b.ready_rename();
        println!("{:?}", b.list);
    }
}
