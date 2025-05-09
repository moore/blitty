use super::*;

use smol;

use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
};

use embedded_graphics_simulator::{SimulatorDisplay, Window, OutputSettingsBuilder};


fn test2() -> Result<(), DisplayListError> {
    let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(320, 240));

    let mut renderer = embedded_render::EmbeddedRender::new(&mut display, 16, 16);

    let mut commands = DisplayList::<3>::new();


    let bounds = BoundingBox::new( 176, 16, 192, 176);
    let rgb = Rgb::new( 0, 64, 128 );

    let rect = Command::new_rect(bounds, rgb);

    commands.set(0, rect)?;

    let bounds = BoundingBox::new( 208, 16, 224, 176);
    let rgb = Rgb::new( 0, 128, 64 );

    let rect = Command::new_rect(bounds, rgb);

    commands.set(2, rect)?;

    let mut bounds = BoundingBox::new( 64, 32, 160, 128);
    let mut rgb = Rgb::new( 64, 64, 64 );

    let rect = Command::new_rect(bounds.clone(), rgb.clone());

    commands.set(1, rect)?;


    smol::block_on(commands.draw(&mut renderer))?;


    let output_settings = OutputSettingsBuilder::new()
        .build();

    let mut window = Window::new("Hello World", &output_settings);

    window.update(renderer.get_display());

    for _ in 0..128 {
        bounds.x1 += 1;
        bounds.x2 += 1;
        rgb.r += 1;

        let rect = Command::new_rect(bounds.clone(), rgb.clone());

        commands.update(1, rect)?;

        smol::block_on(commands.draw(&mut renderer))?;

        window.update(renderer.get_display());
    }

    Ok(())
}

#[test]
fn it_works2() {
    let result = test2();

    assert_eq!(result, Ok(()));
}