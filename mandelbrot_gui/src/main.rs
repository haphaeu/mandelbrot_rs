use nannou::image;
use nannou::prelude::{
    geom, wgpu, App, Frame, Key, KeyPressed, MouseMoved, MousePressed, MouseReleased,
    MouseScrollDelta::LineDelta, MouseScrollDelta::PixelDelta, MouseWheel, Resized, Vec2,
    WindowEvent, WindowId, CORNFLOWERBLUE, RED,
};
use nannou::winit::dpi::PhysicalPosition;

use mandelbrot_cli::{mandel, MandelConfig};

fn main() {
    nannou::app(model).run();
}

struct Model {
    // Store the window ID so we can refer to this specific window later if needed.
    window: WindowId,
    texture: wgpu::Texture,
    cfg: MandelConfig,
    pan_mode: PanMode,
    color_schemes: ColorSchemes,
    float_format_precision: usize,
}

struct ColorSchemes {
	color_schemes: Vec<Box<dyn MandelRGB>>,
	index_current: usize,
}
impl ColorSchemes {
	fn new() -> Self {
		Self {
			color_schemes: vec![
				Box::new(Bluey {}),
				Box::new(Greeny {}),
				Box::new(Purply {}),
				Box::new(Weirdy {}),
			],
			index_current: 0,
		}
	}
	fn get(&self) -> &Box<dyn MandelRGB> {
		&self.color_schemes[self.index_current]
	}
	fn next(&mut self) {
		if self.index_current == self.color_schemes.len() - 1 {
			self.index_current = 0;
		} else {
			self.index_current += 1;
		}
	}
}
// Color schemes ////////////////////////////////////////////////////
//               ///////////////////////////////
// Color schemes must be implemented as structs that implement
// the `MandelRGB` trait, ie, they must have a function that
// take 2 `usize` parameters, `c` and `max_iters`, and return a
// 3-tuple of type `u8` with the RGB values of a color.
trait MandelRGB {
    fn rgb(&self, c: usize, max_iters: usize) -> (u8, u8, u8);
}

struct Bluey {}
impl MandelRGB for Bluey {
    fn rgb(&self, c: usize, max_iters: usize) -> (u8, u8, u8) {
        if c < max_iters {
			let c = c as f64;
			( (255.0 * c / max_iters as f64) as u8,
              (255.0 * c / (c + 8.0)) as u8,
              255 as u8 )
		} else { (0, 0, 0) }
    }
}
struct Greeny {}
impl MandelRGB for Greeny {
    fn rgb(&self, c: usize, max_iters: usize) -> (u8, u8, u8) {
		if c < max_iters {
			let c = c as f64;
			( (255.0 * c / max_iters as f64) as u8,
              255 as u8,
              (255.0 * c / (c + 8.0)) as u8 )
		} else { (0, 0, 0) }
    }
}
struct Purply {}
impl MandelRGB for Purply {
    fn rgb(&self, c: usize, max_iters: usize) -> (u8, u8, u8) {
		if c < max_iters {
			let c = c as f64;
			let m = max_iters as f64;
			( (255.0 * c / m) as u8,
              (255.0 * c / m) as u8,
              (255.0 * c / (c + 8.0)) as u8 )
		} else { (0, 0, 0) }
    }
}
struct Weirdy {}
impl MandelRGB for Weirdy {
    fn rgb(&self, c: usize, max_iters: usize) -> (u8, u8, u8) {
		if c < max_iters {
			let c = c as f64;
			let m = max_iters as f64;
			( (255.0 * (2.0 * c / m) - 1.0).abs() as u8,
              (255.0 * c / m) as u8,
              (255.0 * c / (c + 8.0)) as u8 )
		} else { (0, 0, 0) }
    }
}

/// Pan image by dragging the mouse,
struct PanMode {
    is_active: bool,
    start: Vec2,
    end: Vec2,
    draw: Vec2,
}
impl Default for PanMode {
    fn default() -> Self {
        Self {
            is_active: false,
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(0.0, 0.0),
            draw: Vec2::new(0.0, 0.0),
        }
    }
}

