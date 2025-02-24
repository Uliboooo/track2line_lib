use std::{
    fs::{self, DirEntry},
    io,
    path::Path,
};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
}

pub struct A<P: AsRef<Path>> {
    audio: P,
    line: P,
}

impl<P: AsRef<Path>> A<P> {
    pub fn new(dir: P) -> Self {
        todo!()
    }
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
}
