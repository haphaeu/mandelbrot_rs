use nannou::image;
use nannou::prelude::{
    geom, wgpu, App, Frame, Key, KeyPressed, MouseMoved, MousePressed, MouseReleased,
    MouseScrollDelta::LineDelta, MouseWheel, Resized, Vec2, WindowEvent, WindowId,
    CORNFLOWERBLUE, RED,
};

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
    float_format_precision: usize,
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
    let float_format_precision = 3;

    Model {
        window,
        texture,
        cfg,
        pan_mode,
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
    let text = format!(
        "x ({:.*}, {:.*}), y ({:.*}, {:.*}), y\nMouse @ {:.*}, {:.*}\nMax iters: {}",
        model.float_format_precision,
        model.cfg.xdomain.start,
        model.float_format_precision,
        model.cfg.xdomain.end,
        model.float_format_precision,
        model.cfg.ydomain.start,
        model.float_format_precision,
        model.cfg.ydomain.end,
        model.float_format_precision,
        x,
        model.float_format_precision,
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
            if model.pan_mode.is_active {
                model.pan_mode.end = position;
                model.pan_mode.draw = model.pan_mode.end - model.pan_mode.start;
            }
        }
        // Mouse release - end pan, update x,y domain, call mandel()
        MouseReleased(_button) => {
            model.pan_mode.is_active = false;
            model.pan_mode.draw = Vec2::ZERO;
            update_domain(app, model);
            update(app, model);
        }

        // Zoom with mouse wheel
        MouseWheel(LineDelta(_x, y), ..) => {
            let zoom = 0.10 * y as f64;
            let (x0, x1) = (model.cfg.xdomain.start, model.cfg.xdomain.end);
            let (y0, y1) = (model.cfg.ydomain.start, model.cfg.ydomain.end);
            let (dx, dy) = (x1 - x0, y1 - y0);
            let (ox, oy) = (dx * zoom, dy * zoom);
            model.cfg.xdomain.start += ox;
            model.cfg.xdomain.end += -ox;
            model.cfg.ydomain.start += oy;
            model.cfg.ydomain.end += -oy;
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
        _ => (),
    }
}

/// Update image after changes in `model.cfg`
fn update(app: &App, model: &mut Model) {
    let iters = mandel(model.cfg);
    let imgbuf = get_image_buf(&iters, model.cfg.max_iters);
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

/// Update mandelbrot set x and y domains after pan_mode with mouse
fn update_domain(app: &App, model: &mut Model) {
    let [x0, y0] = mouse2domain(app, model, model.pan_mode.start);
    let [x1, y1] = mouse2domain(app, model, model.pan_mode.end);
    let (dx, dy) = (x1 - x0, y1 - y0);
    model.cfg.xdomain.start += -dx;
    model.cfg.xdomain.end += -dx;
    model.cfg.ydomain.start += dy;
    model.cfg.ydomain.end += dy;
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
            _ => todo!(),
        }
    } else {
        (0, 0, 0)
    }
}
