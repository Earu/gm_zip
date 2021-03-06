#![feature(c_unwind)]

use std::{path::{PathBuf, Path, Component}, fs::{File, self}, io::{copy, Error, ErrorKind}};
use zip::{ZipWriter, write::FileOptions};
use glob::glob;

#[macro_use] extern crate gmod;
extern crate glob;

static GMOD_PATH_FOLDER: &str = "steamapps/common/GarrysMod";
fn get_game_dir() -> String {
    let exe_path = std::env::current_exe().unwrap();
    let str_path = String::from(exe_path.as_os_str().to_str().unwrap()).replace("\\", "/");
    let index = str_path.find(GMOD_PATH_FOLDER).unwrap();

    let mut path_str = String::from(&str_path[0..index + &GMOD_PATH_FOLDER.len()]);
    path_str.push_str("/garrysmod/");
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
        let err = format!("folder not found: {}", folder_path.to_str().unwrap());
        return Err(Error::new(ErrorKind::NotFound, err));
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

    match glob(pattern.as_str())
    {
        Ok(iter) => {
            for e in iter {
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
        },
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
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

fn unzip_archive(input_path: &PathBuf, output_path: Option<PathBuf>, delete_original: bool) -> Result<(), std::io::Error> {
    let file = File::open(input_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let base_path = match output_path {
        None => {
            let mut extension = String::from(".");
            extension.push_str(input_path.extension().unwrap().to_str().unwrap());
            PathBuf::from(input_path.to_str().unwrap().replace(extension.as_str(), ""))
        },
        Some(path) => path
    };

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => base_path.join(path.to_owned()),
            None => continue,
        };

        let comment = file.comment();
        if !comment.is_empty() {
            println!("File {} comment: {}", i, comment);
        }

        if (&*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            copy(&mut file, &mut outfile)?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    if delete_original {
        std::fs::remove_file(input_path)?;
    }

    Ok(())
}

#[lua_function]
unsafe fn unzip(lua: gmod::lua::State) -> i32 {
    let base_path = get_game_dir();
    let passed_input_path = lua.check_string(1);
    let input_path = {
        let root_path = Path::new(&base_path);
        let local_path = Path::new(passed_input_path.as_ref());
        root_path.join(local_path)
    };

    let output_path ;
    let mut delete_original = false;
    match lua.get_type(2) {
        "string" => {
            let passed_output_path = lua.check_string(2);
            output_path = Some({
                let root_path = Path::new(&base_path);
                let local_path = Path::new(passed_output_path.as_ref());
                root_path.join(local_path)
            });

            if lua.get_top() >= 3 && lua.get_type(3) == "boolean" {
                delete_original = lua.check_boolean(3);
            }
        },
        "boolean" => {
            delete_original = lua.check_boolean(2);
            output_path = None
        },
        _ => output_path = None
    };

    if let Err(e) = unzip_archive(&input_path, output_path, delete_original) {
        lua.error(&format!("{}", e));
    }

    0
}

#[gmod13_open]
unsafe fn gmod13_open(lua: gmod::lua::State) -> i32 {
    lua.new_table();
    lua.push_function(zip);
    lua.set_global(lua_string!("Zip"));
    lua.push_function(unzip);
    lua.set_global(lua_string!("Unzip"));

    0
}

#[gmod13_close]
unsafe fn gmod13_close(_: gmod::lua::State) -> i32 {
    0
}