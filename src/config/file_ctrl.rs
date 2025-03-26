use std::{
    fs::{self, read_to_string},
    io::{Error, Write},
    path::Path,
};

/// ファイルを上書きで保存する。存在しないなら作成
pub fn save_content<S: AsRef<str>, P: AsRef<Path>>(content: S, path: P) -> Result<(), Error> {
    let save_content = content.as_ref();
    let mut file = fs::OpenOptions::new()
        // .read(true)
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)?;

    writeln!(file, "{}", save_content)?;

    Ok(())
}

/// read_to_stringをラップしただけ
pub fn load_content<P: AsRef<Path>>(path: &P) -> Result<String, Error> {
    read_to_string(path.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn save_test() {
        let path = PathBuf::from("assets_for_test/for_file_ctrl/test.txt");
        save_content("test", &path).unwrap();
        println!("{}", load_content(&path).unwrap())
    }

    #[test]
    fn load_test() {
        let path = PathBuf::from("assets_for_test/for_file_ctrl/test.txt");
        let content = load_content(&path).unwrap();
        println!("{}", content);
    }
}
