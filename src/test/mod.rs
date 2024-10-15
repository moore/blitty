use std::ops::Bound;

use super::*;


use embedded_graphics::{
    pixelcolor::{BinaryColor, Rgb888},
    prelude::*,
    primitives::{Circle, Line, Rectangle, PrimitiveStyle},
    mono_font::{ascii::FONT_6X9, MonoTextStyle},
    text::Text,
};

use embedded_graphics_simulator::{BinaryColorTheme, SimulatorDisplay, Window, OutputSettingsBuilder};


fn test2() -> Result<(), DisplayListError> {
    let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(640, 480));

    let mut renderer = embedded_render::EmbeddedRender::new(display, 64);

    let mut commands = DisplayList::<100>::new();

    let mut bounds = BoundingBox::new( 64, 32, 256, 128);
    let mut rgb = Rgb::new( 64, 64, 64 );


    let output_settings = OutputSettingsBuilder::new()
        //.theme(BinaryColorTheme::OledBlue)
        .build();

    let mut window = Window::new("Hello World", &output_settings);

    for i in 0..128 {
        bounds.x1 += 1;
        bounds.x2 += 1;
        rgb.r += 1;

        let rect = Command::new_rect(bounds.clone(), rgb.clone());

        commands.set(0, rect)?;

        commands.draw(&mut renderer)?;

        window.update(renderer.get_display());
    }

    Ok(())
}

/*
#[test]
fn it_works() {
    let result = test1();

    assert_eq!(result, Ok(()));
}
*/

#[test]
fn it_works2() {
    let result = test2();

    assert!(result.is_ok());
}