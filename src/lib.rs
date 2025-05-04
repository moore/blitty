#![no_std]

use core::mem;
use core::cmp::min;
use core::result::{Result, Result::{Ok, Err}};
use core::convert::From;
use core::iter::Iterator;

use embedded_graphics::prelude::DrawTarget;


pub mod embedded_render;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BoundingBox {
    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
} 

impl BoundingBox {
    pub fn new( x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        Self { x1, y1, x2, y2 }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Command {
    epoch: u8,
    bounds: BoundingBox,
    flavor: CommandType,
} 

impl Command {
    const fn null() -> Self {
        Command {
            epoch: 0,
            bounds: BoundingBox {
                x1: 0,
                y1: 0,
                x2: 0,
                y2: 0
            }, 
            flavor: CommandType::Null
        }
    }

    pub fn new_rect( bounds: BoundingBox, rgb: Rgb ) -> Self {
        Command {
            epoch: 0,
            bounds,
            flavor: CommandType::Rect(rgb),
        }
    }

    // BUG: This will only work for shapes that fill their bounds,
    // so this is totally wrong and need specialization to
    // actually work.
    fn covers(&self, clip: &BoundingBox) -> Result<bool, RendererError> {
        let covers = (self.bounds.x1 <= clip.x1) 
        && (self.bounds.x2 >= clip.x2)
        && (self.bounds.y1 <= clip.y1)
        && (self.bounds.y2 >= clip.y2);

        Ok(covers)
     }

    // BUG: This should be specialized to look if there are really
    // any pixels to draw in the clip.So this only helps if narrow
    // cases right now.
    fn intersects(&self, clip: &BoundingBox) -> Result<bool, RendererError> {
        let intersects = (self.bounds.x1 <= clip.x2) 
        && (self.bounds.x2 >= clip.x1)
        && (self.bounds.y1 <= clip.y2)
        && (self.bounds.y2 >= clip.y1);

        Ok(intersects)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CommandType {
    Null,
    Rect(Rgb),
}



#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    pub fn new( r: u8, b: u8, g: u8 ) -> Self {
        Self {r, g, b}
    }
}

pub struct DisplayList<const LENGTH: usize> {
    epoch: u8,
    current: [Command; LENGTH],
    new: [Command; LENGTH],
}

#[derive(Debug, PartialEq)]
pub enum DisplayListError {
    IndexOutOfRange,
    UpdateFlavorMismatch(CommandType, CommandType),
    RenderError(RendererError),
}

impl From<RendererError> for DisplayListError {
    fn from(value: RendererError) -> Self {
        DisplayListError::RenderError(value)
    }
}

impl<const LENGTH: usize> DisplayList<LENGTH> {

    pub const LENGTH: usize = LENGTH;

    pub fn new() -> Self {
        DisplayList {
            epoch: 1,
            current: [Command::null(); LENGTH],
            new: [Command::null(); LENGTH],
        }
    }

    pub fn set(&mut self, index: usize, mut command: Command) -> Result<(), DisplayListError> {

        if index >= LENGTH {
            return Err(DisplayListError::IndexOutOfRange);
        }

        command.epoch = self.epoch;

        self.new[index] = command;

        Ok(())
    }

    pub fn get(&self, index: usize) -> Result<Command, DisplayListError> {
        if index >= LENGTH {
            return Err(DisplayListError::IndexOutOfRange);
        }

        Ok(self.new[index].clone())
    }

    pub fn update(&mut self, index: usize, mut command: Command) -> Result<(), DisplayListError> {

        if index >= LENGTH {
            return Err(DisplayListError::IndexOutOfRange);
        }

        let current_flavor = mem::discriminant(&self.current[index].flavor);
        let new_flavor = mem::discriminant(&command.flavor);

        if current_flavor != new_flavor {
            let old = self.current[index].flavor;
            let new = command.flavor;
            return Err(DisplayListError::UpdateFlavorMismatch(old, new))
        }

        command.epoch = self.epoch;

        self.new[index] = command;

        Ok(())
    }

    pub fn draw(&mut self, renderer: &mut impl Renderer) -> Result<(), DisplayListError> {

        let width = renderer.width();
        let height = renderer.height();
        let step = renderer.chunk_size();

        for x1 in (0..width).step_by(step) {
            let x2 = min(width, x1+step);
            for y1 in (0..height).step_by(step) {
                let y2 = min(height, y1+step);

                let mut bottom = 0;
                let mut has_change = false;

                let bounds = BoundingBox {
                    x1,
                    y1,
                    x2,
                    y2,
                };

                for i in 0..LENGTH {
                    let current = &mut self.current[i];
                    let new = &mut self.new[i];

                    // Check for occlusions.
                    // Dose this layer cover the tile?
                    // If so we can start drawing from it
                    // instead of the bottom. (Its possible
                    // some set of tiles above 0 will cover
                    // but we don't take advantage of that.)
                    if new.covers(&bounds)? {
                        bottom = i;
                        if current.epoch == new.epoch {
                            has_change = false;
                        }
                    }


                    // Is there change in this tile.
                    if current.epoch != new.epoch {

                        if current.intersects(&bounds)? {
                            has_change = true
                        } else if new.intersects(&bounds)? {
                            has_change = true
                        } 
                    }
                }

                if has_change {

                    renderer.clear(&bounds)?;

                    for i in bottom..LENGTH {
                        let command = &self.new[i];
                        let old = &self.current[i];
                        if command.intersects(&bounds)? {
                            renderer.draw(command, &bounds)?;
                        } else if old.intersects(&bounds)? {
                            renderer.draw(old, &bounds)?;
                        }
                    }
                    renderer.flush()?;
                }
            }
        }

        // update the state
        for i in 0..LENGTH {
            let current = &mut  self.current[i];
            let new = &mut self.new[i];

            if new.epoch != current.epoch {
                new.epoch = self.epoch;
                *current = *new; 
            } else {
                new.epoch = self.epoch;
                current.epoch = self.epoch;
            }
        }

        // Should this happen here or at the top?
        // What happens if we error our above should
        // the epoch have changed? Probably not as
        // that would mean with enough errors it would
        // wrap which could be really confusing.
        self.epoch = self.epoch.wrapping_add(1);

        Ok(())
    }

}

#[derive(Debug, PartialEq)]
pub enum RendererError {
    BackingError
}

pub trait Renderer {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn chunk_size(&self) -> usize;
    fn clear(&mut self, clip: &BoundingBox) -> Result<(), RendererError>;
    fn draw(&mut self, command: &Command, clip: &BoundingBox) -> Result<(), RendererError>;
    fn flush(&mut self) -> Result<(), RendererError>;
}

#[cfg(test)]
mod test;
