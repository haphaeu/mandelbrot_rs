use nannou::prelude::{
    geom, wgpu, App, Frame, LoopMode, 
    Key, KeyPressed, KeyReleased,
    MouseMoved, MousePressed, MouseReleased,
    MouseScrollDelta::LineDelta, MouseScrollDelta::PixelDelta, MouseWheel, Resized, Update, Vec2,
    WindowEvent, WindowId, BLACK, RED,
};
use nannou::image;
use nannou::winit::dpi::PhysicalPosition;
use mandelbrot_cli::{mandel, MandelConfig};
mod color_schemes;

fn main() {
    nannou::app(model)
        // Vulkan works-ish in WSL. Setting this is not required in native Linux or Windows
        //.backends(wgpu::Backends::VULKAN) 
        .loop_mode(LoopMode::Wait)
        .update(update)
        .run();
}

struct Model {
    // Store the window ID so we can refer to this specific window later if needed.
    window: WindowId,
    texture: wgpu::Texture,
    cfg: MandelConfig,
    pan_mode: SelectMode,
    rect_mode: SelectMode,
    color_schemes: color_schemes::ColorSchemes,
    float_format_precision: usize,
    flag_update: bool,
}

/// Track keys and mouse moves to pan or zoom with a rectangle
struct SelectMode {
    is_active: bool,
    start: Vec2,
    end: Vec2,
    draw: Vec2,
}
impl Default for SelectMode {
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
    let (w, h) = (800, 450);

    let window = app
        .new_window()
        .size(w, h)
        .title("Mandelbrot Set")
        .view(view)
        .event(event)
        .build()
        .unwrap();

    let texture = wgpu::TextureBuilder::new()
        .size([w, h])
        .format(wgpu::TextureFormat::Rgba8Unorm)
        .build(app.window(window).unwrap().device());

    Model {
        window,
        texture,
        cfg: MandelConfig::default(),
        pan_mode: SelectMode::default(),
        rect_mode: SelectMode::default(),
        color_schemes: color_schemes::ColorSchemes::new(),
        float_format_precision: 3,
        flag_update: false,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    //println!("{_update:?}");
    update_mandel(app, model)
}

/// Update image after changes in `model.cfg`
fn update_mandel(app: &App, model: &mut Model) {
    if model.flag_update {
        let iters = mandel(model.cfg);
        let imgbuf = get_image_buf(&iters, model);
        let image = image::DynamicImage::ImageRgb8(imgbuf);
        let texture = wgpu::Texture::from_image(app, &image);
        model.float_format_precision = get_ffmt_precision(model);
        model.texture = texture;
        model.flag_update = false;
    }
}

fn image2file(model: &Model) {
    let iters = mandel(model.cfg);
    let imgbuf = get_image_buf(&iters, model);
    imgbuf.save("fractal.png").unwrap();
    println!("Image saved to 'fractal.png'");
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);
    let draw = app.draw();

    // Draw the image
    draw.texture(&model.texture).xy(model.pan_mode.draw);

