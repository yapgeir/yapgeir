# What is yapgeir?

Yapgeir is a game engine in Rust in a lot of ways inspired by [Bevy](https://github.com/bevyengine/bevy).

The goals of this project are:

- Single-threaded system runner. This greatly simplifies the system scheduler itself and relaxes restrictions on components and resources having to be `Sync` and/or `Send`. Multi-threading is still achievable explicitly, by having a system that launches a thread and keeps a future as a component or resource.
- It should be straightforward to port it to consoles. Currently, Sony Playstation Vita with a homebrew toolchain is supported. This is achieved by modular design and a custom graphics hardware abstraction layer designed around GLES2 capabilities.
- 2D games are first-class citizens.
- Turn-based games are first-class citizens.

## Crates

![Crate graph](/docs/dependencies.png)


## State of the project

This project is under active development right now, so if you have accidentally stumbled upon it, keep in mind that as of today

- A lot of things are experimental. No APIs are guaranteed to be backward compatible.
- There are no crates released. This engine is currently developed alongside a game where it is a submodule. The engine itself is used as a relative dependency.
- Documentation is almost non-existent

## Building for wasm

The engine currently supports wasm via `emscripten`. To compile your project, you need

- `emscripten` toolchain [installed](https://emscripten.org/docs/getting_started/downloads.html) on your system, and `emcc` available in your `$PATH`
- Add Rust target - `rustup target add wasm32-unknown-emscripten`
- Add `.cargo/config.toml` with the following contents:
    ```toml
    [target.wasm32-unknown-emscripten]
    rustflags = [
        "-C", "link-arg=-s", "-C", "link-arg=USE_SDL=2",
        "-C", "link-arg=-s", "-C", "link-arg=MAX_WEBGL_VERSION=2",
        "-C", "link-arg=-s", "-C", "link-arg=MIN_WEBGL_VERSION=2",
        # Optionally set these if you need huge heaps/stack
        "-C", "link-arg=-s", "-C", "link-arg=MAXIMUM_MEMORY=4gb",
        "-C", "link-arg=-s", "-C", "link-arg=ALLOW_MEMORY_GROWTH",
        "-C", "link-arg=-s", "-C", "link-arg=TOTAL_STACK=16mb",
        "-C", "link-arg=-s", "-C", "link-arg=INITIAL_MEMORY=64mb",
    ]

    [env]
    # In order to load your assets, you must preload them. This line assumes all of your assets are in a `assets` folder and is read relatively to the binary
    EMCC_CFLAGS = "--preload-file=assets"
    ```
- To run your project you will need to put an `index.html` file in the same place as your build artifacts:
    ```html
    <!DOCTYPE html>
    <html>
    <body>
        <canvas data-raw-handle="1" id="canvas"></canvas>
        <script type="text/javascript">
        var Module = { canvas: document.getElementById("canvas") };
        </script>
        <!-- Rename src to your actual build artifact -->
        <script src="game.js"></script>
    </body>
    </html>
    ```
- Run `cargo build --target=wasm32-unknown-emscripten --release` to build your project

## Examples

* [2d_sprite](examples/2d_sprite.rs)


## License

Yapgeir is free, open source, and permissively licensed!
Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!

### Your contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
