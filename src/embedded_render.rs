
use embedded_graphics::{
    pixelcolor::{PixelColor, RgbColor},
    prelude::*,
    primitives::{Rectangle, PrimitiveStyleBuilder},

};

use super::*;


pub struct EmbeddedRender<D: DrawTarget<Color = C>, C: PixelColor> {
    width: usize,
    height: usize,
    chuck_size: usize,
    display: D,
}

impl<D: DrawTarget<Color = C>, C: PixelColor> EmbeddedRender<D, C> {
    pub fn new(display: D, chuck_size: usize) -> Self {
        let bounds = display.bounding_box();
        EmbeddedRender {
            width: bounds.size.width as usize, //BUG: why is this safe?
            height: bounds.size.height as usize, //BUG: why is this safe?
            chuck_size,
            display,
        }
    }
}

impl<D: DrawTarget<Color = C>, C: RgbColor> Renderer for EmbeddedRender<D, C> {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn chunk_size(&self) -> usize {
        self.chuck_size
    }

    fn draw(&mut self, command: &Command, clip: &BoundingBox) -> Result<(), RendererError> {
        use CommandType::*;

        match command.flavor {
            Null => Ok(()),
            Rect(color) => {
                let line_style = PrimitiveStyleBuilder::new()
                    .stroke_color(C::RED)
                    .stroke_width(3)
                    .fill_color(C::GREEN)
                    .build();

                Rectangle::new(Point::new(79, 15), Size::new(34, 34))
                .into_styled(line_style)
                .draw(&mut self.display)
                .map_err(|e| RendererError::BackingError)?;



                Ok(())
            }
        }
    }
    fn flush(&mut self) -> Result<(), RendererError> {
        // This is a noop in this implementation
        Ok(())
    }
}