# mandelbrot

Implementation in Rust of the Mandelbrot set.

[Mandelbrot set](mandelbrot_cli/fractal.png)

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

Use the mouse to draw a rectangle to zoom in into a region.

Keyboard shortcuts:

 - `,`, `.` : period/comma to reduce/increase `max_iters`
 - `+`, `-` : plus/minus keys to zoom in/out
 - Arrows: use arrows keys to pan the domain
 - `R` : R key to reset to default domain x (-2.5, 1), y (-1, 1)
 