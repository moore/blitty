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

fn test1() -> Result<(), core::convert::Infallible> {
    let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(128, 64));

    let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    let text_style = MonoTextStyle::new(&FONT_6X9, BinaryColor::On);

    Circle::new(Point::new(72, 8), 48)
        .into_styled(line_style)
        .draw(&mut display)?;

    Line::new(Point::new(48, 16), Point::new(8, 16))
        .into_styled(line_style)
        .draw(&mut display)?;

    Line::new(Point::new(48, 16), Point::new(64, 32))
        .into_styled(line_style)
        .draw(&mut display)?;

    Rectangle::new(Point::new(79, 15), Size::new(34, 34))
        .into_styled(line_style)
        .draw(&mut display)?;

    Text::new("Hello World!", Point::new(5, 5), text_style).draw(&mut display)?;

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Hello World", &output_settings).show_static(&display);

    Ok(())
}

fn test2() -> Result<(), DisplayListError> {
    let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(320, 240));

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