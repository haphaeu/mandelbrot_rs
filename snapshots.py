import subprocess


def zoom(startx, starty, targetx, targety, frames):
    sx0, sx1 = startx
    sy0, sy1 = starty
    tx0, tx1 = targetx
    ty0, ty1 = targety
    syo = targetx[0]
    dx0 = (tx0 - sx0) / frames
    dx1 = (tx1 - sx1) / frames
    dy0 = (ty0 - sy0) / frames
    dy1 = (ty1 - sy1) / frames
    for f in range(frames):
       x = sx0 + f * dx0, sx1 + f * dx1
       y = sy0 + f * dy0, sy1 + f * dy1
       yield x, y

       
def run_cmd(xdomain, ydomain,
            max_iters=256, resx=1920, resy=1080, num=0,
            ):
       cmd_run = [
           'target/release/mandelbrot_cli',
           str(xdomain[0]), str(xdomain[1]),
           str(ydomain[0]), str(ydomain[1]),
           str(max_iters), str(resx), str(resy),
       ]
       cmd_copy = [
           'mv',
           'fractal.png',
           f'snaps/fractal{num:04d}.png'
       ]
       subprocess.run(cmd_run, capture_output=True)
       subprocess.run(cmd_copy)
    

def run_ffmpeg(fps):
       cmd_ffmpeg = [
           'ffmpeg',
           '-framerate', f'{fps}',
           '-pattern_type', 'glob', 'i', 'snaps/*.png',
           '-c:v', 'libx264',
           'out.mp4',
       ]
       subprocess.run(cmd_ffmpeg)
       
def main():           
    # starting domains
    startx = -2.5, 1
    starty = -1, 0
    
    # target zommed in domains
    tgtx = -0.523110006711778, -0.523110006711743
    tgty = 0.680764072151876, 0.680764072151898
    
    # frames per sec and transition time
    fps = 30
    ttime = 15
    frames = fps * ttime
    print(f"{frames=}")
    for i, (x, y) in enumerate(zoom(startx, starty, tgtx, tgty, frames)):
        print(f"{i}", end='\r')
        run_cmd(x, y, num=i)

    run_ffmpeg(fps)
    
if __name__ == '__main__':
    main()
