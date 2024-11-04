#[derive(Copy, Clone)]
pub struct Coord {
    center_px: Pixel,
    x_per_px: f64,
    y_per_px: f64,
}

impl Coord {
    pub fn new(center_px: Pixel, x_max: f64, y_max: f64, width: u32) -> Coord {
        let x_per_px = x_max / (width as f64 - center_px.x as f64);
        let y_per_px = y_max / center_px.y as f64;
        Coord {
            center_px,
            x_per_px,
            y_per_px,
        }
    }

    pub fn px2cartesian(&self, px: Pixel) -> (f64, f64) {
        let cartesian_px_x = px.x as i32 - self.center_px.x as i32;
        let cartesian_px_y = -(px.y as i32 - self.center_px.y as i32);
        let x = cartesian_px_x as f64 * self.x_per_px;
        let y = cartesian_px_y as f64 * self.y_per_px;
        (x, y)
    }
}

#[derive(Copy, Clone)]
pub struct Pixel {
    pub x: u32,
    pub y: u32,
}

impl Pixel {
    pub fn new(x: u32, y: u32) -> Pixel {
        Pixel { x, y }
    }
}
