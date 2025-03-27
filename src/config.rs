mod file_ctrl;

use home::{self};
use serde::{Deserialize, Serialize};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Error {
    FailedGetHome,
    IoErr(io::Error),
    FailedToString,
    FailedSave,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::FailedGetHome => writeln!(f, "failed get home dir"),
            Error::IoErr(error) => writeln!(f, "io error: {}", error),
            Error::FailedToString => writeln!(f, "failed convert to string from Config struct"),
            Error::FailedSave => writeln!(f, "failed save Config"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub audio_extension: String,
    pub txt_extension: String,
}
impl Default for Config {
    /// "wav"と"txt"で初期化
    fn default() -> Self {
        Self {
            audio_extension: "wav".to_string(),
            txt_extension: "txt".to_string(),
        }
    }
}
impl Config {
    pub fn new(audio_ex: &str, txt_ex: &str) -> Result<Self, io::Error> {
        Ok(Self {
            audio_extension: audio_ex.to_string(),
            txt_extension: txt_ex.to_string(),
        })
    }

    /// configファイルがない or 空の場合、デフォルト設定で作成
    pub fn load() -> Result<Self, Error> {
        if file_is_empty(get_config_path()?)? || !get_config_path()?.exists() {
            let default_c = Config::default();
            default_c.save()?;
        }

        let a = file_ctrl::load_content(&get_config_path()?).unwrap();
        Ok(toml::from_str(&a).unwrap())
    }

    // 現在の設定をファイルに書き込む
    pub fn save(&self) -> Result<(), Error> {
        let config_str = toml::to_string(self).map_err(|_| Error::FailedToString)?;
        // file_ctrl::save_content(config_str, &get_config_path()?.map_err(Error::IoError)?)?
        file_ctrl::save_content(config_str, &get_config_path()?).map_err(Error::IoErr)
    }

    pub fn set_audio_ext(&mut self, new_ext: &str) {
        self.audio_extension = new_ext.to_string();
    }

    pub fn set_txt_ext(&mut self, new_ext: &str) {
        self.txt_extension = new_ext.to_string();
    }

    pub fn set_both(&mut self, new_audio_ext: &str, new_txt_ext: &str) {
        self.set_audio_ext(new_audio_ext);
        self.set_txt_ext(new_txt_ext);
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let content = format!(
            "audio extension: {}\ntxt extension: {}\nthese config saved on {}",
            self.audio_extension,
            self.txt_extension,
            get_config_path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or("config file not found.".to_string())
        );
        writeln!(f, "{}", content)
    }
}

/// osごとの設定ファイルパスを返す
fn get_config_path() -> Result<PathBuf, Error> {
    if cfg!(test) {
        return Ok(PathBuf::from(
            "assets_for_test/config_test/test_config.toml",
        ));
    }

    let home_path = home::home_dir().ok_or(Error::FailedGetHome)?;

    let config_folder = if cfg!(target_os = "windows") {
        home_path.join("AppData").join("Local").join("track2line")
    } else if cfg!(target_os = "macos") {
        home_path
            .join("Library")
            .join("Application Support")
            .join("track2line")
    } else {
        // linux
        home_path.join(".config").join("track2line")
    };
    if !config_folder.exists() {
        fs::create_dir(&config_folder).map_err(Error::IoErr)?;
    }

    Ok(config_folder.join("config.toml"))
}

fn file_is_empty<P: AsRef<Path>>(path: P) -> Result<bool, Error> {
    let content = fs::read_to_string(path).map_err(Error::IoErr)?;
    Ok(content.is_empty())
}

/// 普通にユーザディレクトリ以降に/track2line用の設定ファイルを作るため注意
#[cfg(test)]
mod tests {
    use super::*;

    // デバッグ用に基本的にunwrap()
    #[test]
    fn new_config_test() {
        // let path = get_config_path().unwrap();
        let new_config = Config::default();
        new_config.save().unwrap();
    }

    #[test]
    fn show_path_for_test() {
        println!("{:?}", get_config_path());
        assert_eq!(
            PathBuf::from("assets_for_test/config_test/test_config.toml"),
            get_config_path().unwrap()
        );
    }

    #[test]
    fn load_config_test() {
        println!("{:?}", get_config_path());
        let loaded_config = Config::load().unwrap();
        println!("{:?}", loaded_config);
    }

    #[test]
    fn show_test() {
        let loaded_config = Config::load().unwrap();
        println!("{}", loaded_config)
    }
}
