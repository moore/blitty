use core::mem;
use std::cmp::min;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
} 

#[derive(Clone, Copy, PartialEq, Eq)]
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
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CommandType {
    Null,
    Rect(Rgb),
}



#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
    opacity: u8,
}

pub struct DisplayList<const LENGTH: usize> {
    epoch: u8,
    current: [Command; LENGTH],
    new: [Command; LENGTH],
}

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

                for i in 0..LENGTH {
                    let current = &mut self.current[i];
                    let new = &mut self.new[i];

                    // Check for occlusions.
                    // Dose this layer cover the tile?
                    // If so we can start drawing from it
                    // instead of the bottom. (Its possible
                    // some set of tiles above 0 will cover
                    // but we don't take advantage of that.)
                    if (current.bounds.x1 <= x1) 
                    && (current.bounds.x2 >= x2)
                    && (current.bounds.y1 <= y2)
                    && (current.bounds.y2 >= y2) {
                        bottom = i;
                    }

                    if (new.bounds.x1 <= x1) 
                    && (new.bounds.x2 >= x2)
                    && (new.bounds.y1 <= y2)
                    && (new.bounds.y2 >= y2) {
                        bottom = i;
                    }

                    // Is there change in this tile.
                    if current.epoch != new.epoch {

                        if (current.bounds.x1 <= x2) 
                        && (current.bounds.x2 <= x1)
                        && (current.bounds.y1 <= y2)
                        && (current.bounds.y2 <= y2) {
                            has_change = true;
                        } else  if (new.bounds.x1 <= x2) 
                        && (new.bounds.x2 <= x1)
                        && (new.bounds.y1 <= y2)
                        && (new.bounds.y2 <= y2) {
                            has_change = true;
                        } 
                    } 
                }

                let bounds = BoundingBox {
                    x1,
                    y1,
                    x2,
                    y2,
                };

                if has_change {
                    for i in bottom..LENGTH {
                        let command = &self.new[i];
                        renderer.draw(command, &bounds)?;
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

pub enum RendererError {

}

pub trait Renderer {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn chunk_size(&self) -> usize;
    fn draw(&mut self, command: &Command, clip: &BoundingBox) -> Result<(), RendererError>;
    fn flush(&mut self) -> Result<(), RendererError>;
}

#[cfg(test)]
mod test;
