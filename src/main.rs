extern crate sdl2;

use reqwest::blocking::Client;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, RenderTarget};
use sdl2::{event::Event, rect::Point};

use celeste_map::{get_path_from_config, Element, Map};

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
const GRID_COLOR: Color = Color::RGB(0x19, 0x19, 0x19);

fn main() {
    let mut path = get_path_from_config();
    path += "/Content/Maps";
    println!("{path}");
    // if cfg!(target_os = "linux") && env::var("SDL_VIDEODRIVER").is_err() {
    //     env::set_var("SDL_VIDEODRIVER", "wayland");
    // }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Celeste Map", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_blend_mode(BlendMode::Blend);

    let mut state = State::new(&path);

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        state.update();

        state.draw(&mut canvas);
    }
}

#[derive(Debug)]
struct State {
    session_data: SessionData,
    viewport: Rect,
    camera: Rect,
    map: Map,
    map_bin: String,
    maps_path: String,
    http_client: Client,
}

impl State {
    fn new(maps_path: &str) -> State {
        State {
            session_data: SessionData::default(),
            viewport: Rect::new(0, 0, 0, 0),
            camera: Rect::new(0, 0, 0, 0),
            map: Map::default(),
            map_bin: String::new(),
            maps_path: maps_path.into(),
            http_client: Client::new(),
        }
    }

    fn update(&mut self) {
        let session_data = self.get_session().unwrap_or_default();

        if !session_data.map_bin.is_empty() && session_data.map_bin != self.map_bin {
            let path = format!("{}/{}.bin", self.maps_path, session_data.map_bin);
            println!("Loading map from path '{path}'");
            self.map = Element::from_file(path).try_into().unwrap();
            self.map_bin = session_data.map_bin.clone();
        }
        self.session_data = session_data;
    }

    fn draw<T: RenderTarget>(&mut self, canvas: &mut Canvas<T>) {
        self.viewport = canvas.viewport();

        canvas.set_draw_color(Color::RGB(0, 0, 0));

        canvas.clear();

        if !self.session_data.map_bin.is_empty() {
            for level in self.map.levels.iter() {
                if level.name == self.session_data.level {
                    let player_x = level.x + self.session_data.x as i32;
                    let player_y = level.y + self.session_data.y as i32;
                    self.camera = Rect::new(
                        player_x - self.viewport.w / 2,
                        player_y - self.viewport.h / 2,
                        self.viewport.w as u32,
                        self.viewport.h as u32,
                    );
                }
            }

            self.draw_grid(canvas);

            self.draw_levels(canvas);

            self.draw_player(canvas);
        }

        canvas.present();
    }

    fn draw_grid<T: RenderTarget>(&mut self, canvas: &mut Canvas<T>) {
        const GRID_INTERVAL: i32 = 8 * 5;
        const GRID_WIDTH: i32 = 8;
        canvas.set_draw_color(GRID_COLOR);
        let mut x = self.camera.x / GRID_INTERVAL * GRID_INTERVAL - 8;
        while x > self.camera.x - 8 {
            x -= GRID_INTERVAL;
        }
        while x < self.camera.x + self.camera.w {
            let point = self.world_to_screen(x, 0);
            canvas
                .fill_rect(Rect::new(point.x, 0, 8, self.viewport.h as u32))
                .unwrap();
            x += GRID_INTERVAL;
        }
        let mut y = self.camera.y / GRID_INTERVAL * GRID_INTERVAL;
        while y > self.camera.y - 8 {
            y -= GRID_INTERVAL;
        }
        while y < self.camera.y + self.camera.h {
            let point = self.world_to_screen(0, y);
            canvas
                .fill_rect(Rect::new(0, point.y, self.viewport.w as u32, 8))
                .unwrap();
            y += GRID_INTERVAL;
        }
    }

    fn draw_levels<T: RenderTarget>(&mut self, canvas: &mut Canvas<T>) {
        for level in self.map.levels.iter() {
            let level_rect = Rect::new(level.x, level.y, level.width, level.height);
            if level_rect.intersection(self.camera).is_none() {
                continue;
            }
            canvas.set_draw_color(Color::RGBA(0x2f, 0x4f, 0x4f, 127));
            for (y2, line) in level.bg.lines().enumerate() {
                for (x2, char) in line.chars().enumerate() {
                    if char != '0' {
                        let point =
                            self.world_to_screen(level.x + x2 as i32 * 8, level.y + y2 as i32 * 8);
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
                        let point =
                            self.world_to_screen(level.x + x2 as i32 * 8, level.y + y2 as i32 * 8);
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
            let start = self.world_to_screen(level.x, level.y);
            let end =
                self.world_to_screen(level.x + level.width as i32, level.y + level.height as i32);
            canvas
                .draw_rect(Rect::new(
                    start.x,
                    start.y,
                    (end.x - start.x) as u32,
                    (end.y - start.y) as u32,
                ))
                .unwrap();
        }
    }

    fn draw_player<T: RenderTarget>(&mut self, canvas: &mut Canvas<T>) {
        canvas.set_draw_color(Color::RGB(0, 255, 0));

        canvas
            .fill_rect(Rect::new(
                self.viewport.w / 2 - 4,
                self.viewport.h / 2 - 11,
                8,
                11,
            ))
            .unwrap();
    }

    fn world_to_screen(&self, x: i32, y: i32) -> Point {
        let x_new =
            ((x as f64 - self.camera.x as f64) / self.camera.w as f64) * self.viewport.w as f64; // + self.viewport.x;
        let y_new =
            ((y as f64 - self.camera.y as f64) / self.camera.h as f64) * self.viewport.h as f64; // + self.viewport.y;

        Point::new(x_new.round() as i32, y_new.round() as i32)
    }

    /// Fetch Session Data from the Everest debug web api.
    ///
    /// Returns `None` on failure
    fn get_session(&self) -> Option<SessionData> {
        let data = self
            .http_client
            .get("http://localhost:32270/session")
            .send()
            .ok()?
            .text()
            .ok()?;

        let mut lines = data.lines();
        let area = data_assert_key_value(lines.next()?, "Area")?;
        let side = data_assert_key_value(lines.next()?, "Side")?;
        let level = data_assert_key_value(lines.next()?, "Level")?;
        let map_bin = data_assert_key_value(lines.next()?, "MapBin")?;
        let x = data_assert_key_value(lines.next()?, "X")?;
        let y = data_assert_key_value(lines.next()?, "Y")?;
        let tp = data_assert_key_value(lines.next()?, "TP")?;

        let session = SessionData {
            area: area.into(),
            side: side.into(),
            level: level.into(),
            map_bin: map_bin.into(),
            x: x.parse().ok()?,
            y: y.parse().ok()?,
            tp: tp.into(),
        };

        Some(session)
    }
}

#[derive(Debug)]
pub struct SessionData {
    pub area: String,
    pub side: String,
    pub level: String,
    pub map_bin: String,
    pub x: f32,
    pub y: f32,
    pub tp: String,
}

impl Default for SessionData {
    fn default() -> Self {
        SessionData {
            area: "".into(),
            side: "'?'".into(),
            level: "".into(),
            map_bin: "".into(),
            x: 0.0,
            y: 0.0,
            tp: "''".into(),
        }
    }
}

fn data_assert_key_value<'a>(data: &'a str, key: &str) -> Option<&'a str> {
    let (found_key, value) = data.split_once(": ")?;
    if key != found_key {
        return None;
    }
    Some(value)
}
