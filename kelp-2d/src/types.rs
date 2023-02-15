use std::ops::Range;

#[derive(Debug)]
pub struct KelpTexture {
    pub texture: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
}

#[derive(Debug)]
#[repr(C)]
pub struct Transform {
    pub render_x: f32,
    pub render_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub origin_x: f32,
    pub origin_y: f32,
}

#[derive(Debug)]
#[repr(C)]
pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub angle: f32,
    pub scale: f32,
}

#[derive(Debug)]
#[repr(C)]
pub struct InstanceData {
    pub color: [f32; 4],
    pub source: Transform,
    pub world: Transform,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InstanceGPU {
    pub color: [f32; 4],
    pub source: glam::Mat4,
    pub world: glam::Mat4,
}

#[derive(Debug)]
pub struct InstanceGroup<'a> {
    pub bind_group: &'a wgpu::BindGroup,
    pub range: Range<u32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            render_x: 0.0,
            render_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            origin_x: 0.0,
            origin_y: 0.0,
        }
    }
}

impl Camera {
    pub fn new(x: f32, y: f32, width: f32, height: f32, angle: f32, scale: f32) -> Self {
        Self { x, y, width, height, angle, scale }
    }
}

impl From<&Transform> for glam::Mat4 {
    fn from(transform: &Transform) -> Self {
        let (sin, cos) = transform.rotation.sin_cos();
        let a = cos * transform.scale_x;
        let b = -sin * transform.scale_y;
        let c = sin * transform.scale_x;
        let d = cos * transform.scale_y;
        let x = transform.render_x + transform.origin_x - (a * transform.origin_x) - (b * transform.origin_y);
        let y = transform.render_y + transform.origin_y - (c * transform.origin_x) - (d * transform.origin_y);
        Self::from_cols(
            glam::vec4(a, c, 0.0, 0.0),
            glam::vec4(b, d, 0.0, 0.0),
            glam::vec4(0.0, 0.0, 1.0, 0.0),
            glam::vec4(x, y, 0.0, 1.0),
        )
    }
}

impl From<&Camera> for glam::Mat4 {
    fn from(camera: &Camera) -> Self {
        let (sin, cos) = camera.angle.sin_cos();
        let cs = cos * camera.scale;
        let ss = sin * camera.scale;
        let x = 0.5 * camera.width - (cs * camera.x) + (ss * camera.y);
        let y = 0.5 * camera.height - (ss * camera.x) - (cs * camera.y);
        let view = Self::from_cols(
            glam::vec4(cs, ss, 0.0, 0.0),
            glam::vec4(-ss, cs, 0.0, 0.0),
            glam::vec4(0.0, 0.0, 1.0, 0.0),
            glam::vec4(x, y, 0.0, 1.0),
        );
        let projection = Self::orthographic_rh(0.0, camera.width, camera.height, 0.0, 0.0, 1.0);
        projection * view
    }
}

impl From<&InstanceData> for InstanceGPU {
    fn from(data: &InstanceData) -> Self {
        Self {
            color: data.color,
            source: (&data.source).into(),
            world: (&data.world).into(),
        }
    }
}

unsafe impl bytemuck::Zeroable for InstanceGPU {}
unsafe impl bytemuck::Pod for InstanceGPU {}
