use nannou::image;
use nannou::prelude::{
    geom, wgpu, App, Frame, Key, KeyPressed, MouseMoved, MousePressed, MouseReleased, Resized,
    Vec2, WindowEvent, WindowId,
    CORNFLOWERBLUE, YELLOW, RED, ORANGE,
};

use mandelbrot_cli::{mandel, MandelConfig};

fn main() {
    nannou::app(model).run();
}

struct Model {
    // Store the window ID so we can refer to this specific window later if needed.
    _window: WindowId,
    texture: wgpu::Texture,
    cfg: MandelConfig,
    selection: Selection,
}

/// Rectangle selected by dragging the mouse,
/// used to zoom in into the image.
struct Selection {
    is_active: bool,
    start: Vec2,
    end: Vec2,
}
impl Selection {
    /// Return true if selected is larger than a given size
    fn is_valid(&self) -> bool {
        let threashold = 25.0 as f32;
        let delta = (self.end - self.start).abs();
        delta.to_array().iter().all(|&x| x >= threashold)
    }
}
impl Default for Selection {
    fn default() -> Self {
        Self {
            is_active: false,
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(0.0, 0.0),
        }
    }
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    let _window = app
        .new_window()
        .size(350, 200)
        .title("Mandelbrot Set")
        .view(view)
        .event(event)
        .build()
        .unwrap();

    let cfg = MandelConfig {
        ..Default::default()
    };
    let texture = wgpu::TextureBuilder::new()
        .size([350, 200])
        .format(wgpu::TextureFormat::Rgba8Unorm)
        .build(app.window(_window).unwrap().device());

    let selection = Selection::default();

