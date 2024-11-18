extern crate sdl2;

use std::{env, fs};

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::{event::Event, rect::Point};

use celeste_map::{Element, Map, SessionData};

const EDITOR_COLORS: [Color; 7] = [
    Color::WHITE,
    Color::RGB(0xf6, 0x73, 0x5e),
    Color::RGB(0x85, 0xf6, 0x5e),
    Color::RGB(0x37, 0xd7, 0xe3),
    Color::RGB(0x37, 0x6b, 0xe3),
    Color::RGB(0xc3, 0x37, 0xe3),
    Color::RGB(0xe3, 0x37, 0x73),
];

const INACTIVE_BORDER_COLOR: Color = Color::RGB(0x2f, 0x4f, 0x4f);

fn main() {
    if cfg!(target_os = "linux") {
        if env::var("SDL_VIDEODRIVER").is_err() {
            env::set_var("SDL_VIDEODRIVER", "wayland");
        }
    }
    let maps_path = fs::read_to_string("./maps_path.txt")
        .unwrap()
        .trim()
        .to_string();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut map = Map::default();
    let mut map_bin = String::new();

    let window = video_subsystem
        .window("Celeste Map", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        let session_data = SessionData::get().unwrap_or_default();

        if session_data.map_bin != "" && session_data.map_bin != map_bin {
            let path = format!("{maps_path}/{}.bin", session_data.map_bin);
            println!("Loading map from path '{path}'");
            map = Element::from_file(path).try_into().unwrap();
            map_bin = session_data.map_bin.clone();
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        if session_data.map_bin != "" {
            let mut player_x = 0;
            let mut player_y = 0;
            let viewport = canvas.viewport();
            let mut camera = Rect::new(
                player_x - viewport.w / 2,
                player_y - viewport.h / 2,
                viewport.w as u32,
                viewport.h as u32,
            );
            for level in map.levels.iter() {
                if level.name == session_data.level {
                    player_x = level.x + session_data.x as i32;
                    player_y = level.y + session_data.y as i32;
                    camera = Rect::new(
                        player_x - viewport.w / 2,
                        player_y - viewport.h / 2,
                        viewport.w as u32,
                        viewport.h as u32,
                    );
                }
            }

            for level in map.levels.iter() {
                let level_rect = Rect::new(level.x, level.y, level.width, level.height);
                if level_rect.intersection(camera).is_none() {
                    continue;
                }
                canvas.set_draw_color(Color::RGBA(23, 39, 39, 127));
                for (y2, line) in level.bg.lines().enumerate() {
                    for (x2, char) in line.chars().enumerate() {
                        if char != '0' {
                            let point = world_to_screen(
                                level.x + x2 as i32 * 8,
                                level.y + y2 as i32 * 8,
                                camera,
                                viewport,
                            );
                            canvas
                                .fill_rect(Rect::new(
                                    point.x, point.y,
                                    // viewport.w / 2 + level.x + x2 as i32 * 8 - player_x,
                                    // viewport.h / 2 + level.y + y2 as i32 * 8 - player_y,
                                    8, 8,
                                ))
                                .unwrap()
                        }
                    }
                }
                canvas.set_draw_color(EDITOR_COLORS[level.c as usize]);
                for (y2, line) in level.solids.lines().enumerate() {
                    for (x2, char) in line.chars().enumerate() {
                        if char != '0' {
                            let point = world_to_screen(
                                level.x + x2 as i32 * 8,
                                level.y + y2 as i32 * 8,
                                camera,
                                viewport,
                            );
                            canvas
                                .fill_rect(Rect::new(
                                    point.x, point.y,
                                    // viewport.w / 2 + level.x + x2 as i32 * 8 - player_x,
                                    // viewport.h / 2 + level.y + y2 as i32 * 8 - player_y,
                                    8, 8,
                                ))
                                .unwrap()
                        }
                    }
                }
                canvas.set_draw_color(INACTIVE_BORDER_COLOR);
                canvas
                    .draw_line(
                        Point::new(
                            viewport.w / 2 + level.x - player_x,
                            viewport.h / 2 + level.y - player_y,
                        ),
                        Point::new(
                            viewport.w / 2 + level.x - player_x + level.width as i32,
                            viewport.h / 2 + level.y - player_y,
                        ),
                    )
                    .unwrap();
                canvas
                    .draw_line(
                        Point::new(
                            viewport.w / 2 + level.x - player_x + level.width as i32,
                            viewport.h / 2 + level.y - player_y,
                        ),
                        Point::new(
                            viewport.w / 2 + level.x - player_x + level.width as i32,
                            viewport.h / 2 + level.y - player_y + level.height as i32,
                        ),
                    )
                    .unwrap();
                canvas
                    .draw_line(
                        Point::new(
                            viewport.w / 2 + level.x - player_x + level.width as i32,
                            viewport.h / 2 + level.y - player_y + level.height as i32,
                        ),
                        Point::new(
                            viewport.w / 2 + level.x - player_x,
                            viewport.h / 2 + level.y - player_y + level.height as i32,
                        ),
                    )
                    .unwrap();
                canvas
                    .draw_line(
                        Point::new(
                            viewport.w / 2 + level.x - player_x,
                            viewport.h / 2 + level.y - player_y + level.height as i32,
                        ),
                        Point::new(
                            viewport.w / 2 + level.x - player_x,
                            viewport.h / 2 + level.y - player_y,
                        ),
                    )
                    .unwrap();
            }

            canvas.set_draw_color(Color::RGB(0, 255, 0));

            canvas
                .fill_rect(Rect::new(viewport.w / 2 - 4, viewport.h / 2 - 11, 8, 11))
                .unwrap();
        }

        canvas.present();
    }
}

enum State {}

fn world_to_screen(x: i32, y: i32, camera: Rect, viewport: Rect) -> Point {
    // this is where linear algebra might be helpful
    let x_new = ((x as f64 - camera.x as f64) / camera.w as f64) * viewport.w as f64; // + viewport.x;
    let y_new = ((y as f64 - camera.y as f64) / camera.h as f64) * viewport.h as f64; // + viewport.y;

    Point::new(x_new.round() as i32, y_new.round() as i32)
}
