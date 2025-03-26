#[cfg(feature = "experimental")]
mod transcription;

#[cfg(feature = "config")]
pub mod config;

use std::{
    fmt,
    fs::{self},
    io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ExtensionError,
    FailedCreateRenamedFolder(io::Error),
    NoParent,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(error) => writeln!(f, "{}", error),
            Error::ExtensionError => writeln!(f, "extension error"),
            Error::FailedCreateRenamedFolder(e) => writeln!(f, "failed create renamed folder{}", e),
            Error::NoParent => writeln!(f, "no parent"),
        }
    }
}

#[derive(Debug)]
struct PathSet {
    audio_path: PathBuf,
    changed_audio_path: Option<PathBuf>,
    line: String,
}
impl PathSet {
    /// init時に変更後の`changed_audio_path`が取得できることはないため引数は以下のみ
    fn new<P: AsRef<Path>, S: AsRef<str>>(audio_path: P, line: S) -> Self {
        Self {
            audio_path: audio_path.as_ref().to_path_buf(),
            changed_audio_path: None,
            line: line.as_ref().to_string(),
        }
    }
}
impl fmt::Display for PathSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Audio: {}", self.audio_path.display())?;
        writeln!(f, "Line: {}", self.line)?;
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
            let oldd = &old
                .as_ref()
                .unwrap_or(&"None".to_string())
                .chars()
                .take(20)
                .collect::<String>();

            writeln!(
                f,
                "* {:width$} ---> {}",
                oldd,
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
    ) -> Result<Self, Error> {
        let filtered_path_list =
            get_file_list(&dir, audio_extension.as_ref(), line_extension.as_ref())?;

        let tmp_list = build_path_sets(
            filtered_path_list,
            audio_extension.as_ref(),
            line_extension.as_ref(),
        )?;

        let mut new = PathSets {
            work_dir: dir.as_ref().to_path_buf(),
            list: tmp_list,
            audio_extension: audio_extension.as_ref().to_string(),
        };
        new.ready_rename();
        Ok(new)
    }

    /// この関数はまだ正常に動作しません
    #[cfg(feature = "experimental")]
    pub fn new_transcription<P: AsRef<Path>, S: AsRef<str>>(
        dir: P,
        audio_extension: S,
        line_extension: S,
    ) -> Result<Self, Error> {
        let filtered_path_list =
            get_file_list(&dir, audio_extension.as_ref(), line_extension.as_ref())?;

        let path_set_list = filtered_path_list
            .iter()
            .filter(|f| f.extension().unwrap().to_str() == Some(audio_extension.as_ref()))
            .map(|audio_path| {
                let line = transcription::transcription("model_path", audio_path, Some("ja"));
                PathSet::new(audio_path, line)
            })
            .collect();

        let mut new = PathSets {
            work_dir: dir.as_ref().to_path_buf(),
            list: path_set_list,
            audio_extension: audio_extension.as_ref().to_string(),
        };
        new.ready_rename();
        Ok(new)
    }

    /// self.lineの内容を元にchanged_audio_pathをSome(path)に書き換え
    fn ready_rename(&mut self) {
        for i in &mut self.list {
            //build_path_sets()にてセリフが空の処理はしてあるためここでは不要
            i.changed_audio_path = Some(
                self.work_dir
                    .join("renamed")
                    .join(&i.line)
                    .with_extension(&self.audio_extension),
            );
        }
    }

    /// return list of path to be changed(not renamed yet)
    pub fn check(&self) -> Result<ListForCheck, Error> {
        let mut tmp = ListForCheck::new();
        for i in &self.list {
            tmp.0.push((
                i.audio_path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string()),
                i.changed_audio_path
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
            let changed_audio = match i.changed_audio_path.as_ref() {
                Some(v) => v,
                None => continue,
            };
            if fs::rename(&i.audio_path, changed_audio).is_err() {
                i.changed_audio_path = None
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
) -> Result<Vec<PathBuf>, Error> {
    Ok(fs::read_dir(&dir)
        .map_err(Error::IoError)?
        .filter_map(|entry| entry.ok())
        .map(|ok_entry| ok_entry.path())
        .filter(|entry| {
            entry.extension().unwrap() == audio_ext || entry.extension().unwrap() == line_ext
        })
        .collect())
}

fn create_renamed_folder<P: AsRef<Path>>(dir: P) -> Result<(), Error> {
    fs::create_dir(dir.as_ref().join("renamed")).map_err(Error::FailedCreateRenamedFolder)?;
    Ok(())
}

/// リスト中のオーディオファイルパスから、対応するテキストファイルからセリフを20文字にカットし、Vec<Pathset>として返す
fn build_path_sets(
    list: Vec<PathBuf>,
    audio_ext: &str,
    line_ext: &str,
) -> Result<Vec<PathSet>, Error> {
    let mut tmp_list = Vec::<PathSet>::new();
    let mut empty_count = 0;

    for path in list {
        if path.extension().unwrap() == audio_ext {
            // パスを探す
            let text_path = path.with_extension(line_ext);

            let mut empty = false;

            let line = if text_path.exists() {
                let tmp = fs::read_to_string(text_path)
                    .map_err(Error::IoError)?
                    .chars()
                    .take(20)
                    .collect::<String>()
                    .trim()
                    .to_string();
                if tmp.is_empty() {
                    // テキストファイルはあったが、セリフが空だった場合
                    empty = true;
                    format!("empty_{}", empty_count)
                } else {
                    // セリフがあった場合
                    tmp
                }
            } else {
                // テキストファイルがなかった場合
                empty = true;
                format!("empty_{}", empty_count)
            };

            if empty {
                empty_count += 1;
            }

            // empty_countによって変更になる場合があるためmut
            // let mut line = fs::read_to_string(text_path)
            //     .map_err(Error::IoError)?
            //     .chars()
            //     .take(20)
            //     .collect::<String>()
            //     .trim()
            //     .to_string();
            // if line.is_empty() {
            //     line = format!("empty_{}", empty_count);
            //     empty_count += 1;
            // }

            let new_set = PathSet::new(path, line);
            tmp_list.push(new_set);
        }
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
        let a = PathSets::new(&cud, "wav", "txt").unwrap();
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
        let a = PathSets::new(&cud, "wav", "txt").unwrap().check().unwrap();
        println!("{}", a);
    }

    #[test]
    fn test_rename() {
        ready();
        let cud = env::current_dir()
            .unwrap()
            .join("assets_for_test")
            .join("assets");
        PathSets::new(&cud, "wav", "txt").unwrap().rename().unwrap();
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
        let mut b = PathSets::new(&cud, "wav", "txt").unwrap();
        b.ready_rename();
        println!("{:?}", b.list);
    }
}
