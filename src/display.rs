extern crate sdl2;

use self::sdl2::pixels::Color;
use std::time::Duration;
use self::sdl2::event::Event;
use sdl2::keyboard::Keycode;
use self::sdl2::rect::{Point, Rect};
use self::sdl2::render::{TextureCreator, Canvas, Texture, CanvasBuilder};
use self::sdl2::video::{WindowContext, Window};
use synacor_cpu::{utils, constants::*, Machine};

pub fn frontpanelRun(m0:&mut Machine) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("synacor VM frontpanel", 2550, 1600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    //let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut canvas:Canvas<Window> = window.into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0,0,0));
    canvas.clear();
    canvas.present();

    let texture_creator:TextureCreator<_> = canvas.texture_creator();
    let mut square_texture1 = texture_creator.create_texture_target(texture_creator.default_pixel_format(), 3048, 2048).map_err(|e| e.to_string())?;
    canvas.with_texture_canvas(&mut square_texture1, |texture_canvas| {
        texture_canvas.set_draw_color(Color::RGB(0,0,0));
        texture_canvas.clear();
        texture_canvas.set_draw_color(Color::RGB(255,0,0));
        texture_canvas.fill_rect(Rect::new(1, 1, 100, 100)).unwrap();
        texture_canvas.draw_point(Point::new(0,0));
        }
    ).map_err(|e| e.to_string())?;

    let mut x:u16 = 0;
    let mut event_pump = sdl_context.event_pump()?;
    let mut speed:u16 = 1023;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Minus), .. } => {
                    if speed <= 127 {
                        speed = 1;
                    } else {
                        speed -= 127;
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Equals), .. } => {
                    if speed >= 0xFFFF {
                        speed = 0xFFFF;
                    } else {
                        speed += 127;
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    if m0.is_halted() {
                        utils::clear_bit(&mut m0.status, HALT_BIT);
                    } else {
                        m0.halt();
                        m0.dump();
                    }
                },
                _ => {}
            }
        }

        //::std::thread::sleep(Duration::new(0, 1_100_100_100u32 /30));
        for i in 0..16 {
            if utils::get_bit(&m0.status, i) {
                canvas.set_draw_color(Color::RGB(255, 0, 0));
            } else {
                canvas.set_draw_color(Color::RGB(0, 0, 255));
            }
            canvas.fill_rect(Rect::new(10*i as i32, 0, 10, 20));
        }

        if x % speed == 0 {
            canvas.present();
            draw_empty_cells(TOM as u16, &mut canvas, &m0);
        }
        for access_location in m0.recentMemAccess.iter() {
            if access_location.1 == RECENTMEMACCESS_READ_BIT {
                draw_filled_cell(access_location.0, Color::RGB(0, 255, 0), &mut canvas);
            } else if access_location.1 == RECENTMEMACCESS_WRITE_BIT {
                draw_filled_cell(access_location.0, Color::RGB(255, 0, 0), &mut canvas);
            }
        }
        m0.recentMemAccess.clear();

        if !m0.is_halted() {
            m0.fetch_and_execute();
        }
        x = x.wrapping_add(1);
    }

    Ok(())
}

fn draw_empty_cells(up_to:u16, canvas:&mut Canvas<Window>, machine:&Machine) {
    const y_offset:i32 = 20;
    const rect_width:u32 = 40;
    const rect_height:u32 = 3;
    canvas.set_draw_color(Color::RGB(128, 128, 128));
    for y in 0..(TOM as i32) / 64 {
        for x in 0..64 {
            let val:u16 = utils::swap_endian(machine.mem[((y*64)+x) as usize]);
            // let val:u16 = machine.mem[((y*64)+x) as usize];

            if val > 0x001F && val < 0x007F {
                canvas.set_draw_color(Color::RGB(0, 0, (val % 255) as u8));
            } else {
                canvas.set_draw_color(Color::RGB((val / 255) as u8, (val / 255) as u8,(val / 255) as u8));
            }
            // canvas.set_draw_color(Color::RGB((machine.mem[(y*x) as usize] >> 8) as u8, 0, 0));
            canvas.fill_rect(Rect::new(x * rect_width as i32, (y * rect_height as i32) + y_offset, rect_width, rect_height));
        }
    }
}

fn draw_filled_cell(mem_location:u16, color:Color, canvas:&mut Canvas<Window>) {
    const y_offset:i32 = 20;
    const rect_width:i32 = 40;
    const rect_height:i32 = 3;
    canvas.set_draw_color(color);

    let mut x:i32 = (mem_location % 64) as i32;
    x *= rect_width;

    let mut y:i32 = (mem_location / 64) as i32;
    y *= rect_height;
    y += y_offset;

    //canvas.fill_rect(Rect::new(x * rect_width as i32, (y * rect_height as i32) + y_offset, rect_width, rect_height));
    canvas.fill_rect(Rect::new(x, y, rect_width as u32, rect_height as u32));
}

/*fn dummy_texture<'a>(canvas: &mut Canvas<Window>, texture_creator: &'a TextureCreator<WindowContext>) -> Result<(Texture<'a>, Texture<'a>), String> {

}
 */