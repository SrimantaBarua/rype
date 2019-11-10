//! TrueType glyph data table
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use super::error::*;
use super::types::{get_i16, get_i16_unchecked, get_u16, get_u16_unchecked, get_u8, Affine};
use super::GlyphBitmap;
use rster::{PathOp, Point};

pub(super) struct Glyf<'a>(pub(super) &'a [u8]);

impl<'a> std::fmt::Debug for Glyf<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Glyf")
    }
}

impl<'a> Glyf<'a> {
    pub(super) fn glyph(&self, offset: usize) -> Result<TTGlyph<'a>> {
        if offset + 10 > self.0.len() {
            return Err(Error::Invalid);
        }
        let num_contours = get_i16_unchecked(self.0, offset + 0);
        let xmin = get_i16_unchecked(self.0, offset + 2);
        let ymin = get_i16_unchecked(self.0, offset + 4);
        let xmax = get_i16_unchecked(self.0, offset + 6);
        let ymax = get_i16_unchecked(self.0, offset + 8);
        if num_contours < 0 {
            Ok(TTGlyph::Composite(&self.0[offset..]))
        } else {
            Ok(TTGlyph::Simple(SimpleGlyph {
                num_contours: num_contours as u16,
                xmin: xmin,
                ymin: ymin,
                xmax: xmax,
                ymax: ymax,
                data: &self.0[offset + 10..],
            }))
        }
    }
}

pub(super) enum TTGlyph<'a> {
    Simple(SimpleGlyph<'a>),
    Composite(&'a [u8]),
}

#[derive(Debug)]
struct Points<'a> {
    points_remaining: usize,
    flags_remaining: usize,
    flag: u8,
    last_point: Point,
    flag_off: usize,
    x_off: usize,
    y_off: usize,
    affine: Affine,
    data: &'a [u8],
}

impl<'a> Iterator for Points<'a> {
    type Item = (bool, Point);

    fn next(&mut self) -> Option<(bool, Point)> {
        if self.points_remaining == 0 {
            None
        } else {
            if self.flags_remaining > 0 {
                self.flags_remaining -= 1;
            } else {
                self.flag = self.data[self.flag_off];
                self.flag_off += 1;
                if self.flag & 0x08 != 0 {
                    self.flags_remaining = self.data[self.flag_off] as usize;
                    self.flag_off += 1;
                }
            }
            match self.flag & 0x12 {
                0x00 => {
                    self.last_point.x += get_i16(self.data, self.x_off).ok()? as f32;
                    self.x_off += 2;
                }
                0x02 => {
                    self.last_point.x -= get_u8(self.data, self.x_off).ok()? as f32;
                    self.x_off += 1;
                }
                0x12 => {
                    self.last_point.x += get_u8(self.data, self.x_off).ok()? as f32;
                    self.x_off += 1;
                }
                _ => (),
            }
            match self.flag & 0x24 {
                0x00 => {
                    self.last_point.y += get_i16(self.data, self.y_off).ok()? as f32;
                    self.y_off += 2;
                }
                0x04 => {
                    self.last_point.y -= get_u8(self.data, self.y_off).ok()? as f32;
                    self.y_off += 1;
                }
                0x24 => {
                    self.last_point.y += get_u8(self.data, self.y_off).ok()? as f32;
                    self.y_off += 1;
                }
                _ => (),
            }
            self.points_remaining -= 1;
            let new_pt = self.affine.apply_point(&self.last_point);
            Some((
                self.flag & 0x01 > 0,
                self.affine.apply_point(&self.last_point),
            ))
        }
    }
}

#[derive(Debug)]
struct ContourSizes<'a> {
    last: isize,
    data: &'a [u8],
}

impl<'a> Iterator for ContourSizes<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.data.len() == 0 {
            None
        } else {
            let cur = get_u16_unchecked(self.data, 0) as isize;
            let diff = cur - self.last;
            self.data = &self.data[2..];
            self.last = cur;
            Some(diff as usize)
        }
    }
}

#[derive(Debug)]
struct PathIter<'a> {
    contour_start: Option<Point>,
    last_offcurve: Option<Point>,
    points: Points<'a>,
    cur_contour_size: usize,
    contour_sizes: ContourSizes<'a>,
}

impl<'a> Iterator for PathIter<'a> {
    type Item = PathOp;

