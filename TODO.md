## v0.1.0

- [x] Investigate dynamic batching with something like `etagere` or `guillotiere`
  - This could allow us to batch all our draws into one or a few calls by allocating textures on an atlas
- [ ] New separate `RenderTarget` pipeline - keep it very simple
- [ ] fix imgui rendering! let's just use our version of the renderer with `imgui` dep
- [ ] Finish wgsl versions of shaders
- [ ] Move Lutra specific details (eg. transform -> matrix conversion) to ffi crate (and rename that to lutra-kelp???)
- [ ] Removing textures: intend for removal in batches at start of frame
- [ ] Custom fragment shaders
  - Shader parameters - describe the layout at creation time
  - What if we open up the remaining 64 bytes of push constants? that will cover most shaders used in tmfbma/dddb
  - Will still need a binding slot for texture/buffer parameters etc, so not a huge win
  - Write guide to writing custom shaders
- [ ] Benchmarks!
