use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::thread;
//use std::time::SystemTime;
extern crate num_cpus;

#[derive(Clone, Copy, Debug)]
pub struct Resolution {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Domain {
    pub start: f64,
    pub end: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct MandelConfig {
    pub xdomain: Domain,
    pub ydomain: Domain,
    pub resolution: Resolution,
    pub threshold: f64,
    pub max_iters: usize,
}

impl Default for MandelConfig {
    fn default() -> Self {
        Self {
            xdomain: Domain {
                start: -2.5,
                end: 1.0,
            },
            ydomain: Domain {
                start: -1.0,
                end: 1.0,
            },
            resolution: Resolution { x: 1920, y: 1080 },
            threshold: 4.0,
            max_iters: 128,
        }
    }
}
impl MandelConfig {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }
}

/// Process one horizontal row of the domain
//
// This function process one of the rows as below:
//
//             <--- x res --->
//           / .... ---> ..... ^
//   thread0 | .... ---> ..... |
//           \ .... ---> ..... | y
//           / .... ---> ..... | res
//   thread1 | .... ---> ..... |
//           \ .... ---> ..... v

fn mandel_worker(
    iters_row: &mut Vec<usize>,
    y0: f64,
    xdomain: &Vec<f64>,
    xres: usize,
    max_iters: usize,
    threshold: f64,
) {
    for i in 0..xres - 1 {
        let x0 = xdomain[i];
        let mut x1 = 0.0;
        let mut y1 = 0.0;
        let mut c = 0;
        while x1 * x1 + y1 * y1 <= threshold && c < max_iters {
            let xtmp = x1 * x1 - y1 * y1 + x0;
            y1 = 2.0 * x1 * y1 + y0;
            x1 = xtmp;
            c += 1;
        }
        // Pushing instead of indexing, since the matrix is not
        // initialised with zeros, but rather just allocated by size.
        //iters_row[i] = c;
        iters_row.push(c);
    }
}

pub fn mandel(cfg: MandelConfig) -> Vec<Vec<usize>> {
    //let t0 = SystemTime::now();

    // The domain is chunked along y, meaning that each thread will
    // process along x - horizontally

    // fill the x- and y-domain vectors
    let mut xdomain = vec![];
    {
        let step = (cfg.xdomain.end - cfg.xdomain.start) / (cfg.resolution.x - 1) as f64;
        let start = cfg.xdomain.start;

        for i in 0..cfg.resolution.x {
            xdomain.push(start + step * i as f64)
        }
    }
    let xdomain = Arc::new(Vec::from_iter(xdomain));

    let mut ydomain = vec![];
    {
        let step = (cfg.ydomain.end - cfg.ydomain.start) / (cfg.resolution.y - 1) as f64;
        let start = cfg.ydomain.start;

        for i in 0..cfg.resolution.y {
            ydomain.push(start + step * i as f64)
        }
    }
    let ydomain = Arc::new(Vec::from_iter(ydomain));

    // Divide y-resolution to run in parallel
    let cpus = 4 * num_cpus::get();
    let delta_py = cfg.resolution.y / cpus;
    let remain_py = cfg.resolution.y % cpus;

    let mut chunks_py = vec![];
    for i in 0..cpus {
        chunks_py.push((i * delta_py, (i + 1) * delta_py));
    }
    {
        let idx = chunks_py.len() - 1;
        chunks_py[idx].1 += remain_py; // last thread gets a bit more work
    }
    let chunks_py = Vec::from_iter(chunks_py);

    // Matrix with number of Mandelbrot iterations:
    //
    //    iters[pixel_y][pixel_x]
    //
    // Must wrap each vector item in an `Arc<Mutex>` since the rows will
    // be updated in parallel by multiple threads. So the type of `iters`
    // is `Vec<Arc<Mutex<Vec<usize>>>>` since multiple threads
    //
    let mut iters = vec![];
    for _ in 0..cfg.resolution.y {
        // Here instead of initialising with zero, I'm just allocating
        // the capacity. Will need to change the workers too to `push`
        // instead of assining by indes.
        //let row = Arc::new(Mutex::new(vec![0; cfg.resolution.x]));
        let row = Arc::new(Mutex::new(Vec::with_capacity(cfg.resolution.x)));
        iters.push(row);
    }

    //let t1 = t0.elapsed().unwrap().as_millis();
    //println!("Initialised all arrays - eta {} ms", t1);

    // spawn of threads
    let mut handles = vec![];
    for (py_start, py_end) in chunks_py {
        let mut rows = vec![];
        for py in py_start..py_end {
            rows.push(Arc::clone(&iters[py]));
        }
        let ydomain = Arc::clone(&ydomain);
        let xdomain = Arc::clone(&xdomain);

        let hdl = thread::spawn(move || {
            for py in py_start..py_end {
                let mut row = rows[py - py_start].lock().unwrap();
                mandel_worker(
                    &mut row,
                    ydomain[py],
                    &xdomain,
                    cfg.resolution.x,
                    cfg.max_iters,
                    cfg.threshold,
                );
            }
        });
        handles.push(hdl);
    }

    for hdl in handles {
        hdl.join().unwrap();
    }

    //let t2 = t0.elapsed().unwrap().as_millis() - t1;
    //println!("All threads done - eta {} ms", t2);

    // converting here from:
    //     &Vec<Arc<Mutex<Vec<usize>>>>
    // to
    //     &Vec<Vec<usize>>
    //
    // https://stackoverflow.com/questions/78768409/fill-a-matrix-in-
    // parallel-how-to-convert-vecarcmutexvec-to-vecvec
    let mut ret = vec![];
    for row in iters {
        ret.push(Mutex::into_inner(Arc::into_inner(row).unwrap()).unwrap());
    }

    //let t3 = t0.elapsed().unwrap().as_millis() - t1 - t2;
    //println!("Conversion done - eta {} ms", t3);

    ret
}

pub fn save_image(iters: &Vec<Vec<usize>>, max_iters: usize) {
    let resy = iters.len() as u32;
    let resx = iters[0].len() as u32;

    let mut imgbuf = image::ImageBuffer::new(resx, resy);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let c = iters[y as usize][x as usize];
        let (mut r, mut g, mut b) = (0 as u8, 0 as u8, 0 as u8);
        if c < max_iters {
            let c = c as f32;
            r = (255.0 * c / max_iters as f32) as u8;
            g = 255 as u8;
            b = (255.0 * c / (c + 8.0)) as u8;
        }
        *pixel = image::Rgb([r, g, b]);
    }
    imgbuf.save("fractal.png").unwrap();
}
