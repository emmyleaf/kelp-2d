use crate::{FrameState, Kelp, Window};

static mut KELP: Option<Kelp> = None;
static mut FRAME: Option<FrameState> = None;

#[no_mangle]
pub unsafe extern "C" fn initialise(window: Window, width: u32, height: u32) {
    KELP = Some(pollster::block_on(Kelp::new(&window, width, height)));
}

#[no_mangle]
pub unsafe extern "C" fn begin_frame() {
    let kelp = KELP.as_mut().expect("Cannot call begin_frame before initialise.");
    FRAME = Some(kelp.begin_frame());
}

// TODO: finish this function & the rest :)
#[no_mangle]
pub unsafe extern "C" fn begin_render_pass() {
    let kelp = KELP.as_ref().expect("Cannot call begin_render_pass before initialise.");
    let frame = FRAME.as_mut().expect("Cannot call begin_render_pass before begin_frame.");
    let mut render_pass = frame.begin_render_pass();
    render_pass.set_pipeline(&kelp.pipeline);
    render_pass.set_vertex_buffer(0, kelp.vertex_buffer.slice(..));
    render_pass.set_bind_group(0, &kelp.vertex_group.bind, &[]);
}
