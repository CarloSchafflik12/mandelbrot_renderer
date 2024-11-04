pub mod canvas;
pub mod coord;
pub mod mandelbrot_color;
pub mod mandelbrot_kernel;

use canvas::Canvas;
use clap::{Parser, ValueEnum};
use coord::{Coord, Pixel};
use image::Rgb;
use indicatif::ProgressBar;
use num_complex::Complex;

use std::sync::{mpsc, Arc, Mutex};
use std::thread;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Render mode
    #[arg(value_enum)]
    mode: Mode,

    /// Path of output image
    #[arg(long, default_value_t = String::from("out.png"))]
    path: String,

    /// Number of max iterations per pixel
    #[arg(short, long, default_value_t = 100)]
    iterations_max: u16,

    /// Image resolution
    #[arg(short, long, default_value_t = 2048)]
    res: u32,

    /// Color frequency if used in colored render mode
    #[arg(long, default_value_t = 1)]
    color_frequency: u32,

    /// Color offset if used in colored render mode
    #[arg(long, default_value_t = 0)]
    color_offset: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Binary,
    Colored,
}

pub fn run(config: &Config) {
    match config.mode {
        Mode::Binary => run_binary(config),
        Mode::Colored => run_colored(config),
    }
}

fn run_binary(config: &Config) {
    let w = config.res;
    let h = config.res;

    let mut canvas = Canvas::new(w, h);
    let coord = Coord::new(Pixel::new(w / 2, h / 2), 1.5, 1.5, w);

    println!("Rendering columns ...");
    let bar = ProgressBar::new(w as u64);
    let max_iter = config.iterations_max;
    for x in 0..w {
        for y in 0..h {
            let p = coord.px2cartesian(Pixel::new(x, y));
            let iterations = mandelbrot_kernel::mandelbrot(p.0 - 0.75, p.1, max_iter);
            if iterations == max_iter {
                canvas.img_buffer.put_pixel(x, y, Rgb([0, 0, 0]));
            } else {
                canvas.img_buffer.put_pixel(x, y, Rgb([255, 255, 255]));
            }
        }
        bar.inc(1);
    }
    bar.finish();

    canvas.save(config.path.as_str());
}

const THREADS: usize = 100;
const COL_PER_THREADS: usize = 500; // res / threads

fn run_colored(config: &Config) {
    let w = config.res;
    let h = config.res;

    let mut canvas = Canvas::new(w, h);
    let coord = Coord::new(Pixel::new(w / 2, h / 2), 1.5, 1.5, w);

    let sh_buffer = Arc::new(Mutex::new(vec![0u16; (w * h) as usize]));
    let max_iter = config.iterations_max;
    let (tx, rx) = mpsc::channel();

    let mut handles = Vec::<thread::JoinHandle<()>>::with_capacity(THREADS);
    for thr_index in 0..THREADS {
        let sh_buffer = sh_buffer.clone();
        let tx = tx.clone();
        let handle = thread::spawn(move || {
            let mut lc_buffer = Vec::<u16>::with_capacity(COL_PER_THREADS * w as usize);
            for x in (COL_PER_THREADS * thr_index)..(COL_PER_THREADS * (thr_index + 1)) {
                for y in 0..h {
                    let p = coord.px2cartesian(Pixel::new(x as u32, y as u32));
                    let iterations = mandelbrot_kernel::mandelbrot(p.0 - 0.75, p.1, max_iter);
                    lc_buffer.push(iterations);
                }
                tx.send(1).unwrap();
            }
            let offset = COL_PER_THREADS * w as usize * thr_index;
            let mut gl_buffer = sh_buffer.lock().unwrap();
            for n in 0..(COL_PER_THREADS * w as usize) {
                gl_buffer[n + offset] = lc_buffer[n];
            }
        });
        handles.push(handle);
    }

    println!("Calculating columns ...");
    let bar = ProgressBar::new(w as u64);
    for _ in rx.iter().take(w as usize) {
        bar.inc(1);
    }
    bar.finish();

    for h in handles.into_iter() {
        h.join().unwrap();
    }

    println!("Generating image ...");
    let bar = ProgressBar::new(w as u64);
    let buffer = sh_buffer.lock().unwrap();
    for x in 0..w {
        for y in 0..h {
            let iterations = buffer[(x * h + y) as usize];
            if iterations == max_iter {
                canvas.img_buffer.put_pixel(x, y, Rgb([0, 0, 0]));
            } else {
                canvas.img_buffer.put_pixel(
                    x,
                    y,
                    mandelbrot_color::get_rgb(
                        iterations as u32 * config.color_frequency + config.color_offset,
                    ),
                );
            }
        }
        bar.inc(1);
    }
    bar.finish();

    println!("Writing image to disk ...");
    canvas.save(config.path.as_str());

    println!("Done!\n");

    /*
    println!("Rendering columns ...");
    let bar = ProgressBar::new(w as u64);
    let max_iter = config.iterations_max;
    for x in 0..w {
        for y in 0..h {
            let p = coord.px2cartesian(Pixel::new(x, y));
            let iterations = mandelbrot_kernel::mandelbrot(p.0 - 0.75, p.1, max_iter);
            if iterations == max_iter {
                canvas.img_buffer.put_pixel(x, y, Rgb([0, 0, 0]));
            } else {
                canvas.img_buffer.put_pixel(
                    x,
                    y,
                    mandelbrot_color::get_rgb(
                        iterations * config.color_frequency + config.color_offset,
                    ),
                );
            }
        }
        bar.inc(1);
    }
    bar.finish();

    canvas.save(config.path.as_str());
    */
}
