use crate::{Kelp, Window};

static mut KELP: Option<Kelp> = None;

#[no_mangle]
pub unsafe extern "C" fn initialise_kelp(window: Window, width: u32, height: u32) {
    KELP = Some(pollster::block_on(Kelp::new(&window, width, height)));
}
