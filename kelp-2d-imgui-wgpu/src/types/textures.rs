use ahash::AHashMap;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct FontTexture {
    /// Texture identifier
    pub tex_id: Option<TextureId>,
    /// Texture width (in pixels)
    pub width: u32,
    /// Texture height (in pixels)
    pub height: u32,
    /// Raw texture data (in bytes).
    ///
    /// The format depends on which function was called to obtain this data.
    pub data: *const u8,
    /// The length of the data array
    pub data_length: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct TextureId(usize);

// TODO: indexmap here?
#[derive(Debug, Default)]
pub struct Textures<T> {
    textures: AHashMap<usize, T>,
    next: usize,
}

impl TextureId {
    /// Creates a new texture id with the given identifier.
    #[inline]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    /// Returns the id of the TextureId.
    #[inline]
    pub const fn id(self) -> usize {
        self.0
    }
}

impl From<usize> for TextureId {
    #[inline]
    fn from(id: usize) -> Self {
        TextureId(id)
    }
}

impl<T> Textures<T> {
    pub fn new() -> Self {
        Textures { textures: Default::default(), next: 0 }
    }

    pub fn insert(&mut self, texture: T) -> TextureId {
        let id = self.next;
        self.textures.insert(id, texture);
        self.next += 1;
        TextureId::from(id)
    }

    pub fn replace(&mut self, id: TextureId, texture: T) -> Option<T> {
        self.textures.insert(id.0, texture)
    }

    pub fn remove(&mut self, id: TextureId) -> Option<T> {
        self.textures.remove(&id.0)
    }

    pub fn get(&self, id: TextureId) -> Option<&T> {
        self.textures.get(&id.0)
    }

    pub fn get_mut(&mut self, id: TextureId) -> Option<&mut T> {
        self.textures.get_mut(&id.0)
    }
}
