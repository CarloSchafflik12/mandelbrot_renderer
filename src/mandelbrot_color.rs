use super::Rgb;

struct Hsv {
    h: f64,
    s: f64,
    v: f64,
}

impl Hsv {
    fn new(h: f64, s: f64, v: f64) -> Hsv {
        Hsv { h, s, v }
    }

    fn to_rgb(&self) -> Rgb<u8> {
        let h_i = (self.h / 60.0).floor();
        let f = (self.h / 60.0) - h_i;
        let p = self.v * (1.0 - self.s);
        let q = self.v * (1.0 - self.s * f);
        let t = self.v * (1.0 - self.s * (1.0 - f));

        let rgb;
        match h_i as u8 {
            0 | 6 => rgb = (self.v, t, p),
            1 => rgb = (q, self.v, p),
            2 => rgb = (p, self.v, t),
            3 => rgb = (p, q, self.v),
            4 => rgb = (t, p, self.v),
            5 => rgb = (self.v, p, q),
            _ => rgb = (1.0, 1.0, 1.0),
        }
        let r = (rgb.0 * 255.0) as u8;
        let g = (rgb.1 * 255.0) as u8;
        let b = (rgb.2 * 255.0) as u8;
        Rgb([r, g, b])
    }
}

pub fn get_rgb(iterations: u32) -> Rgb<u8> {
    let hsv_hue = iterations % 360;
    let hsv = Hsv::new(hsv_hue as f64, 1.0, 1.0);
    hsv.to_rgb()
}
