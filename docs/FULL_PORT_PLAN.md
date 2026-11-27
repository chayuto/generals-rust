# Full port plan

Goal (owner, 2026-11-27): a full Rust port of the game. `PLAN.md` covers the
simulation up to a headless runner; this note covers the whole game.

Line counts are C++ in `Core` and `GeneralsMD/Code` at the pinned commit, headers
included. They are upper bounds: some of that code is for old platforms and will
not be carried over. No token or time estimate is given for stages 2 to 6;
none has been measured.

## Stages

| Stage | What | C++ lines | Done when |
|---|---|---|---|
| 1. Simulation | The packets in `PORT_MAP.md`: weapons doing damage, movement and pathfinding, unit AI (`AI_PORT_PLAN.md`), the rest, map loading, a game-wide CRC | 191k unported; some of the 138k marked implemented is partial | A Rust headless run of a replay matches the C++ CRC at every checkpoint |
| 2. Assets and platform | Readers for the retail model, texture, map, text, sound and video files; window and input | not separated out; includes 30k `WWMath`, 40k `WWLib`, partly covered | Every retail asset the game opens loads in Rust |
| 3. Renderer | `WW3D2` (129k) and the `W3DDevice` game client (86k) | 215k | A map with units draws and matches reference screenshots |
| 4. Interface | `GUI` (81k), other game client (49k), input to commands (14k) | 144k | A skirmish can be set up, played and finished |
| 5. Audio and video | `WWAudio` (12k) and one audio device layer | about 15k | Sound, music and cutscenes play |
| 6. Network | `GameNetwork` (50k), of which GameSpy and online services 21k | 50k | Two Rust builds play a match in lockstep |

Not carried over: editors and tools (289k), the debug and profiling libraries,
and the audio and window backends that duplicate the one chosen.

## Rules that differ by stage

- The simulation stays bit-exact and free of external crates.
- The client has to behave like the original, not match it bit for bit. The
  renderer is a rewrite against a modern graphics interface, not a line by line
  translation of a DirectX 8 design.
- The C++ build is the reference that Rust output is compared with. It is not part
  of the finished game, and is retired when stage 6 is done.

## Assumed until the owner says otherwise

- Client crates (window, graphics, audio, video) are planned for, each checked
  for bundled non-Rust code. None has been added, and the owner is asked before
  the first one is (`OPEN_DECISIONS.md` item 12).
- Online services whose servers no longer exist come last, after LAN play.

## Order

Stage 1 first: everything later shows its results. The asset readers of stage 2
can run beside it, since they touch separate code.
