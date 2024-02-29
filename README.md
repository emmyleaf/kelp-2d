# Kelp 🌿

`kelp-2d` is a dynamically batching sprite renderer.
Textures are allocated on an array of atlases, which avoids rebinding resources to the graphics pipeline.
This means that all drawing to a given target can be done in a single draw call.

Under the hood, Kelp uses:

- [wgpu](https://github.com/gfx-rs/wgpu), a safe cross-platform graphics api abstraction.
- [guillotière](https://github.com/nical/guillotiere), a dynamic texture atlas allocator.
- [interoptopus](https://github.com/ralfbiedert/interoptopus/), a C# bindings generator.

The intention is to use this as a basis for other game engine projects, such as future versions of [Lutra](https://github.com/emmyleaf/Lutra).

Bindings for C# can be found in the directory `/kelp-2d-cdylib/bindings`.

## License

`kelp-2d` is licensed under the [MIT License](LICENSE).
