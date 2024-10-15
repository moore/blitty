
use embedded_graphics::{
    pixelcolor::{PixelColor, Rgb888},
    prelude::*,
    primitives::{Rectangle, PrimitiveStyleBuilder},

};

use std::cmp::{min, max};

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

    pub fn get_display(&self) -> &D {
        &self.display
    }
}

impl<D: DrawTarget<Color = Rgb888>> Renderer for EmbeddedRender<D, Rgb888> {
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
            Rect(rgb) => {


                let x1 = max(command.bounds.x1, clip.x1);
                let y1 = max(command.bounds.y1, clip.y1);
                let x2 = min(command.bounds.x2, clip.x2);
                let y2 = min(command.bounds.y2, clip.y2);

                let width = (x2 - x1) as u32; // BUG is this safe
                let height = (y2 - y1) as u32; // Bug is this safe
                let x = x1 as i32; // Bug is this safe
                let y = y1 as i32; // Bug is this safe

                let color = Rgb888::new(rgb.r, rgb.b, rgb.g);

                let line_style = PrimitiveStyleBuilder::new()
                    .fill_color(color)
                    .build();

                Rectangle::new(Point::new(x, y), Size::new(width, height))
                .into_styled(line_style)
                .draw(&mut self.display)
                .map_err(|e| RendererError::BackingError)?;
                Ok(())
            }
        }
    }

    fn clear(&mut self, clip: &BoundingBox) -> Result<(), RendererError> {
        let top_left = Point::new(clip.x1 as i32, clip.y1 as i32);
        let size = Size::new((clip.x2 - clip.x1) as u32, (clip.y2 - clip.y1) as u32);
        let area = Rectangle::new(top_left, size);
        let mut clipped = self.display.clipped(&area);
        clipped.clear(Rgb888::BLACK)
        .map_err(|e| RendererError::BackingError)?;

        Ok(())
    }

    fn flush(&mut self) -> Result<(), RendererError> {
        // This is a noop in this implementation
        Ok(())
    }
}