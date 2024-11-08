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
#[structopt(allow_negative_numbers = true)]
pub struct Config {
    /// Render mode
    #[arg(value_enum)]
    mode: Mode,

    /// Number of threads (0 -> auto)
    #[arg(short, long, value_parser = threads_parser, default_value_t = 0)]
    threads: usize,

    /// Path of output image
    #[arg(short, long, default_value_t = String::from("out.png"))]
    path: String,

    /// Number of max iterations per pixel
    #[arg(short, long, default_value_t = 100)]
    iterations_max: u16,

    /// Image resolution
    #[arg(short, long, default_value_t = 2048)]
    res: u32,

    /// Color frequency if used in colored render mode
    #[arg(short = 'f', long, default_value_t = 1)]
    color_frequency: u32,

    /// Color offset if used in colored render mode
    #[arg(short = 'o', long, default_value_t = 0)]
    color_offset: u32,

    /// Center real coordinate
    #[arg(short = 'R', long, default_value_t = -0.75)]
    center_re: f64,

    /// Center imaginary coordinate
    #[arg(short = 'I', long, default_value_t = 0.0)]
    center_im: f64,

    /// Render zoom scale
    #[arg(short, long, default_value_t = 0.67)]
    zoom: f64,
}

fn threads_parser(s: &str) -> Result<usize, String> {
    let threads: usize = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a unsigned number"))?;
    Ok(threads)
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Binary,
    Colored,
}

pub fn run(config: &Config) {
    let w = config.res;
    let h = config.res;

    let scale_xy = 1.0 / config.zoom;

    let mut canvas = Canvas::new(w, h);
    let coord = Coord::new(Pixel::new(w / 2, h / 2), scale_xy, scale_xy, w);

    let sh_buffer = Arc::new(Mutex::new(vec![0u16; w as usize * h as usize]));
    let max_iter = config.iterations_max;
    let offset_re = config.center_re;
    let offset_im = config.center_im;
    let threads = match config.threads {
        0 => get_auto_threads(),
        n => n,
    };
    let (tx, rx) = mpsc::channel();

    let mut handles = Vec::<thread::JoinHandle<()>>::with_capacity(threads);
    for thr_index in 0..threads {
        let sh_buffer = sh_buffer.clone();
        let tx = tx.clone();
        let handle = thread::spawn(move || {
            let mut lc_buffer = vec![0u16; h as usize];
            for x in (thr_index..w as usize).step_by(threads) {
                for y in 0..h {
                    let p = coord.px2cartesian(Pixel::new(x as u32, y as u32));
                    let iterations =
                        mandelbrot_kernel::mandelbrot(p.0 + offset_re, p.1 + offset_im, max_iter);
                    lc_buffer[y as usize] = iterations;
                }
                let mut gl_buffer = sh_buffer.lock().unwrap();
                for n in 0..h {
                    gl_buffer[x as usize * h as usize + n as usize] = lc_buffer[n as usize];
                }
                tx.send(1).unwrap();
            }
        });
        handles.push(handle);
    }

    let sty = indicatif::ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {percent}%",
    )
    .unwrap()
    .progress_chars("=> ");

    println!("\n[1/2] Calculating columns with {threads} threads ...");
    let bar = ProgressBar::new(w as u64);
    bar.set_style(sty.clone());
    bar.enable_steady_tick(std::time::Duration::from_millis(100));
    for _ in rx.iter().take(w as usize) {
        bar.inc(1);
    }
    bar.finish();

    println!("[2/2] Generating image ...");
    let bar = ProgressBar::new(w as u64);
    bar.set_style(sty.clone());
    bar.enable_steady_tick(std::time::Duration::from_millis(100));
    let buffer = sh_buffer.lock().unwrap();
    for x in 0..w {
        for y in 0..h {
            let iterations = buffer[(x * h + y) as usize];
            if iterations == max_iter {
                canvas.img_buffer.put_pixel(x, y, Rgb([0, 0, 0]));
            } else {
                if config.mode == Mode::Binary {
                    canvas.img_buffer.put_pixel(x, y, Rgb([255, 255, 255]));
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
        }
        bar.inc(1);
    }
    bar.finish();

    println!("Writing image to disk ...");
    canvas.save(config.path.as_str());

    println!("Done!\n");
}

fn get_auto_threads() -> usize {
    let threads: usize = match thread::available_parallelism() {
        Ok(n) => n.get(),
        Err(_) => 1,
    };
    threads
}
