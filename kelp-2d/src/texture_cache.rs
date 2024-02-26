use crate::{KelpError, KelpMap, KelpTargetId, KelpTextureId};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct TextureAllocation {
    pub(crate) id: KelpTextureId,
    pub(crate) rectangle: guillotiere::Rectangle,
}

// TODO: move the samplers outta here now?
pub(crate) struct TextureCache {
    allocators: Vec<guillotiere::AtlasAllocator>,
    texture_cache: KelpMap<KelpTextureId, TextureAllocation>,
    target_cache: KelpMap<KelpTargetId, wgpu::Texture>,
    point_sampler: wgpu::Sampler,
    linear_sampler: wgpu::Sampler,
}

impl TextureCache {
    pub fn new(texture_array: &wgpu::Texture, point_sampler: wgpu::Sampler, linear_sampler: wgpu::Sampler) -> Self {
        let alloc_size = guillotiere::Size::new(texture_array.width() as i32, texture_array.height() as i32);
        let layers = texture_array.depth_or_array_layers() as usize;
        Self {
            allocators: vec![guillotiere::AtlasAllocator::new(alloc_size); layers],
            texture_cache: Default::default(),
            target_cache: Default::default(),
            point_sampler,
            linear_sampler,
        }
    }

    pub fn new_texture_alloc(&mut self, width: u32, height: u32) -> KelpTextureId {
        // TODO: handle extending the array! and error for failing to do so
        let allocation = self.allocate_texture(width as i32, height as i32).unwrap();
        let id = allocation.id;
        self.texture_cache.insert(id, allocation);
        id
    }

    pub fn insert_target(&mut self, texture: wgpu::Texture) -> KelpTargetId {
        let id = KelpTargetId(texture.global_id());
        self.target_cache.insert(id, texture);
        id
    }

    pub fn get_texture(&self, texture_id: KelpTextureId) -> Result<TextureAllocation, KelpError> {
        self.texture_cache.get(&texture_id).map(Clone::clone).ok_or(KelpError::InvalidTextureId)
    }

    pub fn get_target(&self, target_id: KelpTargetId) -> Result<&wgpu::Texture, KelpError> {
        self.target_cache.get(&target_id).ok_or(KelpError::InvalidTextureId)
    }

    /* private */
    fn allocate_texture(&mut self, width: i32, height: i32) -> Option<TextureAllocation> {
        let size_with_padding = guillotiere::Size::new(width + 1, height + 1);
        let mut allocation: Option<guillotiere::Allocation> = None;
        let mut layer = 0;
        for (i, allocator) in self.allocators.iter_mut().enumerate() {
            allocation = allocator.allocate(size_with_padding);
            if allocation.is_some() {
                layer = i as u32;
                break;
            }
        }

        if let Some(guillotiere::Allocation { id, mut rectangle }) = allocation {
            rectangle.max.x -= 1;
            rectangle.max.y -= 1;
            Some(TextureAllocation { id: KelpTextureId { layer, alloc_id: id }, rectangle })
        } else {
            None
        }
    }
}
