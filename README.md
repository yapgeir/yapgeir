# What is yapgeir?

Yapgeir is a game engine in Rust in a lot of ways inspired by [Bevy](https://github.com/bevyengine/bevy).

The goals of this project are:

- Single-threaded system runner. This greatly simplifies the system scheduler itself and relaxes restrictions on components and resources having to be `Sync` and/or `Send`. Multi-threading is still achievable explicitly, by having a system that launches a thread and keeps a future as a component or resource.
- It should be straightforward to port it to consoles. Currently, Sony Playstation Vita with a homebrew toolchain is supported. This is achieved by modular design and a custom graphics hardware abstraction layer designed around GLES2 capabilities.
- 2D games are first-class citizens.
- Turn-based games are first-class citizens.


## State of the project

This project is under active development right now, so if you have accidentally stumbled upon it, keep in mind that as of today

- A lot of things are experimental. No APIs are guaranteed to be backward compatible.
- There are no crates released. This engine is currently developed alongside a game where it is a submodule. The engine itself is used as a relative dependency.
- Documentation is almost non-existent
- There are no examples of how to use it


## License

Yapgeir is free, open source, and permissively licensed!
Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!

### Your contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.