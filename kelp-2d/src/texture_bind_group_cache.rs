use crate::KelpTexture;
use rustc_hash::FxHashMap;
use std::rc::Rc;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Device, Id, Sampler,
    TextureViewDescriptor,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct TextureBindingId {
    pub texture_id: Id,
    pub smooth: bool,
}

#[derive(Debug)]
pub(crate) struct TextureBindGroupCache {
    cache: FxHashMap<TextureBindingId, Rc<BindGroup>>,
    texture_bind_layout: BindGroupLayout,
    linear_sampler: Sampler,
    point_sampler: Sampler,
}

impl TextureBindGroupCache {
    pub fn new(texture_bind_layout: BindGroupLayout, linear_sampler: Sampler, point_sampler: Sampler) -> Self {
        Self {
            cache: Default::default(),
            texture_bind_layout,
            linear_sampler,
            point_sampler,
        }
    }

    pub fn get_texture_bind_group(&mut self, device: &Device, texture: &KelpTexture, smooth: bool) -> Rc<BindGroup> {
        let id = Self::to_binding_id(texture, smooth);
        let contains = self.cache.contains_key(&id);
        if !contains {
            let bind_group = self.create_texture_bind_group(device, texture, smooth);
            self.cache.insert(id, Rc::new(bind_group));
        }
        self.cache.get(&id).expect("Failed to get or create texture bind group.").clone()
    }

    pub fn remove_texture_bind_group(&mut self, texture: &KelpTexture, smooth: bool) {
        _ = self.cache.remove(&Self::to_binding_id(texture, smooth))
    }

    /* private */
    fn create_texture_bind_group(&mut self, device: &Device, texture: &KelpTexture, smooth: bool) -> BindGroup {
        let sampler = if smooth {
            &self.linear_sampler
        } else {
            &self.point_sampler
        };
        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.texture_bind_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &texture.wgpu_texture.create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry { binding: 1, resource: BindingResource::Sampler(sampler) },
            ],
        })
    }

    #[inline]
    fn to_binding_id(texture: &KelpTexture, smooth: bool) -> TextureBindingId {
        TextureBindingId { texture_id: texture.wgpu_texture.global_id(), smooth }
    }
}
