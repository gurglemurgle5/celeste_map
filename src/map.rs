use crate::Element;

#[derive(Debug, Default)]
pub struct Map {
    pub levels: Vec<Level>,
    pub filler: Vec<Filler>,
    pub style: Style,
}

impl TryFrom<Element> for Map {
    type Error = ();
    fn try_from(value: Element) -> Result<Self, Self::Error> {
        let mut levels = None;
        let mut filler = None;
        let mut style = None;

        for child in value.children {
            if child.name == "levels" {
                levels = Some(
                    child
                        .children
                        .into_iter()
                        .map(|child| child.into())
                        .collect(),
                );
            } else if child.name == "Filler" {
                filler = Some(
                    child
                        .children
                        .into_iter()
                        .map(|child| child.into())
                        .collect(),
                );
            } else if child.name == "Style" {
                style = Some(child.into());
            }
        }

        Ok(Map {
            levels: levels.unwrap(),
            filler: filler.unwrap_or_default(),
            style: style.unwrap(),
        })
    }
}

#[derive(Debug)]
pub struct Level {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub solids: String,
    pub bg: String,
    pub name: String,
    pub c: i32,
}

impl From<Element> for Level {
    fn from(value: Element) -> Self {
        debug_assert_eq!(&value.name, "level");
        let x = value.attributes.get("x").unwrap().as_int().unwrap();
        let y = value.attributes.get("y").unwrap().as_int().unwrap();
        let width = value.attributes.get("width").unwrap().as_int().unwrap() as u32;
        let height = value.attributes.get("height").unwrap().as_int().unwrap() as u32;
        let mut name: String = value
            .attributes
            .get("name")
            .unwrap()
            .as_str()
            .unwrap()
            .into();
        let c = value.attributes.get("c").unwrap().as_int().unwrap();

        if name.starts_with("lvl_") {
            name = name[4..].into();
        }

        let mut solids = None;
        let mut bg = None;

        for child in value.children.iter() {
            if child.name == "solids" {
                solids = child
                    .attributes
                    .get("innerText")
                    .map(|text| text.as_str().unwrap().into());
            }
            if child.name == "bg" {
                bg = child
                    .attributes
                    .get("innerText")
                    .map(|text| text.as_str().unwrap().into());
            }
        }
        Level {
            x,
            y,
            width,
            height,
            solids: solids.unwrap_or_default(),
            name,
            bg: bg.unwrap_or_default(),
            c,
        }
    }
}

#[derive(Debug)]
pub struct Filler {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl From<Element> for Filler {
    fn from(value: Element) -> Self {
        debug_assert_eq!(&value.name, "rect");
        Filler {
            x: value.attributes.get("x").unwrap().as_int().unwrap(),
            y: value.attributes.get("y").unwrap().as_int().unwrap(),
            w: value.attributes.get("w").unwrap().as_int().unwrap() as u32,
            h: value.attributes.get("h").unwrap().as_int().unwrap() as u32,
        }
    }
}

#[derive(Debug, Default)]
pub struct Style {}

impl From<Element> for Style {
    fn from(_value: Element) -> Self {
        Style {}
    }
}
