# gm_zip
Create archives of files within Garry's Mod.

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

## Compiling
- open a terminal
- get [git](https://git-scm.com/downloads) or download the archive for the repository directly
- `git clone https://github.com/Earu/gm_zip` (ignore this if you've downloaded the archive)
- run `cd gm_zip`
- install cargo (on windows => https://win.rustup.rs) (on linux/macos => curl https://sh.rustup.rs -sSf | sh)
- `cargo build`
- go in `target/debug` and rename the binary according to your branch and realm (gmsv_zip_win64, gmcl_zip_win64, gmsv_zip_linux, gmcl_zip_linux, gmcl_zip_osx64)
- put the binary in your gmod `lua/bin` directory
