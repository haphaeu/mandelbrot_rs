# mandelbrot

Implementation in Rust of the Mandelbrot set.

![Mandelbrot set](fractal.png)


§[Animation](https://www.youtube.com/watch?v=PxpqSaIbplE)

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
 
# Benchs

Have tried 3 different approaches for the paralel processing:

 - `thread::spawn` with chunks of rows
 - `threadpool::execute` with chunks of rows
 - `threadpool::execute` for every row

All approaches give similar performance for low setting, however
for high setting, using `threadpool::execute` for every row gives 
the best performance. 


### `thread::spawn` with chunks

The y domain is discretised in `4 * CPUs`. The factor `4` is optimised.
Then each of these chunks is sent to a thread for processing.

Drawback is that is a thread finishes early, it does not take more jobs
to run.

### `threadpool::execute` with chunks

A threadpool is created with a number of processing units equal to
the `1 * CPUs` in the computer. Then the y domain is discretised in 
`2 * CPUs`. 

Doing this, the threadpool jobs that finished faster can take more 
jobs from the pool.

Performance increase over `thread::spawn`: 7% less time


### `threadpool::execute` no chunks

Here the domain is not chunked. Instead, each row becomes one job.
The list of jobs is sent to a `threadpool` with `8 * CPUs` processing
units. The factor `8` has been optimised. 

With smaller jobs, the threads processing rows quicker become available
to fetch more jobs from the pool. 

This seems to be the fastest method with performance nearly twice
of the previous approaches.

Performance increase over `thread::spawn`: 50% less time
