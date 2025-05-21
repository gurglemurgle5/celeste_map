extern crate sdl2;

use reqwest::blocking::Client;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, RenderTarget};
use sdl2::{event::Event, rect::Point};

use celeste_map::{get_path_from_config, Element, Map};

fn main() {
    let mut path = get_path_from_config();
    path += "/Content/Maps";
    eprintln!("Maps will be loaded from {path}");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Celeste Map", 960, 540)
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
    player_pos: Point,
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
            player_pos: Point::new(0, 0),
        }
    }

    fn update(&mut self) {
        let session_data = self.get_session().unwrap_or_default();

        if !session_data.map_bin.is_empty() && session_data.map_bin != self.map_bin {
            let path = format!("{}/{}.bin", self.maps_path, session_data.map_bin);
            eprintln!("Loading map from path '{path}'");
            let element = Element::from_file(path);
            // println!("{element:#?}");
            self.map = element.try_into().unwrap();
            self.map_bin = session_data.map_bin.clone();
        }
        self.session_data = session_data;
    }

    fn draw<T: RenderTarget>(&mut self, canvas: &mut Canvas<T>) {
        self.viewport = canvas.viewport();

        canvas.set_draw_color(Color::RGB(0, 0, 0));

        canvas.clear();

        if !self.session_data.map_bin.is_empty() {
            for level in &self.map.levels {
                if level.name == self.session_data.level {
                    let player_x = level.x + self.session_data.x;
                    let player_y = level.y + self.session_data.y;
                    self.player_pos = Point::new(player_x, player_y);
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
        const GRID_COLOR: Color = Color::RGB(0x19, 0x19, 0x19);

        const GRID_INTERVAL: i32 = 8 * 5;
        const GRID_WIDTH: u32 = 8;
        canvas.set_draw_color(GRID_COLOR);
        let mut x = self.camera.x / GRID_INTERVAL * GRID_INTERVAL - 8;
        while x > self.camera.x - GRID_WIDTH as i32 {
            x -= GRID_INTERVAL;
        }
        while x < self.camera.x + self.camera.w {
            let point = self.translate_point((x, 0));
            canvas
                .fill_rect(Rect::new(point.x, 0, GRID_WIDTH, self.viewport.h as u32))
                .unwrap();
            x += GRID_INTERVAL;
        }
        let mut y = self.camera.y / GRID_INTERVAL * GRID_INTERVAL;
        while y > self.camera.y - GRID_WIDTH as i32 {
            y -= GRID_INTERVAL;
        }
        while y < self.camera.y + self.camera.h {
            let point = self.translate_point((0, y));
            canvas
                .fill_rect(Rect::new(0, point.y, self.viewport.w as u32, GRID_WIDTH))
                .unwrap();
            y += GRID_INTERVAL;
        }
    }

    fn draw_levels<T: RenderTarget>(&mut self, canvas: &mut Canvas<T>) {
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

        for level in &self.map.levels {
            let level_rect = Rect::new(level.x, level.y, level.width, level.height);
            if level_rect.intersection(self.camera).is_none() {
                continue;
            }
            canvas.set_draw_color(Color::RGBA(0x2f, 0x4f, 0x4f, 127));
            for (y2, line) in level.bg.lines().enumerate() {
                for (x2, char) in line.chars().enumerate() {
                    if char != '0' {
                        let rect =
                            Rect::new(level.x + x2 as i32 * 8, level.y + y2 as i32 * 8, 8, 8);
                        canvas.fill_rect(self.translate_rect(rect)).unwrap();
                    }
                }
            }
            canvas.set_draw_color(EDITOR_COLORS[level.c as usize]);
            for (y2, line) in level.solids.lines().enumerate() {
                for (x2, char) in line.chars().enumerate() {
                    if char != '0' {
                        let rect =
                            Rect::new(level.x + x2 as i32 * 8, level.y + y2 as i32 * 8, 8, 8);
                        canvas.fill_rect(self.translate_rect(rect)).unwrap();
                    }
                }
            }

            canvas.set_draw_color(INACTIVE_BORDER_COLOR);
            canvas.draw_rect(self.translate_rect(level_rect)).unwrap();
        }
    }

    fn draw_player<T: RenderTarget>(&mut self, canvas: &mut Canvas<T>) {
        const PLAYER_COLOR: Color = Color::RGB(0xAC, 0x32, 0x32);
        canvas.set_draw_color(PLAYER_COLOR);

        let player_rect = Rect::new(self.player_pos.x - 4, self.player_pos.y - 11, 8, 11);

        canvas.fill_rect(self.translate_rect(player_rect)).unwrap();
    }

    fn translate_point<T: Into<Point>>(&self, point: T) -> Point {
        let (x, y): (i32, i32) = point.into().into();
        let x_new =
            ((x as f64 - self.camera.x as f64) / self.camera.w as f64) * self.viewport.w as f64; // + self.viewport.x;
        let y_new =
            ((y as f64 - self.camera.y as f64) / self.camera.h as f64) * self.viewport.h as f64; // + self.viewport.y;

        Point::new(x_new.round() as i32, y_new.round() as i32)
    }

    fn translate_rect(&self, rect: Rect) -> Rect {
        let (x_new, y_new) = self.translate_point((rect.x, rect.y)).into();
        let width = self.viewport.w as f64 / self.camera.w as f64 * rect.w as f64;
        let height = self.viewport.h as f64 / self.camera.h as f64 * rect.h as f64;
        Rect::new(x_new, y_new, width.round() as u32, height.round() as u32)
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
            x: x.parse::<f32>().ok()?.round() as i32,
            y: y.parse::<f32>().ok()?.round() as i32,
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
    pub x: i32,
    pub y: i32,
    pub tp: String,
}

impl Default for SessionData {
    fn default() -> Self {
        SessionData {
            area: String::new(),
            side: "'?'".into(),
            level: String::new(),
            map_bin: String::new(),
            x: 0,
            y: 0,
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
