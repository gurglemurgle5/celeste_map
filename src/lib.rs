mod element;
mod map;

pub use element::*;
pub use map::*;

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

impl SessionData {
    /// Fetch Session Data from the Everest debug web api.
    ///
    /// Returns `None` on failure
    pub fn get() -> Option<SessionData> {
        let data = reqwest::blocking::get("http://localhost:32270/session")
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
