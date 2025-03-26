# track2line lib

**this lib version is 0.1.x**

## about transcription mod

this is still doesn't work properly.therefore, if you use this, turn on "experimental" in feature flag.

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

### use config (feature)

Cargo.toml
```toml
[dependencies]
track2line_lib = { git = "https://github.com/Uliboooo/track2line_lib", tag = "v0.8.0", features = ["config"]}
```

```rust
use track2line_lib;

// use default config(wav, txt)
let default_config: Config = track2line_lib::config::Config::default();

//save
default_config.save()

// load
// もしconfigファイルがない場合、デフォルト設定でファイルを作成します
let setting = track2line_lib::config::Config::load();
```

## description

this is a library for converting track files to line files.