// //////////////////////////////////////////////////////////////////

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    let window = app
        .new_window()
        .size(800, 450)
        .title("Mandelbrot Set")
        .view(view)
        .event(event)
        .build()
        .unwrap();

    let texture = wgpu::TextureBuilder::new()
        .size([800, 450])
        .format(wgpu::TextureFormat::Rgba8Unorm)
        .build(app.window(window).unwrap().device());

    let cfg = MandelConfig::default();
    let pan_mode = PanMode::default();
    let color_schemes = ColorSchemes::new();
	let color_scheme_idx = 0;
    let float_format_precision = 3;

    Model {
        window,
        texture,
        cfg,
        pan_mode,
        color_schemes,
        float_format_precision,
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(CORNFLOWERBLUE);
    let draw = app.draw();

    // Draw the image
    draw.texture(&model.texture).xy(model.pan_mode.draw);

    // Write some text
    let [x, y] = mouse2domain(app, model, model.pan_mode.end);
    let p = model.float_format_precision;
    let text = format!(
        "x ({:.p$}, {:.p$}), y ({:.p$}, {:.p$}), y\nMouse @ {:.p$}, {:.p$}\nMax iters: {}",
        model.cfg.xdomain.start,
        model.cfg.xdomain.end,
        model.cfg.ydomain.start,
        model.cfg.ydomain.end,
        x,
        y,
        model.cfg.max_iters,
    );
    let winp = app.window_rect().pad(20.0);
    let text_area = geom::Rect::from_wh(winp.wh()).top_left_of(winp);
    draw.text(&text)
        .xy(text_area.xy())
        .wh(text_area.wh())
        .align_text_bottom()
        .left_justify()
        .color(RED);

    // Write to window's frame
    draw.to_frame(app, &frame).unwrap();
}

/// Handle events related to the window and update the model if necessary
fn event(app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        // Window resize - update resolution
        Resized(size) => {
            if size != Vec2::ZERO {
                let size = size.to_array();
                let sf = app.window(model.window).unwrap().scale_factor();
                model.cfg.resolution.x = (sf * size[0]) as usize;
                model.cfg.resolution.y = (sf * size[1]) as usize;
                update(app, model);
            }
        }
        // Mouse press - start pan
        MousePressed(_button) => {
            model.pan_mode.is_active = true;
            model.pan_mode.start = Vec2::new(app.mouse.x, app.mouse.y);
        }
        // Mouse move - update pan, shift image buffer without calling mandel()
        MouseMoved(position) => {
            model.pan_mode.end = position;
            if model.pan_mode.is_active {
                model.pan_mode.draw = model.pan_mode.end - model.pan_mode.start;
            }
        }
        // Mouse release - end pan, update x,y domain, call mandel()
        MouseReleased(_button) => {
            model.pan_mode.is_active = false;
            model.pan_mode.draw = Vec2::ZERO;
            mouse_pan(app, model);
            update(app, model);
        }

        // Zoom with mouse wheel
        MouseWheel(LineDelta(_x, y), ..) => {
            mouse_zoom(app, model, y as f64);
            update(app, model);
        }
		// TODO! test this
        MouseWheel(PixelDelta(PhysicalPosition { x: _x, y }), ..) => {
            mouse_zoom(app, model, y);
            update(app, model);
        }

        // ,/. keys increase/reduce max_iters
        KeyPressed(Key::Period) => {
            if model.cfg.max_iters < 20000 {
                model.cfg.max_iters *= 2;
            }
            update(app, model);
        }
        KeyPressed(Key::Comma) => {
            if model.cfg.max_iters > 32 {
                model.cfg.max_iters /= 2;
            }
            update(app, model);
        }

        // +/- keys zoom in and out
        KeyPressed(Key::Plus) | KeyPressed(Key::NumpadAdd) => {
            keyboard_zoom(model, 0.25);
            update(app, model);
        }
        KeyPressed(Key::Minus) | KeyPressed(Key::NumpadSubtract) => {
            keyboard_zoom(model, -0.25);
            update(app, model);
        }

        // arrows keys pan the domain by half
        KeyPressed(Key::Up) => {
            keyboard_pan(model, 0.0, -0.25);
            update(app, model);
        }
        KeyPressed(Key::Down) => {
            keyboard_pan(model, 0.0, 0.25);
            update(app, model);
        }
        KeyPressed(Key::Right) => {
            keyboard_pan(model, -0.25, 0.0);
            update(app, model);
        }
        KeyPressed(Key::Left) => {
            keyboard_pan(model, 0.25, 0.0);
            update(app, model);
        }
		
		// Change color scheme
		KeyPressed(Key::C) => {
            model.color_schemes.next();
            update(app, model);
        }

        // R key resets domain to default
        KeyPressed(Key::R) => {
            model.cfg.xdomain.start = -2.5;
            model.cfg.xdomain.end = 1.0;
            model.cfg.ydomain.start = -1.0;
            model.cfg.ydomain.end = 1.0;
            update(app, model);
        }
        _ => println!("{:?}", event),
    }
}

/// Update image after changes in `model.cfg`
fn update(app: &App, model: &mut Model) {
    let iters = mandel(model.cfg);
    let imgbuf = get_image_buf(&iters, model);
    let image = image::DynamicImage::ImageRgb8(imgbuf);
    let texture = wgpu::Texture::from_image(app, &image);
    model.float_format_precision = get_ffmt_precision(model);
    model.texture = texture;
}

