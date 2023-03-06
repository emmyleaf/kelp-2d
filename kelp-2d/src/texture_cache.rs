use crate::{KelpError, KelpTextureId};
use ahash::AHashMap;
use std::rc::Rc;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Device, Sampler, Texture,
    TextureViewDescriptor,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TextureBindGroupId {
    texture_id: KelpTextureId,
    smooth: bool,
}

#[derive(Debug)]
pub(crate) struct TextureCache {
    texture_cache: AHashMap<KelpTextureId, Texture>,
    bind_group_cache: AHashMap<TextureBindGroupId, BindGroup>,
    texture_bind_layout: Rc<BindGroupLayout>,
    linear_sampler: Sampler,
    point_sampler: Sampler,
}

impl TextureCache {
    pub fn new(texture_bind_layout: Rc<BindGroupLayout>, linear_sampler: Sampler, point_sampler: Sampler) -> Self {
        Self {
            texture_cache: AHashMap::new(),
            bind_group_cache: AHashMap::new(),
            texture_bind_layout,
            linear_sampler,
            point_sampler,
        }
    }

    pub fn get_texture(&self, id: KelpTextureId) -> Result<&Texture, KelpError> {
        self.texture_cache.get(&id).ok_or(KelpError::InvalidTextureId)
    }

    pub fn insert_texture(&mut self, texture: Texture) -> KelpTextureId {
        let id = KelpTextureId(texture.global_id());
        self.texture_cache.insert(id, texture);
        id
    }

    pub fn get_bind_group(&self, id: &TextureBindGroupId) -> Result<&BindGroup, KelpError> {
        self.bind_group_cache.get(id).ok_or(KelpError::InvalidBindGroupId)
    }

    #[allow(clippy::map_entry)]
    pub fn get_valid_bind_group_id(
        &mut self,
        device: &Device,
        texture_id: KelpTextureId,
        smooth: bool,
    ) -> Result<TextureBindGroupId, KelpError> {
        let id = TextureBindGroupId { texture_id, smooth };
        if !self.bind_group_cache.contains_key(&id) {
            let bind_group = self.create_texture_bind_group(device, texture_id, smooth)?;
            self.bind_group_cache.insert(id, bind_group);
        }
        Ok(id)
    }

    /* private */
    fn create_texture_bind_group(
        &mut self,
        device: &Device,
        texture_id: KelpTextureId,
        smooth: bool,
    ) -> Result<BindGroup, KelpError> {
        let texture = self.get_texture(texture_id)?;
        let sampler = if smooth {
            &self.linear_sampler
        } else {
            &self.point_sampler
        };
        Ok(device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: self.texture_bind_layout.as_ref(),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture.create_view(&TextureViewDescriptor::default())),
                },
                BindGroupEntry { binding: 1, resource: BindingResource::Sampler(sampler) },
            ],
        }))
    }
}
