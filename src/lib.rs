use std::{
    fmt::Display,
    ops::{Index, IndexMut},
    str::FromStr,
};

#[derive(Debug, Clone, Copy)]
enum Dist {
    One,
    Sqrt2,
    Inf,
}

#[derive(Debug, Clone)]
pub struct Neighboors([Dist; 9]);

#[derive(Debug)]
pub enum ParseNeighboorsError {
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

pub struct Matrix<T> {
    data: Vec<T>,
    width: usize,
    height: usize,
}

pub struct DiffConf {
    pub neiboors: Neighboors,
    pub add: f64,
    pub mult: f64,
    pub inv: bool,
    pub gray: bool,
}

use rayon::prelude::*;

impl<T> Matrix<T> {
    pub fn new(data: Vec<T>, width: usize, height: usize) -> Matrix<T> {
        assert!(data.len() == width * height);
        Matrix {
            data,
            width,
            height,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data[..]
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

pub fn make_grave(img: Matrix<(u8, u8, u8)>, conf: DiffConf) -> Matrix<(u8, u8, u8)> {
    let r: [(i32, i32); 9] = [
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

    let v = (0..img.height())
        .into_par_iter()
        .flat_map(|x| {
            (0..img.width())
                .map(|y| {
                    let mut s = (0.0, 0.0, 0.0);
                    let mut ms = 0.0;
                    for (m, dx, dy) in &r {
                        let mx = x.overflowing_add(*dx as usize).0;
                        let my = y.overflowing_add(*dy as usize).0;
                        if mx < img.height() && my < img.width() {
                            s.0 += *m * img[mx][my].0 as f64;
                            s.1 += *m * img[mx][my].1 as f64;
                            s.2 += *m * img[mx][my].2 as f64;
                            ms += *m;
                        }
                    }
                    if ms != 0.0 {
                        s.0 /= ms;
                        s.1 /= ms;
                        s.2 /= ms;
                    }
                    s.0 -= img[x][y].0 as f64;
                    s.1 -= img[x][y].1 as f64;
                    s.2 -= img[x][y].2 as f64;
                    s.0 = -s.0 * conf.mult + conf.add;
                    s.1 = -s.1 * conf.mult + conf.add;
                    s.2 = -s.2 * conf.mult + conf.add;
                    if conf.inv {
                        s.0 = 255.0 - s.0;
                        s.1 = 255.0 - s.1;
                        s.2 = 255.0 - s.2;
                    }
                    if conf.gray {
                        let s = (
                            s.0.round() as u64 as f64,
                            s.1.round() as u64 as f64,
                            s.2.round() as u64 as f64 / 2.0,
                        );
                        let abs = (s.0.powi(2) + s.1.powi(2) + s.2.powi(2)).sqrt();
                        let bright = (abs.round() / 1.5) as u8;
                        (bright, bright, bright)
                    } else {
                        (s.0.round() as u8, s.1.round() as u8, s.2.round() as u8)
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    Matrix::new(v, img.width(), img.height())
}
