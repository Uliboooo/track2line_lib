# track2line lib

**this lib version is 0.1.x**

## tools

- cli ver. https://github.com/Uliboooo/track2line
- gui ver. https://github.com/Uliboooo/track2line_gui

## known Issues

- maybe don't work when empty line file is more than one file. https://github.com/Uliboooo/track2line_lib/issues/1

## usage

```toml:
track2line_lib = { git = "https://github.com/Uliboooo/track2line_lib", tag = "0.3.0" }
```

```rust: usage
use track2line_lib;

fn main() {
    let path = PathBuf::from("target_folder_path");
    let mut sets = track2line_lib::PathSets::new(&path, "wav", "txt").unwrap();

    // print list of path to be changed(not renamed yet)
    println!("{}", sets.check().unwrap());

    // rename
    sets.rename().unwrap();
}
```

## description

this is a library for converting track files to line files.
