use crate::{KelpError, KelpMap, KelpTextureId};
use std::rc::Rc;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Device, Sampler, Texture,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TextureBindGroupId {
    texture_id: KelpTextureId,
    smooth: bool,
}

#[derive(Debug)]
pub(crate) struct TextureCache {
    texture_cache: KelpMap<KelpTextureId, Texture>,
    bind_group_cache: KelpMap<TextureBindGroupId, BindGroup>,
    texture_bind_layout: Rc<BindGroupLayout>,
    linear_sampler: Sampler,
    point_sampler: Sampler,
}

impl TextureCache {
    pub fn new(texture_bind_layout: Rc<BindGroupLayout>, linear_sampler: Sampler, point_sampler: Sampler) -> Self {
        Self {
            texture_cache: Default::default(),
            bind_group_cache: Default::default(),
            texture_bind_layout,
            linear_sampler,
            point_sampler,
        }
    }

    pub fn insert_texture(&mut self, texture: Texture) -> KelpTextureId {
        let id = KelpTextureId(texture.global_id());
        self.texture_cache.insert(id, texture);
        id
    }

    pub fn ensure_bind_group(
        &mut self,
        device: &Device,
        texture_id: KelpTextureId,
        smooth: bool,
    ) -> Result<(), KelpError> {
        let id = TextureBindGroupId { texture_id, smooth };
        if !self.bind_group_cache.contains_key(&id) {
            let bind_group = self.create_texture_bind_group(device, texture_id, smooth)?;
            self.bind_group_cache.insert(id, bind_group);
        }
        Ok(())
    }

    pub fn get_bind_group(&self, texture_id: KelpTextureId, smooth: bool) -> Result<&BindGroup, KelpError> {
        let id = TextureBindGroupId { texture_id, smooth };
        self.bind_group_cache.get(&id).ok_or(KelpError::InvalidBindGroupId)
    }

    /* private */
    fn create_texture_bind_group(
        &self,
        device: &Device,
        texture_id: KelpTextureId,
        smooth: bool,
    ) -> Result<BindGroup, KelpError> {
        let texture = self.texture_cache.get(&texture_id).ok_or(KelpError::InvalidTextureId)?;
        let texture_view = texture.create_view(&Default::default());
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
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry { binding: 1, resource: BindingResource::Sampler(sampler) },
            ],
        }))
    }
}
