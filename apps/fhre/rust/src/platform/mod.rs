use crate::{BackendCapabilities, InputQueue, PixelFormat, RenderBackend, Surface};

pub trait FramebufferBackend: RenderBackend {
    fn width(&self) -> u16;
    fn height(&self) -> u16;
    fn pixel_format(&self) -> PixelFormat;
    fn flush(&mut self) {}

    fn framebuffer_capabilities(&self) -> BackendCapabilities {
        self.capabilities()
    }
}

pub trait InputSource<const N: usize> {
    fn poll(&mut self, queue: &mut InputQueue<N>, timestamp_us: u64) -> usize;
}

impl FramebufferBackend for Surface {
    fn width(&self) -> u16 {
        Surface::width(self)
    }

    fn height(&self) -> u16 {
        Surface::height(self)
    }

    fn pixel_format(&self) -> PixelFormat {
        self.format
    }
}