    fn next(&mut self) -> Option<PathOp> {
        if self.contour_start.is_none() {
            self.cur_contour_size = self.contour_sizes.next()? - 1;
            self.contour_start = match self.points.next() {
                Some((true, p)) => Some(p),
                // TODO: These are errors
                _ => None,
            };
            self.contour_start.map(|p| PathOp::Move(p))
        } else {
            match self.last_offcurve {
                None => {
                    if self.cur_contour_size > 0 {
                        let (on_curve, p0) = self.points.next()?;
                        self.cur_contour_size -= 1;
                        if on_curve {
                            Some(PathOp::Line(p0))
                        } else if self.cur_contour_size > 0 {
                            let (on_curve, p1) = self.points.next()?;
                            self.cur_contour_size -= 1;
                            if on_curve {
                                Some(PathOp::QuadBez(p0, p1))
                            } else {
                                let pmid = Point::linterp(0.5, p0, p1);
                                self.last_offcurve = Some(p1);
                                Some(PathOp::QuadBez(p0, pmid))
                            }
                        } else {
                            let ret = Some(PathOp::QuadBez(p0, self.contour_start.unwrap()));
                            self.contour_start = None;
                            ret
                        }
                    } else {
                        let ret = Some(PathOp::Line(self.contour_start.unwrap()));
                        self.contour_start = None;
                        ret
                    }
                }
                Some(p0) => {
                    if self.cur_contour_size > 0 {
                        let (on_curve, p1) = self.points.next()?;
                        self.cur_contour_size -= 1;
                        if on_curve {
                            self.last_offcurve = None;
                            Some(PathOp::QuadBez(p0, p1))
                        } else {
                            let pmid = Point::linterp(0.5, p0, p1);
                            self.last_offcurve = Some(p1);
                            Some(PathOp::QuadBez(p0, pmid))
                        }
                    } else {
                        let ret = Some(PathOp::QuadBez(p0, self.contour_start.unwrap()));
                        self.last_offcurve = None;
                        self.contour_start = None;
                        ret
                    }
                }
            }
        }
    }
}

pub(super) struct SimpleGlyph<'a> {
    num_contours: u16,
    xmin: i16,
    ymin: i16,
    xmax: i16,
    ymax: i16,
    data: &'a [u8],
}

impl<'a> TTGlyph<'a> {
    /// Draw glyph with given scaling
    pub(super) fn render(&self, scale_x: f32, scale_y: f32) -> Result<GlyphBitmap> {
        match self {
            TTGlyph::Simple(ref s) => {
                // Get offsets for flags, x, y
                let num_contours = s.num_contours as usize;
                let num_points = get_u16(s.data, (num_contours - 1) * 2)? as usize + 1;
                let num_insn = get_u16(s.data, num_contours * 2)? as usize;
                let flag_off = num_contours * 2 + 2 + num_insn;
                let (x_off, y_off) = get_ttglyph_offsets(s.data, num_points, flag_off)?;

                // Prepare for rendering glyph
                let width = ((s.xmax - s.xmin) as f32 * scale_x).ceil() as usize + 2;
                let height = ((s.ymax - s.ymin) as f32 * scale_y).ceil() as usize + 2;
                let affine =
                    Affine::translation(-s.xmin as f32, -s.ymax as f32).scaled(scale_x, -scale_y);
                let mut rster = rster::Rster::new(width, height);

                // Get iterator over contours
                let contour_sizes = ContourSizes {
                    last: -1,
                    data: &s.data[..(num_contours * 2)],
                };
                // Get iterator over points
                let points = Points {
                    points_remaining: num_points,
                    flags_remaining: 0,
                    flag: 0,
                    last_point: Point::new(0.0, 0.0),
                    flag_off: flag_off,
                    x_off: x_off,
                    y_off: y_off,
                    affine: affine.clone(),
                    data: s.data,
                };
                // Get iterator over path
                let path_iter = PathIter {
                    contour_start: None,
                    last_offcurve: None,
                    points: points,
                    cur_contour_size: 0,
                    contour_sizes: contour_sizes,
                };
                // Draw path
                rster.draw_path(path_iter);
                Ok(GlyphBitmap {
                    width: width,
                    height: height,
                    data: rster.accumulate(),
                })
            }
            TTGlyph::Composite(ref data) => {
                Err(Error::Unimplemented("composite glyphs".to_owned()))
            }
        }
    }
}

fn get_ttglyph_offsets(
    data: &[u8],
    mut points_remaining: usize,
    flags_off: usize,
) -> Result<(usize, usize)> {
    let mut flags_size = 0;
    let mut x_size = 0;
    while points_remaining > 0 {
        let flag = get_u8(data, flags_off + flags_size)?;
        let repeat_count = if flag & 0x08 != 0 {
            flags_size += 1;
            get_u8(data, flags_off + flags_size)? as usize + 1
        } else {
            1
        };
        flags_size += 1;
        match flag & 0x12 {
            0x00 => x_size += repeat_count * 2,
            0x02 | 0x12 => x_size += repeat_count,
            _ => (),
        }
        points_remaining -= repeat_count;
    }
    let x_off = flags_off + flags_size;
    let y_off = x_off + x_size;
    Ok((x_off, y_off))
}

impl<'a> std::fmt::Debug for TTGlyph<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TTGlyph::Simple(_) => write!(f, "TTGlyph::Simple"),
            TTGlyph::Composite(_) => write!(f, "TTGlyph::Composite"),
        }
    }
}
