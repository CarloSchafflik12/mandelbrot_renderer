use super::Complex;

pub fn mandelbrot(c_re: f64, c_im: f64, max_iterations: u16) -> u16 {
    let c = Complex::new(c_re, c_im);
    let mut z = Complex::new(0.0, 0.0);
    let mut iterations = max_iterations;
    for n in 1..=max_iterations {
        z = z.powi(2) + c;
        if ((z.re * z.re + z.im * z.im) as f64).sqrt() > 2.0 {
            iterations = n;
            break;
        }
    }
    iterations
}
