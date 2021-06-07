use std::{
    fmt::Display,
    ops::{Index, IndexMut},
    str::FromStr,
};

#[macro_use]
extern crate structopt;
pub struct Matrix<T> {
    data: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> Matrix<T> {
    pub fn new(data: Vec<T>, width: usize, height: usize) -> Matrix<T> {
        assert!(data.len() == width * height);
        Matrix {
            data,
            width,
            height,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

impl<T: Clone + Default> Matrix<T> {
    pub fn zeroed(width: usize, height: usize) -> Matrix<T> {
        Matrix {
            data: vec![T::default(); width * height],
            width,
            height,
        }
    }
}

impl<T> Index<usize> for Matrix<T> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[(index * self.width)..((index + 1) * self.width)]
    }
}

impl<T> IndexMut<usize> for Matrix<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[(index * self.width)..((index + 1) * self.width)]
    }
}

#[derive(Debug, Clone, Copy)]
enum Dist {
    One,
    Sqrt2,
    Inf,
}

#[derive(Debug, Clone)]
struct Neighboors([Dist; 9]);

#[derive(Debug)]
enum ParseNeighboorsError {
    InvalidLength,
    InvalidDigit,
}

impl Display for ParseNeighboorsError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

impl FromStr for Neighboors {
    type Err = ParseNeighboorsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 9 {
            Err(ParseNeighboorsError::InvalidLength)
        } else {
            let mut res = [Dist::Inf; 9];
            let mut n = s
                .parse::<u32>()
                .map_err(|_| ParseNeighboorsError::InvalidDigit)?;
            let mut i = 8;
            while n > 0 {
                res[i] = match n % 10 {
                    0 => Dist::Inf,
                    1 => Dist::Sqrt2,
                    2 => Dist::One,
                    _ => return Err(ParseNeighboorsError::InvalidDigit),
                };
                n /= 10;
                i -= 1;
            }
            Ok(Neighboors(res))
        }
    }
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "Огравировыватель",
    about = "Превращает изображение в подобие гравюры"
)]
struct Config {
    #[structopt(help = "Входной файл картинки в формате PNG")]
    infile: String,
    #[structopt(help = "Выходной файл картинки в формате PNG")]
    outfile: String,
    #[structopt(help = "Расстояния до соседей", short = "n", long = "heiboors", default_value = "121202121")]
    neiboors: Neighboors,
    #[structopt(help = "Яркость (от 0 до 255)", short = "a", long = "add", default_value = "127.0")]
    add: f64,
    #[structopt(help = "Множитель контрастности", short = "m", long = "mult", default_value = "0.5")]
    mult: f64,
    #[structopt(help = "Инвертировать цвета", short = "i", long = "invert")]
    inv: bool,
}

fn read_img(inpfile: &str) -> Matrix<(u8, u8, u8)> {
    let decoder = png::Decoder::new(
        std::fs::File::open(inpfile)
            .unwrap_or_else(|_| panic!("Не удалось открыть файл {:?}", inpfile)),
    );
    let (info, mut reader) = decoder.read_info().unwrap_or_else(|_| {
        panic!(
            "Ошибка чтения файла (возможно файл имеет неверный формат) {:?}",
            inpfile
        )
    });
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).unwrap_or_else(|_| {
        panic!(
            "Ошибка чтения файла (возможно файл имеет неверный формат) {:?}",
            inpfile
        )
    });

    if reader.info().bit_depth != png::BitDepth::Eight
        || reader.info().color_type != png::ColorType::RGB
    {
        panic!("Неверный цветовой формат файла {:?}\nПрограмма не поддерживает прозрачность и ЧБ изображения", inpfile)
    }

    let buf = {
        let mut new_buf = Vec::new();
        for i in 0..(buf.len() / 3) {
            new_buf.push((buf[3 * i], buf[3 * i + 1], buf[3 * i + 2]));
        }
        new_buf
    };

    Matrix::new(
        buf,
        reader.info().width as usize,
        reader.info().height as usize,
    )
}

fn make_diff(img: Matrix<(u8, u8, u8)>, conf: Config) -> Matrix<(u8, u8, u8)> {
    let mut res = Matrix::new(
        vec![(0, 0, 0); (img.width() - 2) * (img.height() - 2)],
        img.width() - 2,
        img.height() - 2,
    );

    let r = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 0),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];
    let r = conf
        .neiboors
        .0
        .iter()
        .zip(r.iter())
        .map(|(d, (dx, dy))| {
            (
                match d {
                    Dist::Inf => 0.0,
                    Dist::Sqrt2 => 1.0,
                    Dist::One => 2.0f64.sqrt(),
                },
                *dx,
                *dy,
            )
        })
        .collect::<Vec<_>>();

    for x in 1..(img.height() - 1) {
        for y in 1..(img.width() - 1) {
            res[x - 1][y - 1] = {
                let mut s = (0.0, 0.0, 0.0);
                for (m, dx, dy) in &r {
                    s.0 += *m
                        * img[x.overflowing_add(*dx as usize).0][y.overflowing_add(*dy as usize).0]
                            .0 as f64;
                    s.1 += *m
                        * img[x.overflowing_add(*dx as usize).0][y.overflowing_add(*dy as usize).0]
                            .1 as f64;
                    s.2 += *m
                        * img[x.overflowing_add(*dx as usize).0][y.overflowing_add(*dy as usize).0]
                            .2 as f64;
                }
                let ms: f64 = r.iter().map(|(m, _, _)| m).sum();
                s.0 /= ms;
                s.1 /= ms;
                s.2 /= ms;
                s.0 -= img[x][y].0 as f64;
                s.1 -= img[x][y].1 as f64;
                s.2 -= img[x][y].2 as f64;
                s.0 = if conf.inv {
                    255.0 + s.0 * conf.mult - conf.add
                } else {
                    -s.0 * conf.mult + conf.add
                };
                s.1 = if conf.inv {
                    255.0 + s.1 * conf.mult - conf.add
                } else {
                    -s.1 * conf.mult + conf.add
                };
                s.2 = if conf.inv {
                    255.0 + s.2 * conf.mult - conf.add
                } else {
                    -s.2 * conf.mult + conf.add
                };
                (s.0.round() as u8, s.1.round() as u8, s.2.round() as u8)
            };
        }
    }
    res
}

fn save_diff(outfile: &str, img: Matrix<(u8, u8, u8)>) {
    let mut encoder = png::Encoder::new(
        std::fs::File::create(outfile)
            .unwrap_or_else(|_| panic!("Не удалось открыть файл {:?}", outfile)),
        img.width() as u32,
        img.height() as u32,
    );
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap_or_else(|_| panic!("Ошибка записи в файл {:?}", outfile));
    let buf = {
        let mut buf = Vec::new();
        for i in 0..img.height() {
            for j in 0..img.width() {
                buf.push(img[i][j].0);
                buf.push(img[i][j].1);
                buf.push(img[i][j].2);
            }
        }
        buf
    };
    writer.write_image_data(&buf).unwrap_or_else(|_| panic!("Ошибка записи в файл {:?}", outfile));
}

fn main() {
    let config: Config = structopt::StructOpt::from_args();

    let img = read_img(&config.infile);
    let diff = make_diff(img, config.clone());
    save_diff(&config.outfile, diff)
}
