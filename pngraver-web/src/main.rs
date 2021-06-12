use std::{io::prelude::*, str::FromStr};

use image::{EncodableLayout, ImageEncoder};
use pngraver_core::*;

#[macro_use]
extern crate rouille;

#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(remote = "Neighboors")]
struct NeighboorsDef(#[serde(getter = "panic")] String);
impl From<NeighboorsDef> for Neighboors {
    fn from(def: NeighboorsDef) -> Neighboors {
        Neighboors::from_str(&def.0).unwrap()
    }
}

#[derive(Deserialize)]
pub struct Config {
    file: (String, String),
    #[serde(with = "NeighboorsDef")]
    neighboors: Neighboors,
    add: f64,
    mult: f64,
    inv: bool,
    gray: bool,
}

impl Config {
    fn parse(self) -> Option<(Matrix<(u8, u8, u8)>, DiffConf)> {
        let content = base64::decode(&self.file.1).ok()?;
        let img = image::load_from_memory(&content).ok()?;
        let img = img.as_rgb8()?;
        let (w, h) = img.dimensions();
        let buf = {
            let mut buf = Vec::new();
            let t = img.as_bytes();
            for i in 0..(t.len() / 3) {
                buf.push((t[i * 3], t[i * 3 + 1], t[i * 3 + 2]));
            }
            buf
        };

        let conf = DiffConf {
            neiboors: self.neighboors,
            add: self.add,
            mult: self.mult,
            inv: self.inv,
            gray: self.gray,
        };

        Some((Matrix::new(buf, w as usize, h as usize), conf))
    }
}

static INDEX: &'static str = include_str!("../static/index.html");
static NOTFOUND: &'static str = include_str!("../static/404.html");

fn main() {
    println!("Now listening on localhost:8000");

    rouille::start_server("127.0.0.1:8000", move |request| {
        router!(request,
            (GET) (/) => {
                rouille::Response::from_data("text/html", INDEX)
            },

            (POST) (/) => {
                let mut s = String::new();
                let data = serde_json::from_str::<Config>({
                    request.data().unwrap().read_to_string(&mut s).ok().unwrap();
                    &s[..]
                }).unwrap();
                let (image, conf) = data.parse().unwrap();
                let res = make_grave(image, conf);

                let mut c = std::io::Cursor::new(Vec::new());
                image::codecs::png::PngEncoder::new(&mut c)
                    .write_image(
                        &res.as_slice().iter().flat_map(|x| vec![x.0, x.1, x.2]).collect::<Vec<_>>()[..],
                        res.width() as u32,
                        res.height() as u32,
                        image::ColorType::Rgb8,
                ).unwrap();

                rouille::Response::from_data("image/png", c.into_inner())
            },

            _ => rouille::Response::from_data("text/html", NOTFOUND).with_status_code(404)
        )
    });
}
