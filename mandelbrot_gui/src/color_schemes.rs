// Color schemes ////////////////////////////////////////////////////
//               ///////////////////////////////
// Color schemes must be implemented as structs that implement
// the `MandelRGB` trait, ie, they must have a function that
// take 2 `usize` parameters, `c` and `max_iters`, and return a
// 3-tuple of type `u8` with the RGB values of a color.
pub trait MandelRGB {
    fn rgb(&self, c: usize, max_iters: usize) -> (u8, u8, u8);
}

pub struct ColorSchemes {
    color_schemes: Vec<Box<dyn MandelRGB>>,
    index_current: usize,
}
impl ColorSchemes {
    pub fn new() -> Self {
        Self {
            color_schemes: vec![
                Box::new(Bluey {}),
                Box::new(Greeny {}),
                Box::new(Purply {}),
                Box::new(Weirdy {}),
                Box::new(GreyeyDark {}),
                Box::new(GreyeyLight {}),
		Box::new(Hulky {}),
		Box::new(Wiky {}),
		
            ],
            index_current: 0,
        }
    }
    pub fn get(&self) -> &Box<dyn MandelRGB> {
        &self.color_schemes[self.index_current]
    }
    pub fn next(&mut self) {
        if self.index_current == self.color_schemes.len() - 1 {
            self.index_current = 0;
        } else {
            self.index_current += 1;
        }
    }
}

struct Wiky {}
impl MandelRGB for Wiky {
    fn rgb(&self, c: usize, max_iters: usize) -> (u8, u8, u8) {
        if c < max_iters {
            let q = (c as f64) / (max_iters as f64);
	    if q < 0.16 { ( 0, 7, 100) }
	    else if q < 0.42 { (32, 107, 203) }
	    else if q < 0.64 { (237, 255, 255) }
	    else if q < 0.86 { (255, 170, 0) }
	    else { (0, 2, 0) }
        } else {
            (0, 0, 0)
        }
    }
}

struct Hulky {}
impl MandelRGB for Hulky {
    fn rgb(&self, c: usize, max_iters: usize) -> (u8, u8, u8) {
        if c < max_iters {
            let q = (c as f64) / (max_iters as f64);
	    if q > 0.5 {
		(
                    (255.0 * q) as u8,
                    255 as u8,
                    (255.0 * q) as u8,
		)
	    } else {
		(
		    0 as u8,
		    (255.0 * q) as u8,
		    0 as u8,
		)
	    }
		    
        } else {
            (0, 0, 0)
        }
    }
}

struct Bluey {}
impl MandelRGB for Bluey {
    fn rgb(&self, c: usize, max_iters: usize) -> (u8, u8, u8) {
        if c < max_iters {
            let c = c as f64;
            (
                (255.0 * c / max_iters as f64) as u8,
                (255.0 * c / (c + 8.0)) as u8,
                255 as u8,
            )
        } else {
            (0, 0, 0)
        }
    }
}
struct Greeny {}
impl MandelRGB for Greeny {
    fn rgb(&self, c: usize, max_iters: usize) -> (u8, u8, u8) {
        if c < max_iters {
            let c = c as f64;
            (
                (255.0 * c / max_iters as f64) as u8,
                255 as u8,
                (255.0 * c / (c + 8.0)) as u8,
            )
        } else {
            (0, 0, 0)
        }
    }
}
struct Purply {}
impl MandelRGB for Purply {
    fn rgb(&self, c: usize, max_iters: usize) -> (u8, u8, u8) {
        if c < max_iters {
            let c = c as f64;
            let m = max_iters as f64;
            (
                (255.0 * c / m) as u8,
                (255.0 * c / m) as u8,
                (255.0 * c / (c + 8.0)) as u8,
            )
        } else {
            (0, 0, 0)
        }
    }
}
struct Weirdy {}
impl MandelRGB for Weirdy {
    fn rgb(&self, c: usize, max_iters: usize) -> (u8, u8, u8) {
        if c < max_iters {
            let c = c as f64;
            let m = max_iters as f64;
            (
                (255.0 * (2.0 * c / m) - 1.0).abs() as u8,
                (255.0 * c / m) as u8,
                (255.0 * c / (c + 8.0)) as u8,
            )
        } else {
            (0, 0, 0)
        }
    }
}
struct GreyeyLight {}
impl MandelRGB for GreyeyLight {
    fn rgb(&self, c: usize, max_iters: usize) -> (u8, u8, u8) {
        if c < max_iters {
            let c = c as f64;
            let m = max_iters as f64;
            (
                (255.0 * (2.0 * c / m - 1.0).abs()) as u8,
                (255.0 * (2.0 * c / m - 1.0).abs()) as u8,
                (255.0 * (2.0 * c / m - 1.0).abs()) as u8,
            )
        } else {
            (255, 255, 255)
        }
    }
}
struct GreyeyDark {}
impl MandelRGB for GreyeyDark {
    fn rgb(&self, c: usize, max_iters: usize) -> (u8, u8, u8) {
        if c < max_iters {
            let c = c as f64;
            let m = max_iters as f64;
            (
                (255.0 * c / m) as u8,
                (255.0 * c / m) as u8,
                (255.0 * c / m) as u8,
            )
        } else {
            (0, 0, 0)
        }
    }
}
