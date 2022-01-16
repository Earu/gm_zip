# gm_zip
Create and extract archives of files within Garry's Mod.

**Note: The scope of this module only works accross the gmod installation files e.g from GarrysMod/garrysmod/. Anything out of this scope is not supported.**

## Examples
Folder archiving:
```lua
require("zip")

local delete_original_folder = true
Zip("folder.zip", "path/to/folder", delete_original_folder)

Zip("folder.zip", "path/to/folder2") -- not providing the 3rd argument (or setting it to false) will keep the original folder
```

Simple file archiving:
```lua
require("zip")

Zip("archive.zip", { "lua/send.txt" })
```

Change the paths in the archive:
```lua
require("zip")

Zip("addon_send.zip", {
  { Path = "lua/send.txt", ArchivePath = "send.txt" } -- Path is the real path to the file, ArchivePath is the path used within the archive
})
```

Advanced file archiving:
```lua
require("zip")

local function get_lua_files(res, dir)
    res = res or {}
    dir = dir or "lua"

    local files, dirs = file.Find(dir .. "/*", "MOD")
    for _, f in pairs(files or {}) do
        if not f:EndsWith(".lua") then continue end
        table.insert(res, dir .. "/" .. f)
    end

    for _, d in pairs(dirs or {}) do
        get_lua_files(res, dir .. "/" .. d)
    end

    return res
end

local lua_files = get_lua_files()
Zip("my_lua_files.zip", lua_files)
```

Extracting an archive:
```lua
require("zip")

Unzip("archive.zip") -- this will extract the archive in the same directory
Unzip("archive.zip", "addons/extracted_archive") -- this will extract the archive in GarrysMod/garrysmod/addon/extracted_archive

Unzip("archive.zip", true) -- extracts and removes the original archive file
Unzip("archive.zip", "addons/extracted_archive", true) -- extracts to passed path and removes the original archive file
```

## Compiling
- Open a terminal
- Install **cargo** if you dont have it (on Windows => https://win.rustup.rs) (on Linux/Macos => curl https://sh.rustup.rs -sSf | sh)
- Get [git](https://git-scm.com/downloads) or download the archive for the repository directly
- `git clone https://github.com/Earu/gm_zip` (ignore this if you've downloaded the archive)
- Run `cd gm_zip`
- `cargo build`
- Go in `target/debug` and rename the binary according to your branch and realm (gmsv_zip_win64.dll, gmcl_zip_win64.dll, gmsv_zip_linux.dll, gmcl_zip_linux.dll, gmcl_zip_osx64.dll)
- Put the binary in your gmod `lua/bin` directory

*Note: Even on other platforms than Windows the extension of your modules **needs** to be **.dll***
