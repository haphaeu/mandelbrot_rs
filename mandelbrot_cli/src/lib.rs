use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;
//use std::time::SystemTime;
extern crate num_cpus;

pub mod color_schemes;
use color_schemes::ColorSchemes;

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
    let pool = ThreadPool::new(cpus);

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

	// sends jobs to the threadpool. each job processes one row
	for py in 0..cfg.resolution.y {
		
	    let ydomain = Arc::clone(&ydomain);
            let xdomain = Arc::clone(&xdomain);
	    let row = Arc::clone(&iters[py]);
		
	    pool.execute(move || {
		mandel_worker(
		    &mut row.lock().unwrap(),
		    ydomain[py],
		    &xdomain,
		    cfg.resolution.x,
		    cfg.max_iters,
		    cfg.threshold,
		);
	    });
	}
    pool.join();

    //let t2 = t0.elapsed().unwrap().as_millis() - t1;
    //println!("All threads done - et {t2} ms");

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

/// Return a buffer with the image of the mandelbrot set
pub fn get_image_buf(
    iters: &Vec<Vec<usize>>,
    max_iters: usize,
    color_schemes: ColorSchemes,
) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    let resy = iters.len() as u32;
    let resx = iters[0].len() as u32;

    let mut imgbuf = image::ImageBuffer::new(resx, resy);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        // imgbuf is indexed top-left to bottom-right,
        // hence the y-index must be reversed:
        let c = iters[(resy - y - 1) as usize][x as usize];
        let (r, g, b) = color_schemes.get().rgb(c, max_iters);
        *pixel = image::Rgb([r, g, b]);
    }
    imgbuf
}