/// Return float format precision based on the current domain
fn get_ffmt_precision(model: &Model) -> usize {
    let delta = (model.cfg.xdomain.end - model.cfg.xdomain.start)
        .min(model.cfg.ydomain.end - model.cfg.ydomain.start);
    let precision = if delta > f64::MIN_POSITIVE {
        (2 - delta.log10() as i32) as usize
    } else {
        20
    };
    precision
}

/// Zoom with mouse. Update mandelbrot set x and y domains.
fn mouse_zoom(app: &App, model: &mut Model, delta: f64) {
    let y = delta / delta.abs();
    let zoom = 0.10 * y;
    let (x0, x1) = (model.cfg.xdomain.start, model.cfg.xdomain.end);
    let (y0, y1) = (model.cfg.ydomain.start, model.cfg.ydomain.end);
    let (dx, dy) = (x1 - x0, y1 - y0);
    let (ox, oy) = (dx * zoom, dy * zoom);
    let [x, y] = mouse2domain(app, model, model.pan_mode.end);
    let (fx, fy) = ((x - x0) / (x1 - x), (y - y0) / (y1 - y));
    let (ox0, oy0) = (ox * fx / (fx + 1.), oy * fy / (fy + 1.));
    model.cfg.xdomain.start += ox0;
    model.cfg.xdomain.end += -(ox - ox0);
    model.cfg.ydomain.start += oy0;
    model.cfg.ydomain.end += -(oy - oy0);
}

/// Zoom with keyboard. Update mandelbrot set x and y domains.
fn keyboard_zoom(model: &mut Model, zoom: f64) {
    let dx = zoom * (model.cfg.xdomain.end - model.cfg.xdomain.start);
    let dy = zoom * (model.cfg.ydomain.end - model.cfg.ydomain.start);
    model.cfg.xdomain.start += dx;
    model.cfg.xdomain.end -= dx;
    model.cfg.ydomain.start += dy;
    model.cfg.ydomain.end -= dy;
}

/// Pan with mouse. Update mandelbrot set x and y domains.
fn mouse_pan(app: &App, model: &mut Model) {
    let [x0, y0] = mouse2domain(app, model, model.pan_mode.start);
    let [x1, y1] = mouse2domain(app, model, model.pan_mode.end);
    let (dx, dy) = (x1 - x0, y1 - y0);
    model.cfg.xdomain.start -= dx;
    model.cfg.xdomain.end -= dx;
    model.cfg.ydomain.start -= dy;
    model.cfg.ydomain.end -= dy;
}

/// Pan with keyboard. Update mandelbrot set x and y domains.
fn keyboard_pan(model: &mut Model, panx: f64, pany: f64) {
    let xoffset = panx * (model.cfg.xdomain.end - model.cfg.xdomain.start);
    let yoffset = pany * (model.cfg.ydomain.end - model.cfg.ydomain.start);
    model.cfg.xdomain.start += xoffset;
    model.cfg.xdomain.end += xoffset;
    model.cfg.ydomain.start += yoffset;
    model.cfg.ydomain.end += yoffset;
}

/// Converts a window-relative `position` into Mandelbrot x,y domain
fn mouse2domain(app: &App, model: &Model, position: Vec2) -> [f64; 2] {
    let [px, py] = position.to_array();
    let (w, h) = app.window(model.window).unwrap().inner_size_points();

    // move origin from screen centre to bottom-left corner
    let px = (px + w / 2.0) as f64;
    let py = (py + h / 2.0) as f64;

    // current domain
    let (x0, x1) = (model.cfg.xdomain.start, model.cfg.xdomain.end);
    let (y0, y1) = (model.cfg.ydomain.start, model.cfg.ydomain.end);

    // calc new domain
    let x_new = x0 + px / w as f64 * (x1 - x0);
    let y_new = y0 + py / h as f64 * (y1 - y0);

    [x_new, y_new]
}

/// Return a buffer with the image of the mandelbrot set
fn get_image_buf(
    iters: &Vec<Vec<usize>>,
	model: &Model,
) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
	
    let resy = iters.len() as u32;
    let resx = iters[0].len() as u32;

    let mut imgbuf = image::ImageBuffer::new(resx, resy);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        // imgbuf is indexed top-left to bottom-right,
        // hence the y-index must be reversed:
        let c = iters[(resy - y - 1) as usize][x as usize];
        let (r, g, b) = model
			.color_schemes.get()
			.rgb(c, model.cfg.max_iters);
        *pixel = image::Rgb([r, g, b]);
    }
    imgbuf
}
