

use embedded_graphics::{draw_target::Clipped, framebuffer::buffer_size};
use oled_async::{display, mode::displaymode::DisplayModeTrait, prelude::*, properties::DisplayProperties};
use display_interface::{AsyncWriteOnlyDataCommand, DisplayError, DataFormat};

use core::cmp::{min, max};

use super::*;

mod commands;
use commands as i2c;


impl From<DisplayError> for RendererError {
    fn from(_value: DisplayError) -> Self {
        return RendererError::BackingError
    }
}

/// `BUFFER_SIZE` must be a perfect square and is the number of bytes
/// used for buffering a chunk when rendering.
pub struct Sh1107Render<'a, DI, const BUFFER_SIZE: usize> 
where
    DI: AsyncWriteOnlyDataCommand,
{
    width: u32,
    height: u32,
    chunk_width: u32,
    chunk_height: u32,
    clip: BoundingBox,
    buffer: [u8; BUFFER_SIZE],
    display: &'a mut DI,
}

impl<'a, DI, const BUFFER_SIZE: usize> Sh1107Render<'a, DI, BUFFER_SIZE>  
where
    DI: AsyncWriteOnlyDataCommand,
{
    const CHUNK_SIZE: usize = usize::isqrt(BUFFER_SIZE) * 8;

    pub fn new(display: &'a mut DI, width: u32, height: u32, chunk_width: u32, chunk_height: u32) -> Self {
        Self {
            width,
            height,
            chunk_width,
            chunk_height,
            clip: BoundingBox::new(0, 0, chunk_width, chunk_height),
            buffer: [0u8;BUFFER_SIZE],
            display,
        }
    }

    /// Initialise the display in column mode (i.e. a byte walks down a column of 8 pixels) with
    /// column 0 on the left and column _(display_width - 1)_ on the right.
    pub async fn init(
        &mut self,
        dimensions: (u8, u8),
    ) -> Result<(), RendererError>
   
    {
        let iface = &mut self.display;
        //iface.init().await?;
        // TODO: Break up into nice bits so display modes can pick whathever they need
        let (_, display_height) = dimensions;

        i2c::Command::DisplayOn(false).send(*iface).await?;
        i2c::Command::DisplayClockDiv(0x8, 0x0).send(*iface).await?;
        i2c::Command::Multiplex(display_height - 1).send(*iface).await?;

        i2c::Command::StartLine(0).send(*iface).await?;
        // TODO: Ability to turn charge pump on/off
        // Display must be off when performing this command
        i2c::Command::ChargePump(true).send(*iface).await?;

        i2c::Command::Contrast(0x80).send(*iface).await?;
        i2c::Command::PreChargePeriod(0x1, 0xF).send(*iface).await?;
        i2c::Command::VcomhDeselect(i2c::VcomhLevel::Auto).send(*iface).await?;
        i2c::Command::AllOn(false).send(*iface).await?;
        i2c::Command::Invert(false).send(*iface).await?;
        i2c::Command::DisplayOn(true).send(*iface).await?;

        // Spisific to 128 x 128
        i2c::Command::DisplayOffset(0).send(*iface).await?;
        i2c::Command::ComPinConfig(true).send(*iface).await?;

        for x in (0..self.width).step_by(self.chunk_width as usize) {
            for y in (0..self.height).step_by(self.chunk_height as usize) {
                self.set_chunk(x, y)?;
                self.clear()?;
                self.flush().await?;
            }
        }
        Ok(())
    }
}

impl<'a, DI, const BUFFER_SIZE: usize> Renderer for Sh1107Render<'a, DI, BUFFER_SIZE> 
where
    DI: AsyncWriteOnlyDataCommand,
{
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn chunk_size(&self) -> (u32, u32) {
        (self.chunk_width, self.chunk_height)
    }

    // Set the current chunk by the top left x and y offset in pixels.
    fn set_chunk(&mut self, x: u32, y: u32) -> Result<(), RendererError> {
        if (x % self.chunk_width != 0) || (y % self.chunk_height != 0) {
            return Err(RendererError::InvalidChunkOffset{x, y})
        }

        self.clip = BoundingBox{
            x1: x,
            y1: y,
            x2: x + self.chunk_width,
            y2: y + self.chunk_height,
        };

        Ok(())
    }

    fn draw(&mut self, command: &Command) -> Result<(), RendererError> {

        use CommandType::*;
        let clip = self.clip;
        match command.flavor {
            Null => Ok(()),
            Rect(rgb) => {

                let color = rgb.r|rgb.g|rgb.b;

                let x1 = max(command.bounds.x1, clip.x1);
                let y1 = max(command.bounds.y1, clip.y1);
                let x2 = min(command.bounds.x2, clip.x2);
                let y2 = min(command.bounds.y2, clip.y2);

                // Offset from chunk left
                let x = x1 - clip.x1;
                let x_end = x2 - clip.x1;
                // Offset fron chunk top
                let y = y1 - clip.y1;
                let y_end = y2 - clip.y1;

                // Each byte is a 8 pixel high column with the fist chunk_width bytes
                // being row 0-7 and each consecutive chunk_width bytes being the 
                // next 8 row. 
                for x_i in x..x_end {
                    for y_i in y..y_end {
                        let index = x_i * (y_i/8 + 1);
                        let byte = &mut self.buffer[index as usize];
                        let bit = y_i % 8;

                        let set_bit = 1u8<<bit;
                        if color > 0 {
                            *byte |= set_bit;
                        } else {
                            *byte &= !set_bit;
                        }
                    }
                }

                Ok(())
            }
        }
    }

    fn clear(&mut self) -> Result<(), RendererError> {
        self.buffer = [0x00u8;BUFFER_SIZE];
        Ok(())
    }

    
    async fn flush(&mut self) -> Result<(), RendererError> {

        let column_start = self.clip.x1 as u8;
        let row_start = (self.clip.y1/8) as u8;
        let row_end = row_start + (self.chunk_height as u8 /8);

        for (index,row) in (row_start..row_end).enumerate() {
            i2c::Command::PageAddress(row)
                .send(self.display)
                .await?;
            i2c::Command::ColumnAddressLow(0xF & column_start)
            .send(self.display)
            .await?;
            i2c::Command::ColumnAddressHigh(0xF & (column_start >> 4))
            .send(self.display)
            .await?;

            let start = self.chunk_width as usize * index;
            let end = start + self.chunk_width as usize;
            let data: DataFormat<'_> = DataFormat::U8(&self.buffer[start..end]);
            self.display.send_data(data).await?;
        }
        Ok(())
    }

    
}