    // Draw the selection rectangle
    if model.rect_mode.is_active && model.rect_mode.draw != Vec2::ZERO {
        let [x0, y0] = model.rect_mode.start.to_array();
        let [x1, y1] = model.rect_mode.end.to_array();
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
    let [x, y] = mouse2domain(app, model, model.pan_mode.end);
    let p = model.float_format_precision;
    let text = format!(
        "x ({:.p$}, {:.p$}), y ({:.p$}, {:.p$}) \nMouse @ {:.p$}, {:.p$}\nMax iters: {}",
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
    //println!("{event:?}");
    match event {
        // Window resize - update resolution
        Resized(size) => {
            if size != Vec2::ZERO {
                let size = size.to_array();
                let sf = app.window(model.window).unwrap().scale_factor();
                model.cfg.resolution.x = (sf * size[0]) as usize;
                model.cfg.resolution.y = (sf * size[1]) as usize;
                model.flag_update = true;
            }
        }
        // Mouse press - start pan
        MousePressed(_button) => {
            if model.rect_mode.is_active {
                model.rect_mode.start = Vec2::new(app.mouse.x, app.mouse.y);
                // for rect_mode, `draw` is a flag to activate drawing after 
                // Ctrl or Shift key is pressed
                model.rect_mode.draw = Vec2::ONE;
            } else {
                model.pan_mode.is_active = true;
                model.pan_mode.start = Vec2::new(app.mouse.x, app.mouse.y);
            }
        }
        // Mouse move - update pan, shift image buffer without calling mandel()
        MouseMoved(position) => {
            model.pan_mode.end = position;
            model.rect_mode.end = position;
            if model.pan_mode.is_active {
                // For pan_mode, `draw` is the offset to shift the image buffer
                model.pan_mode.draw = model.pan_mode.end - model.pan_mode.start;
            } 
        }
        // Mouse release - end pan, update x,y domain, call mandel()
        MouseReleased(_button) => {
            if model.pan_mode.is_active {
                model.pan_mode.is_active = false;
                model.pan_mode.draw = Vec2::ZERO;
                mouse_pan(app, model);
            } else if model.rect_mode.is_active {
                model.rect_mode.is_active = false;
                mouse_zoom_rect(app, model);
            }
        }
        
        // Ctrl or Shift keys zoom with rectangle
        KeyPressed(Key::LControl) | KeyPressed(Key::LShift) => {
            if ! model.rect_mode.is_active {
                model.rect_mode.is_active = true;
                model.rect_mode.draw = Vec2::ZERO;
        }
        }
        KeyReleased(Key::LControl) | KeyReleased(Key::LShift) => {
            model.rect_mode.is_active = false;
            model.rect_mode.draw = Vec2::ZERO;
        }

        // Zoom with mouse wheel
        MouseWheel(LineDelta(_x, y), ..) => {
            mouse_zoom(app, model, y as f64);
        }
        MouseWheel(PixelDelta(PhysicalPosition { x: _x, y }), ..) => {
            mouse_zoom(app, model, y);
        }

        // ,/. keys increase/reduce max_iters
        KeyPressed(Key::Period) => {
            if model.cfg.max_iters < 20000 {
                model.cfg.max_iters *= 2;
                model.flag_update = true;
            }
        }
        KeyPressed(Key::Comma) => {
            if model.cfg.max_iters > 32 {
                model.cfg.max_iters /= 2;
                model.flag_update = true;
            }
        }

        // +/- keys zoom in and out
        KeyPressed(Key::Plus) | KeyPressed(Key::NumpadAdd) => {
            keyboard_zoom(model, 0.25);
        }
        KeyPressed(Key::Minus) | KeyPressed(Key::NumpadSubtract) => {
            keyboard_zoom(model, -0.25);
        }

        // arrows keys pan the domain by half
        KeyPressed(Key::Up) => {
            keyboard_pan(model, 0.0, -0.25);
        }
        KeyPressed(Key::Down) => {
            keyboard_pan(model, 0.0, 0.25);
        }
        KeyPressed(Key::Right) => {
            keyboard_pan(model, -0.25, 0.0);
        }
        KeyPressed(Key::Left) => {
            keyboard_pan(model, 0.25, 0.0);
        }

        // Change color scheme
        KeyPressed(Key::C) => {
            model.color_schemes.next();
            model.flag_update = true;
        }

        // R key resets domain to default
        KeyPressed(Key::R) => {
            model.cfg.xdomain.start = -2.5;
            model.cfg.xdomain.end = 1.0;
            model.cfg.ydomain.start = -1.0;
            model.cfg.ydomain.end = 1.0;
            model.flag_update = true;
        }

        // F key saves image to file
        KeyPressed(Key::F) => {
            image2file(model);
        }
        _ => (),
    }
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
    if delta.abs() < f64::MIN_POSITIVE {
        return;
    }
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
    model.flag_update = true;
}

/// Update mandelbrot set x and y domains after selection with mouse
fn mouse_zoom_rect(app: &App, model: &mut Model) {
    let [x0, y0] = mouse2domain(app, model, model.rect_mode.start);
    let [x1, y1] = mouse2domain(app, model, model.rect_mode.end);
    (model.cfg.xdomain.start, model.cfg.xdomain.end) = min_max(x0, x1);
    (model.cfg.ydomain.start, model.cfg.ydomain.end) = min_max(y0, y1);
    model.flag_update = true;
}

/// Zoom with keyboard. Update mandelbrot set x and y domains.
fn keyboard_zoom(model: &mut Model, zoom: f64) {
    let dx = zoom * (model.cfg.xdomain.end - model.cfg.xdomain.start);
    let dy = zoom * (model.cfg.ydomain.end - model.cfg.ydomain.start);
    model.cfg.xdomain.start += dx;
    model.cfg.xdomain.end -= dx;
    model.cfg.ydomain.start += dy;
    model.cfg.ydomain.end -= dy;
    model.flag_update = true;
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
    model.flag_update = true;
}

/// Pan with keyboard. Update mandelbrot set x and y domains.
fn keyboard_pan(model: &mut Model, panx: f64, pany: f64) {
    let xoffset = panx * (model.cfg.xdomain.end - model.cfg.xdomain.start);
    let yoffset = pany * (model.cfg.ydomain.end - model.cfg.ydomain.start);
    model.cfg.xdomain.start += xoffset;
    model.cfg.xdomain.end += xoffset;
    model.cfg.ydomain.start += yoffset;
    model.cfg.ydomain.end += yoffset;
    model.flag_update = true;
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
        let (r, g, b) = model.color_schemes.get().rgb(c, model.cfg.max_iters);
        *pixel = image::Rgb([r, g, b]);
    }
    imgbuf
}

/// Return a tuple `(min(a, b), max(a, b))`
fn min_max(a: f64, b: f64) -> (f64, f64) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}