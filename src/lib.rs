#![feature(c_unwind)]

use std::{path::{PathBuf, Path, Component}, fs::File, io::{copy, Error, ErrorKind}};
use zip::{ZipWriter, write::FileOptions};
use glob::glob;

#[macro_use] extern crate gmod;
extern crate glob;

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

fn is_path_transversal(path: &PathBuf) -> bool {
    path.components().into_iter().any(|c| c == Component::ParentDir)
}

struct ArchiveFile {
    actual_path: PathBuf,
    archive_path: String,
}

fn archive_files(output: PathBuf, archived_files: &[ArchiveFile]) -> Result<(), std::io::Error> {
    let file = File::create(output)?;
    let mut zip = ZipWriter::new(file);

    let opts = FileOptions::default().large_file(true);
    for archived_file in archived_files {
        if is_path_transversal(&archived_file.actual_path) {
            return Err(Error::new(ErrorKind::Other, "path transversal"));
        } else {
            match archived_file.actual_path.to_str() {
                Some(path_str) => {
                    if archived_file.actual_path.exists() && archived_file.actual_path.is_file() && !is_path_transversal(&archived_file.actual_path) {
                        zip.start_file(archived_file.archive_path.as_str(), opts)?;
                        let file = &mut File::open(path_str)?;
                        copy(file, &mut zip)?;
                    }
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

fn archive_folder(output: PathBuf, folder_path: PathBuf, delete_original: bool) -> Result<(), std::io::Error> {
    if is_path_transversal(&folder_path) || is_path_transversal(&output) {
        return Err(Error::new(ErrorKind::Other, "path transversal"));
    }

    if !folder_path.exists() {
        return Err(Error::new(ErrorKind::NotFound, "folder not found"));
    }

    let mut paths: Vec<ArchiveFile> = Vec::new();
    let pattern = {
        let mut root_path = String::from(folder_path.to_str().unwrap());
        if !root_path.ends_with('/') {
            root_path.push('/');
        }

        root_path.push_str("**/*");
        root_path
    };

    for e in glob(pattern.as_str()).expect("Failed to read glob pattern") {
        let archive_path =  e.as_ref().unwrap().strip_prefix(folder_path.to_str().unwrap()).unwrap().to_str().unwrap();
        paths.push(ArchiveFile {
            actual_path: PathBuf::from(e.as_ref().unwrap()),
            archive_path: archive_path
                .replace("\\", "/")
                .replace(".lua.txt", ".lua"), // we all know what that means
        });
    }

    match archive_files(output, &paths) {
        Ok(_) => {
            if delete_original {
                std::fs::remove_dir_all(folder_path)
            } else {
                Ok(())
            }
        },
        Err(e) => Err(e)
    }
}

unsafe fn zip_folder(lua: gmod::lua::State, output_path: PathBuf) -> i32 {
    let base_path = get_game_dir();
    let passed_folder_path = lua.check_string(2);
    let folder_path = {
        let root_path = Path::new(&base_path);
        let local_path = Path::new(passed_folder_path.as_ref());
        root_path.join(local_path)
    };

    let delete_original = {
        if lua.get_top() >= 3 {
            lua.check_boolean(3)
        } else {
            false
        }
    };

    if let Err(err) = archive_folder(output_path, folder_path, delete_original) {
        lua.error(&format!("{}", err));
    }

    0
}

unsafe fn zip_files(lua: gmod::lua::State, output_path: PathBuf) -> i32 {
    lua.check_table(2);
    lua.push_value(2);

    let mut paths: Vec<ArchiveFile> = Vec::new();
    let base_path = get_game_dir();
    for i in 1..=lua.len(-1) {
        lua.raw_geti(-2, i);

        match lua.get_type(-1) {
            "string" => {
                let relative_path = lua.check_string(-1).into_owned();
                paths.push(ArchiveFile {
                    actual_path: {
                        let root_path = Path::new(&base_path);
                        let local_path = Path::new(relative_path.as_str());
                        root_path.join(local_path)
                    },
                    archive_path: relative_path,
                });
            },
            "table" => {
                lua.get_field(-1, lua_string!("Path"));
                lua.get_field(-2, lua_string!("ArchivePath"));

                let relative_path = lua.check_string(-2).into_owned();
                let wanted_path = lua.check_string(-1).into_owned();

                lua.pop_n(2);

                paths.push(ArchiveFile {
                    actual_path: {
                        let root_path = Path::new(&base_path);
                        let local_path = Path::new(relative_path.as_str());
                        root_path.join(local_path)
                    },
                    archive_path: wanted_path,
                });
            }
            _ => ()
        }

        lua.pop();
    }

    if paths.is_empty() {
        return 0;
    }

    if let Err(err) = archive_files(output_path, &paths) {
        lua.error(&format!("{}", err));
    }

    0
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

    match lua.get_type(2) {
        "string" => zip_folder(lua, output_path),
        _ => zip_files(lua, output_path),
    }
}

#[gmod13_open]
unsafe fn gmod13_open(lua: gmod::lua::State) -> i32 {
    lua.new_table();
    lua.push_function(zip);
    lua.set_global(lua_string!("Zip"));

    0
}

#[gmod13_close]
unsafe fn gmod13_close(_: gmod::lua::State) -> i32 {
    0
}