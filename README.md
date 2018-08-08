frangiclave-mod-manager
=================

*A desktop utility for patching Cultist Simulator and applying mods.*

**frangiclave-mod-manager** is a Rust application for applying, creating and distributing mods for the game Cultist Simulator.
It depends on [frangiclave-patch](https://gitlab.com/frangiclave/frangiclave-patch) for the mods it applies.

License: ![CC0](https://licensebuttons.net/p/zero/1.0/88x15.png "CC0")

## Building

In order to build the application, you will need to have installed [Rust](https://www.rust-lang.org/) on your system, and then to follow these steps:

1. Build a release version of [frangiclave-patch](https://gitlab.com/frangiclave/frangiclave-patch).
2. Copy its output files (`Assembly-CSharp.FrangiclavePatch.mm.dll`, `Mono.Cecil.dll`, `Mono.Cecil.Mdb.dll`, `Mono.Cecil.Pdb.dll`, `MonoMod.exe`, `MonoMod.Utils.dll`) to the `data/patch/` directory.
3. Run `cargo build --release`.

You should now have a build of `frangiclave-mod-manager` in your `target/release/` folder.