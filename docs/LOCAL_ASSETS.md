# Local Retail Assets (`env:local-full` only)

Retail Zero Hour data is **never committed** (AGENTS.md §3.2). It lives outside the
repo on the host Mac and is read in place.

| Item | Location |
|---|---|
| Retail Zero Hour install (Steam app `2732960`, Windows depots) | `~/Games/generalszh` |
| Env var / CLI flag | `CNC_GENERALS_ZH_PATH`, `-assetPath ~/Games/generalszh` |
| C++ reference clone (pinned `84a7a20`) | `vendor/GeneralsX` (gitignored) |
| Reference build tree | `vendor/GeneralsX/build/macos-vulkan` |

Contents (~2.8 GB): 20 `.big` archives at the top level — `INIZH`, `MapsZH`,
`EnglishZH`, `AudioZH`, `AudioEnglishZH`, `SpeechZH`, `SpeechEnglishZH`, `MusicZH`,
`Music`, `TerrainZH`, `TexturesZH`, `W3DZH`, `W3DEnglishZH`, `WindowZH`, `ShadersZH`,
`gensecZH`, `PatchZH`, `PatchINI`, `PatchData`, `PatchWindow` — plus `Generals.exe`,
`Data/` and the Windows runtime DLLs. The `.exe`/`.dll` files are not used.

## Check it is present

```bash
ls ~/Games/generalszh/*.big | wc -l     # expect 20
du -sh ~/Games/generalszh               # expect ~2.8G
```

## (Re)download

The Steam account must own *Command & Conquer Generals Zero Hour*. The Mac Steam app
and `steamcmd` (`brew install --cask steamcmd`) share cached credentials, so no
password is needed if the Steam app is logged in. Otherwise run once, interactively
(password + Steam Guard): `steamcmd +login <steam_user> +quit`.

```bash
steamcmd +@sSteamCmdForcePlatformType windows \
  +force_install_dir ~/Games/generalszh \
  +login <steam_user> +app_update 2732960 validate +quit
```

- `+@sSteamCmdForcePlatformType windows` is required: the depots are Windows-only and
  steamcmd on macOS skips them otherwise.
- `+force_install_dir` must come **before** `+login`.
- App IDs are easy to mix up: `2229870` is Generals (base game), `2229850` is Red Alert 2.
  Zero Hour is `2732960`. Confirm with `+app_info_print <id>` if unsure.
- Re-running with `validate` is safe and resumes partial downloads.

## Rules for agents

- Pass `-assetPath ~/Games/generalszh` and run headless (`-headless`,
  `SDL_VIDEODRIVER=dummy`, `SDL_AUDIODRIVER=dummy`); never open a game window.
- Never copy, stage or commit anything from this directory. Golden vectors derived from
  it are fine; the source archives are not.
- `env:ephemeral-web` agents have no access to it. Anything that needs it must be
  committed first as a packet or golden vector (AGENTS.md §4.2).
