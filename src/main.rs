use clap::Parser;
use mandelbrot_generator::Config;

fn main() {
    let config = Config::parse();

    mandelbrot_generator::run(&config);
}
