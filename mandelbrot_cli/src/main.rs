use std::env;
use std::fmt::Debug;
use std::process;
use std::str::FromStr;
use std::time::SystemTime;

use mandelbrot_cli::{mandel, save_image, Domain, MandelConfig, Resolution};

fn help() {
    eprintln!("Use:");
    eprintln!(
        "  {} x0 x1 y0 y1 max_iters resx resy",
        env::args().collect::<Vec<_>>()[0]
    );
    eprintln!("Typical call:");
    eprintln!(
        "  {} -2.5 1.0 -1.0 1.0 128 1920 1080 ",
        env::args().collect::<Vec<_>>()[0]
    );
}

// Parse a string into a value, eg, `"2.5" => 2.5`.
// Exit process in case of parse error.
fn arg_parse<T: FromStr>(arg: &str, name: &str) -> T
where
    <T as FromStr>::Err: Debug,
{
    match arg.parse() {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Error parsing {name} - {e:?}, got \"{arg}\"");
            help();
            process::exit(1);
        }
    }
}

fn main() {
    let t0 = SystemTime::now();

    let args: Vec<_> = env::args().collect();

    let cfg: MandelConfig;

    if args.len() == 1 {
        println!("Using default values.");
        cfg = MandelConfig::new();
    } else if args.len() != 8 {
        eprintln!("Error: invalid number of arguments.");
        help();
        process::exit(1);
    } else {
        let x0 = arg_parse::<f64>(&args[1], "x0");
        let x1 = arg_parse::<f64>(&args[2], "x1");
        let y0 = arg_parse::<f64>(&args[3], "y0");
        let y1 = arg_parse::<f64>(&args[4], "y1");
        let max_iters = arg_parse::<usize>(&args[5], "max_iters");
        let resx = arg_parse::<usize>(&args[6], "resx");
        let resy = arg_parse::<usize>(&args[7], "resy");

        cfg = MandelConfig {
            xdomain: Domain { start: x0, end: x1 },
            ydomain: Domain { start: y0, end: y1 },
            resolution: Resolution { x: resx, y: resy },
            threshold: 4.0,
            max_iters: max_iters,
        };
    }
    println!("{:?}", cfg);

    let t1 = t0.elapsed().unwrap().as_millis();
    println!("==> arg parsing took {} ms", t1);

    let iters = mandel(cfg);

    let t2 = t0.elapsed().unwrap().as_millis() - t1;
    println!("==> `mandel()` took {} ms", t2);

    save_image(&iters, cfg.max_iters);

    let t3 = t0.elapsed().unwrap().as_millis() - t2 - t1;
    println!("==> `save_image()` took {} ms", t3);

    let t4 = t0.elapsed().unwrap().as_millis();
    println!("==> Overall took {} ms", t4);
}
