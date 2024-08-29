# mandelbrot

Implementation in Rust of the Mandelbrot set.

![Mandelbrot set](fractal.png)

# `mandelbrot_cli`

Low level library and CLI to get a static image of the set.

CLI parameters are:

```
mandelbrot_cli x0 x1 y0 y1 max_iters resx resy
```

where:

 - `x0`, `x1` : domain along the x axis
 - `y0`, `y1` : domain along the y axis
 - `max_iters` : maximum number of iterations for divergence
 - `resx`, `resy` : s and y resolution of the image

The image is saved as `fractal.png`

# `mandelbrot_gui`

GUI to visualise the set dynamically.

Mouse moves:

 - drag the mouse to pan
 - scroll mouse wheel to zoom
 - press Ctrl or Shift and drag the mouse to select a rectangle to zoom in

Keyboard shortcuts:

 - `,`, `.` : reduce/increase `max_iters`
 - `+`, `-` : zoom in/out
 - Arrows: use arrows keys to pan the domain
 - `R` : reset to default domain x (-2.5, 1), y (-1, 1)
 - `C` : change color scheme
 - `F` : save current image to `fractal.png`

# TODO

 - [x] more color schemes.
 - [x] switch between color schemes.
 - [x] save current view to file
 - [x] bring back the rectangle
 - [x] fiz mouse wheel zoom
 - [x] touchpad scroll support
 - [x] avoid touchpad scroll event flood
 - [x] set flag to update 