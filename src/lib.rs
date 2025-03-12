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
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(error) => writeln!(f, "{}", error),
            Error::ExtensionError => writeln!(f, "extension error"),
            Error::FailedCreateRenamedFolder => writeln!(f, "failed create renamed folder"),
            Error::NoParent => writeln!(f, "no parent"),
        }
    }
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
impl fmt::Display for PathSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Audio: {}", self.audio.display())?;
        writeln!(f, "Line: {}", self.line.display())?;
        Ok(())
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

#[derive(Debug)]
pub struct PathSets {
    work_dir: PathBuf,
    list: Vec<PathSet>,
    audio_extension: String,
    // line_extension: String,
}
impl fmt::Display for PathSets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in &self.list {
            writeln!(f, "{}", i)?;
        }
        Ok(())
    }
}
impl PathSets {
    /// create a new instance of PathSets.
    /// # Arguments
    /// * `dir` - The directory where the audio and line files are located.
    /// * `audio_extension` - The extension of the audio file.
    /// * `line_extension` - The extension of the line file.
    pub fn new<P: AsRef<Path>, S: AsRef<str>>(
        dir: P,
        audio_extension: S,
        line_extension: S,
        use_recognition: bool,
    ) -> Result<Self, Error> {
        let path_list = get_file_list(&dir, audio_extension.as_ref(), line_extension.as_ref())?;
        
        
        let tmp_list =
            build_path_sets(path_list, audio_extension.as_ref(), line_extension.as_ref())?;
        let mut new = PathSets {
            work_dir: dir.as_ref().to_path_buf(),
            list: tmp_list,
            audio_extension: audio_extension.as_ref().to_string(),
            // line_extension: line_extension.as_ref().to_string(),
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

            // セリフファイルから読み込んだwavファイルのパスを生成する
            i.changed_audio = tmp_line.map(|p| {
                self.work_dir
                    .join("renamed")
                    .join(if p.is_empty() { "_" } else { &p })
                    .with_extension(&self.audio_extension) // default is wav
            });
        }
    }

    /// return list of path to be changed(not renamed yet)
    pub fn check(&self) -> Result<ListForCheck, Error> {
        let mut tmp = ListForCheck::new();
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

    /// rename audio files
    pub fn rename(&mut self) -> Result<(), Error> {
        create_renamed_folder(&self.work_dir)?;
        for i in &mut self.list {
            let changed_audio = match i.changed_audio.as_ref() {
                Some(v) => v,
                None => continue,
            };
            if fs::rename(&i.audio, changed_audio).is_err() {
                i.changed_audio = None
            };
        }
        Ok(())
    }
}

/// Get file list
/// audio_extentionかline_extentionにかかるファイルのみのリスト
fn get_file_list<P: AsRef<Path>>(
    dir: P,
    audio_ext: &str,
    line_ext: &str,
) -> Result<Vec<DirEntry>, Error> {
    let filtered_list: Vec<_> = fs::read_dir(dir)
        .map_err(Error::IoError)?
        .filter_map(|e| {
            e.ok().filter(|ee| {
                ee.path().extension().is_some_and(|n| {
                    n.to_str().is_some_and(|f| f == audio_ext)
                        || n.to_str().is_some_and(|f| f == line_ext)
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

fn build_path_sets(
    list: Vec<DirEntry>,
    audio_ext: &str,
    line_ext: &str,
) -> Result<Vec<PathSet>, Error> {
    let mut tmp_list = Vec::<PathSet>::new();

    for i in list {
        let path = i.path();
        let line_path = match path
            .extension()
            .and_then(|ext| ext.to_str())
            .filter(|&ext| ext == audio_ext)
            .map(|_| path.with_extension(line_ext))
        {
            Some(v) => v,
            None => continue,
        };
        if !line_path.exists() {
            return Err(Error::NoParent); // あとで
        }
        tmp_list.push(PathSet::new(path, line_path));
    }
    Ok(tmp_list)
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
                .args([
                    "-ExecutionPolicy",
                    "Bypass",
                    "-NoExit",
                    "-File",
                    windows_path.to_str().unwrap(),
                ])
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
                    .join("assets"),
                "wav",
                "txt"
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
        let a = PathSets::new(&cud, "wav", "txt", false).unwrap();
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
        let a = PathSets::new(&cud, "wav", "txt", false)
            .unwrap()
            .check()
            .unwrap();
        println!("{}", a);
    }

    #[test]
    fn test_rename() {
        ready();
        let cud = env::current_dir()
            .unwrap()
            .join("assets_for_test")
            .join("assets");
        PathSets::new(&cud, "wav", "txt", false)
            .unwrap()
            .rename()
            .unwrap();
        for i in fs::read_dir(cud.join("renamed")).unwrap() {
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
        let mut b = PathSets::new(&cud, "wav", "txt", false).unwrap();
        b.ready_rename();
        println!("{:?}", b.list);
    }
}
