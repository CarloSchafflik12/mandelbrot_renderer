use image::{Rgb, RgbImage};

pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub stroke: Rgb<u8>,
    pub img_buffer: RgbImage,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Canvas {
        Canvas {
            width,
            height,
            stroke: Rgb([255, 255, 255]),
            img_buffer: RgbImage::new(width, height),
        }
    }

    pub fn save(&self, path: &str) {
        self.img_buffer.save(path).unwrap();
    }

    pub fn line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32) {
        if (y2 as i32 - y1 as i32).abs() < (x2 as i32 - x1 as i32).abs() {
            if x1 > x2 {
                self.line_low(x2, y2, x1, y1);
            } else {
                self.line_low(x1, y1, x2, y2);
            }
        } else {
            if y1 > y2 {
                self.line_high(x2, y2, x1, y1);
            } else {
                self.line_high(x1, y1, x2, y2);
            }
        }
    }

    fn line_low(&mut self, x1: u32, y1: u32, x2: u32, y2: u32) {
        let dx = x2 as i32 - x1 as i32;
        let mut dy = y2 as i32 - y1 as i32;

        let mut yi = 1;
        if dy < 0 {
            yi = -1;
            dy = -dy;
        }

        let mut sel = 2 * dy - dx;
        let mut y = y1;

        for x in x1..=x2 {
            if self.in_bounds(x, y) {
                self.img_buffer.put_pixel(x, y, self.stroke);
            }
            if sel > 0 {
                y = (y as i32 + yi) as u32;
                sel += 2 * (dy - dx);
            } else {
                sel += 2 * dy;
            }
        }
    }

    fn line_high(&mut self, x1: u32, y1: u32, x2: u32, y2: u32) {
        let mut dx = x2 as i32 - x1 as i32;
        let dy = y2 as i32 - y1 as i32;

        let mut xi = 1;
        if dx < 0 {
            xi = -1;
            dx = -dx;
        }

        let mut sel = 2 * dx - dy;
        let mut x = x1;

        for y in y1..=y2 {
            if self.in_bounds(x, y) {
                self.img_buffer.put_pixel(x, y, self.stroke);
            }
            if sel > 0 {
                x = (x as i32 + xi) as u32;
                sel += 2 * (dx - dy);
            } else {
                sel += 2 * dx;
            }
        }
    }

    fn in_bounds(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }
}
