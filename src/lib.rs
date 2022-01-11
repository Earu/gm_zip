#![feature(c_unwind)]

use std::{path::{PathBuf, Path, Component}, fs::File, io::{copy, Error, ErrorKind}};
use zip::{ZipWriter, write::FileOptions};

#[macro_use] extern crate gmod;

fn get_game_dir() -> String {
    let exe_path = std::env::current_exe().unwrap();
    let mut path = exe_path.parent().unwrap();
    if path.to_str().unwrap().ends_with("win64") { // if its in win64 folder get back one more folder
        path = path.parent().unwrap();
    }

    path = path.parent().unwrap(); // remove the bin folder
    let mut path_str = String::from(path.to_str().unwrap());
    path_str.push_str("\\garrysmod\\");
    path_str.replace("\\", "/")
}

struct ArchiveFile {
    actual_path: PathBuf,
    archive_path: String,
}

#[lua_function]
unsafe fn zip(lua: gmod::lua::State) -> i32 {
    let base_path = get_game_dir();
    let passed_output_path = lua.check_string(1);
    let output_path = {
        let root_path = Path::new(&base_path);
        let local_path = Path::new(passed_output_path.as_ref());
        root_path.join(local_path)
    };

    let paths = {
        lua.check_table(2);

        let mut paths: Vec<ArchiveFile> = Vec::new();
        lua.push_value(2);
        for i in 1..=lua.len(-1) {
            lua.raw_geti(-2, i);

            match lua.get_type(-1) {
                "string" => {
                    let relative_path = lua.check_string(-1);
                    paths.push(ArchiveFile {
                        actual_path: {
                            let root_path = Path::new(&base_path);
                            let local_path = Path::new(relative_path.as_ref());
                            root_path.join(local_path)
                        },
                        archive_path: relative_path.into_owned(),
                    });
                },
                "table" => {
                    lua.get_field(-1, lua_string!("Path"));
                    let relative_path = lua.check_string(-1);
                    lua.pop();

                    lua.get_field(-1, lua_string!("ArchivePath"));
                    let wanted_path = lua.check_string(-1);
                    lua.pop();

                    paths.push(ArchiveFile {
                        actual_path: {
                            let root_path = Path::new(&base_path);
                            let local_path = Path::new(relative_path.as_ref());
                            root_path.join(local_path)
                        },
                        archive_path: wanted_path.into_owned(),
                    });
                }
                _ => ()
            }

            lua.pop();
        }

        if paths.is_empty() {
            return 0;
        }

        paths
    };

    fn archive_files(output: PathBuf, archived_files: &[ArchiveFile]) -> Result<(), std::io::Error> {
        let file = File::create(output)?;
        let mut zip = ZipWriter::new(file);

        let opts = FileOptions::default().large_file(true);
        for archived_file in archived_files {
            if archived_file.actual_path.components().into_iter().any(|c| c == Component::ParentDir) {
                return Err(Error::new(ErrorKind::Other, "path transversal"));
            } else {
                match archived_file.actual_path.to_str() {
                    Some(path_str) => {
                        zip.start_file(archived_file.archive_path.as_str(), opts)?;
                        let file = &mut File::open(path_str)?;
                        copy(file, &mut zip)?;
                    }
                    None => {
                        return Err(Error::new(ErrorKind::Other, "path contains invalid UTF-8"));
                    }
                }
            }
        }

        zip.finish()?;
        Ok(())
    }

    if let Err(err) = archive_files(output_path, &paths) {
        lua.error(&format!("{}", err));
    }

    0
}

#[gmod13_open]
unsafe fn gmod13_open(lua: gmod::lua::State) -> i32 {
    lua.push_function(zip);
    lua.set_global(lua_string!("Zip"));
    0
}

#[gmod13_close]
unsafe fn gmod13_close(_: gmod::lua::State) -> i32 {
    0
}