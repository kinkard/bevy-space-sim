# Project description

A small space simulator game, powered by [bevy](https://github.com/bevyengine/bevy), with a physics-based control (no absolute velocity).

Project goals:

- Practice with Rust language.
- Do a small hobby project.
- Test how to perform docking/navigation/interception/etc. in a real space.
- Create a platform for various AI algorithms tests.

## Build & Rud

```sh
cargo run --release
```

## License

All code in this project is dual-licensed under either:

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) ([`LICENSE-APACHE`](LICENSE-APACHE))
- [MIT license](https://opensource.org/licenses/MIT) ([`LICENSE-MIT`](LICENSE-MIT))

at your option.
This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.

The [assets](assets) included in this repository are taken from the public sources and typically fall under different open licenses.

- [Praetor drone model](assets/models/praetor.glb) from [EvE-3D-Printing](https://github.com/Kyle-Cerniglia/EvE-3D-Printing)
- [Infiltrator drone model](assets/models/infiltrator.glb) from [EvE-3D-Printing](https://github.com/Kyle-Cerniglia/EvE-3D-Printing)
- [Spaceship](assets/models/spaceship_v1.glb) from [Free3D](https://free3d.com/3d-model/intergalactic-spaceship-in-blender-28-eevee-394046.html)


## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as below, without any additional terms or conditions.
