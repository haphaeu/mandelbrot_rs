use nannou::prelude::{
    App,
    Frame,
    WindowEvent,
    WindowId,
    Resized,
    wgpu,
    CORNFLOWERBLUE,
};
use nannou::image;

use mandelbrot_cli::{
    MandelConfig,
    Domain,
    Resolution,
    mandel,
};

fn main() {
    nannou::app(model).run();
}

struct Model {
    // Store the window ID so we can refer to this specific window later if needed.
    _window: WindowId,
    texture: wgpu::Texture,
    cfg: MandelConfig,
}

fn model(app: &App) -> Model {
    // Create a new window! Store the ID so we can refer to it later.
    let _window = app
        .new_window()
        .size(350, 200)
        .title("Mandelbrot Set")
        .view(view) // The function that will be called for presenting graphics to a frame.
        .event(event) // The function that will be called when the window receives events.
        .build()
        .unwrap();

    let cfg = MandelConfig { ..Default::default() };  
    let iters = mandel(cfg);
    let imgbuf = get_image_buf(&iters, cfg.max_iters);
    let image = image::DynamicImage::ImageRgb8(imgbuf);
    let texture = wgpu::Texture::from_image(app, &image);
    
    Model { _window, texture, cfg }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(CORNFLOWERBLUE);
    let draw = app.draw();
    draw.texture(&model.texture);
    draw.to_frame(app, &frame).unwrap();
}


// Handle events related to the window and update the model if necessary
fn event(app: &App, model: &mut Model, event: WindowEvent) {
    print!("{:?}", event);
    match event {
	Resized(size) => {
	    let size = size.to_array();
	    model.cfg.resolution.x = (app.window(model._window).unwrap().scale_factor() * size[0]) as usize;
	    model.cfg.resolution.y = (app.window(model._window).unwrap().scale_factor() * size[1]) as usize;
	    println!(" - changing image size to {:?}", model.cfg.resolution);
	    update(app, model);
	},
	_ => println!(" - event not implemented."),
    }
}


fn update(app: &App, model: &mut Model) {
    
    let iters = mandel(model.cfg);
    let imgbuf = get_image_buf(&iters, model.cfg.max_iters);
    let image = image::DynamicImage::ImageRgb8(imgbuf);
    let texture = wgpu::Texture::from_image(app, &image);
    model.texture = texture;
}

fn get_image_buf(
    iters: &Vec<Vec<usize>>,
    max_iters: usize
) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>>
{
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
    imgbuf
}
