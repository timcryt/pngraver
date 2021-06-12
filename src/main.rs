use pngraver::*;

#[macro_use]
extern crate structopt;

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
    #[structopt(
        help = "Расстояния до соседей",
        short = "n",
        long = "neiboors",
        default_value = "121202121"
    )]
    neiboors: Neighboors,
    #[structopt(
        help = "Яркость (от 0 до 255)",
        short = "a",
        long = "add",
        default_value = "127.0"
    )]
    add: f64,
    #[structopt(
        help = "Множитель контрастности",
        short = "m",
        long = "mult",
        default_value = "0.5"
    )]
    mult: f64,
    #[structopt(help = "Инвертировать цвета", short = "i", long = "invert")]
    inv: bool,
    #[structopt(help = "Убрать цвета", short = "g", long = "gray")]
    gray: bool,
}

impl From<Config> for DiffConf {
    fn from(conf: Config) -> Self {
        DiffConf {
            neiboors: conf.neiboors,
            add: conf.add,
            mult: conf.mult,
            inv: conf.inv,
            gray: conf.gray,
        }
    }
}

fn read_img(inpfile: &str) -> Matrix<(u8, u8, u8)> {
    let decoder = image::open(inpfile)
        .unwrap_or_else(|e| panic!("Не удалось открыть файл.\nПричина: '{}'", e))
        .to_rgb8();

    let (w, h) = decoder.dimensions();

    let buf = {
        let mut buf = Vec::new();
        let t = decoder.to_vec();
        for i in 0..(decoder.len() / 3) {
            buf.push((t[i * 3], t[i * 3 + 1], t[i * 3 + 2]));
        }
        buf
    };

    Matrix::new(buf, w as usize, h as usize)
}

fn save_diff(outfile: &str, img: Matrix<(u8, u8, u8)>) {
    image::save_buffer(
        outfile,
        &img.as_slice()
            .into_iter()
            .flat_map(|x| vec![x.0, x.1, x.2])
            .collect::<Vec<_>>()[..],
        img.width() as u32,
        img.height() as u32,
        image::ColorType::Rgb8,
    )
    .unwrap_or_else(|e| panic!("Не удалось сохранить результат.\nПричиниа: '{}'", e))
}

fn main() {
    let config: Config = structopt::StructOpt::from_args();

    let img = read_img(&config.infile);
    let diff = make_grave(img, config.clone().into());
    save_diff(&config.outfile, diff)
}