    Model {
        _window,
        texture,
        cfg,
        selection,
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(CORNFLOWERBLUE);
    let draw = app.draw();

    // Draw the image
    draw.texture(&model.texture);

    // Draw the selection rectangle
    if model.selection.is_active && model.selection.is_valid() {
        let [x0, y0] = model.selection.start.to_array();
        let [x1, y1] = model.selection.end.to_array();
        let points = [
            Vec2::new(x0, y0),
            Vec2::new(x1, y0),
            Vec2::new(x1, y1),
            Vec2::new(x0, y1),
        ];

        draw.polyline()
            .weight(1.0)
            .rgb8(255, 0, 0)
            .points_closed(points);
    }

    // Write some text
    let [x, y] = mouse2domain(app, model, model.selection.end);
    let text = format!(
        "x ({:.3}, {:.3}), y ({:.3}, {:.3}), y\nMouse @ {x:.3}, {y:.3}",
        model.cfg.xdomain.start,
        model.cfg.xdomain.end,
        model.cfg.ydomain.start,
        model.cfg.ydomain.end,
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
    //print!("{:?}", event);
    match event {
        // Window resize - update resolution
        Resized(size) => {
            print!("{:?}", event);
            if size != Vec2::ZERO {
                let size = size.to_array();
                let sf = app.window(model._window).unwrap().scale_factor();
                model.cfg.resolution.x = (sf * size[0]) as usize;
                model.cfg.resolution.y = (sf * size[1]) as usize;
                println!(" - changing image size to {:?}", model.cfg.resolution);
                update(app, model);
            }
        }
        // Mouse press - start selection rectangle
        MousePressed(_button) => {
            print!("{:?}", event);
            model.selection.is_active = true;
            model.selection.start = Vec2::new(app.mouse.x, app.mouse.y);
            println!(
                " - select active, start at {}, {}",
                app.mouse.x, app.mouse.y
            );
        }
        MouseMoved(position) => {
            model.selection.end = position;
        }
        // Mouse release - end rectangle selection
        // Update x,y domain
        MouseReleased(_button) => {
            print!("{:?}", event);
            model.selection.is_active = false;
            if model.selection.is_valid() {
                print!(
                    " at {:?}- domain updated from {:?}, {:?}",
                    model.selection.end, model.cfg.xdomain, model.cfg.ydomain
                );
                update_domain(app, model);
                println!(" to {:?}, {:?}", model.cfg.xdomain, model.cfg.ydomain);
                update(app, model);
            }
        }
	
	// ,/. keys increase/reduce max_iters
        KeyPressed(Key::Period) => {
	    if model.cfg.max_iters < 20000 {
		model.cfg.max_iters *= 2;
	    }
            println!(" max_iters updated to {}", model.cfg.max_iters);
            update(app, model);
        }
        KeyPressed(Key::Comma) => {
            if model.cfg.max_iters > 32 {
                model.cfg.max_iters /= 2;
            }
            println!(" max_iters updated to {}", model.cfg.max_iters);
            update(app, model);
        }

	// +/- keys zoom in and out
	KeyPressed(Key::Plus) | KeyPressed(Key::NumpadAdd) => {
	    let dx = (model.cfg.xdomain.end - model.cfg.xdomain.start) / 4.0;
	    let dy = (model.cfg.ydomain.end - model.cfg.ydomain.start) / 4.0;
            model.cfg.xdomain.start += dx;
	    model.cfg.xdomain.end -= dx;
	    model.cfg.ydomain.start += dy;
	    model.cfg.ydomain.end -= dy;
            update(app, model);
        }
        KeyPressed(Key::Minus) | KeyPressed(Key::NumpadSubtract) => {
            let dx = -(model.cfg.xdomain.end - model.cfg.xdomain.start) / 4.0;
	    let dy = -(model.cfg.ydomain.end - model.cfg.ydomain.start) / 4.0;
            model.cfg.xdomain.start += dx;
	    model.cfg.xdomain.end -= dx;
	    model.cfg.ydomain.start += dy;
	    model.cfg.ydomain.end -= dy;
            update(app, model);
        }
	
	// arrows keys pan the domain by half
        KeyPressed(Key::Up) => {
	    let offset = (model.cfg.ydomain.end - model.cfg.ydomain.start) / 4.0;
            model.cfg.ydomain.start += offset;
            model.cfg.ydomain.end += offset;
            update(app, model);
        }
	KeyPressed(Key::Down) => {
	    let offset = (model.cfg.ydomain.end - model.cfg.ydomain.start) / 4.0;
            model.cfg.ydomain.start -= offset;
            model.cfg.ydomain.end -= offset;
            update(app, model);
        }
	KeyPressed(Key::Right) => {
	    let offset = (model.cfg.xdomain.end - model.cfg.xdomain.start) / 4.0;
            model.cfg.xdomain.start -= offset;
            model.cfg.xdomain.end -= offset;
            update(app, model);
        }
        KeyPressed(Key::Left) => {
	    let offset = (model.cfg.xdomain.end - model.cfg.xdomain.start) / 4.0;
            model.cfg.xdomain.start += offset;
            model.cfg.xdomain.end += offset;
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
	
	KeyPressed(_key) => {
	    println!("{:?}", event);
	}
        _ => (), //println!(" - event not implemented."),
    }
}

/// Update image after changes in `model.cfg`
fn update(app: &App, model: &mut Model) {
    let iters = mandel(model.cfg);
    let imgbuf = get_image_buf(&iters, model.cfg.max_iters);
    let image = image::DynamicImage::ImageRgb8(imgbuf);
    let texture = wgpu::Texture::from_image(app, &image);
    model.texture = texture;
}

/// Update mandelbrot set x and y domains after selection with mouse
fn update_domain(app: &App, model: &mut Model) {
    let [x0, y0] = mouse2domain(app, model, model.selection.start);
    let [x1, y1] = mouse2domain(app, model, model.selection.end);

    (model.cfg.xdomain.start, model.cfg.xdomain.end) = min_max(x0, x1);
    (model.cfg.ydomain.start, model.cfg.ydomain.end) = min_max(y0, y1);
}

/// Return a tuple `(min(a, b), max(a, b))`
fn min_max(a: f64, b: f64) -> (f64, f64) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Converts a window-relative `position` into Mandelbrot x,y domain
fn mouse2domain(app: &App, model: &Model, position: Vec2) -> [f64; 2] {
    let [px, py] = position.to_array();
    let (w, h) = app.window(model._window).unwrap().inner_size_points();

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

fn get_image_buf(
    iters: &Vec<Vec<usize>>,
    max_iters: usize,
) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    let resy = iters.len() as u32;
    let resx = iters[0].len() as u32;

    let mut imgbuf = image::ImageBuffer::new(resx, resy);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let c = iters[y as usize][x as usize];
	let (r, g, b) = color_scheme(c, max_iters);
        *pixel = image::Rgb([r, g, b]);
    }
    imgbuf
}

// Returns a tuple `(r, g, b)` for the RGB color for
// a number of iterations `c` and `max_iters`.
fn color_scheme(c: usize, max_iters: usize) -> (u8, u8, u8) {

    let scheme = "bluey";
    
    if c < max_iters {
	let c = c as f64;
	match scheme {
	    "greeny" => (
		(255.0 * c / max_iters as f64) as u8,
		255 as u8,
		(255.0 * c / (c + 8.0)) as u8,
	    ),
	    "bluey" => (
		(255.0 * c / max_iters as f64) as u8,
		(255.0 * c / (c + 8.0)) as u8,
		255 as u8,
	    ),
	    _ => todo!()
	}
    } else {
	(0, 0, 0)
    }
}
