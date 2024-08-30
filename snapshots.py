import os
import subprocess
import multiprocessing
import tqdm

def zoom(startx, starty, targetx, targety, frames):
    sx0, sx1 = startx
    sy0, sy1 = starty
    tx0, tx1 = targetx
    ty0, ty1 = targety
    r = 0.9
    dx0 = (tx0 - sx0) * (1 -r) / (1 - r**(frames - 1))
    dx1 = (tx1 - sx1) * (1 -r) / (1 - r**(frames - 1))
    dy0 = (ty0 - sy0) * (1 -r) / (1 - r**(frames - 1))
    dy1 = (ty1 - sy1) * (1 -r) / (1 - r**(frames - 1))
    x, y = startx, starty
    for f in range(frames):
        yield x, y
        x = x[0] + dx0 * r**f, x[1] + dx1 * r**f
        y = y[0] + dy0 * r**f, y[1] + dy1 * r**f

       
def run_cmd(xdomain, ydomain, resx, resy, max_iters, num):

    fname = f'snaps/fractal_{num:04d}.png'
    if os.path.exists(fname):
        return
    
    cmd_run = [
        'target/release/mandelbrot_cli',
        str(xdomain[0]), str(xdomain[1]),
        str(ydomain[0]), str(ydomain[1]),
        str(max_iters), str(resx), str(resy),
        fname
    ]
    subprocess.run(cmd_run, capture_output=True)

       
def worker(job):
    run_cmd(*job)

    
def run_ffmpeg(fps):
    cmd_ffmpeg = [
        'ffmpeg',
        '-hide_banner',
        '-loglevel', 'error',
        '-stats', 
        '-framerate', f'{fps}',
        '-pattern_type', 'glob', '-i', 'snaps/*.png',
        '-c:v', 'libx264',
        '-crf', '15',
        '-tune', 'stillimage',
        '-pix_fmt', 'yuv444p',
        'out.mp4',
    ]
    subprocess.run(cmd_ffmpeg)
       
def main():
    
    # starting domains
    startx = -2.5, 1.0
    starty = 0.0, 1.0
    
    # target zommed-in domains
    tgtx = -0.523110006711778, -0.523110006711743
    tgty =  0.680764072151876,  0.680764072151898
    
    # frames per sec and transition time
    fps = 60
    ttime = 15
    
    frames = fps * ttime
    print(f"{frames=}")

    if not os.path.exists('snaps/'):
        os.mkdir('snaps')
    
    # Run the jobs in parallel, with a progress bar
    # Note: don't go too crazy in number of processes, since
    # mandelbrot_cli itself already (ab)uses multithreading.
    pool = multiprocessing.Pool(2)
    jobs = []
    for i, (x, y) in enumerate(zoom(startx, starty, tgtx, tgty, frames)):
        jobs.append((x, y, 1920, 1080, 2048, i))

    for _ in tqdm.tqdm(pool.imap_unordered(worker, jobs), total=len(jobs)):
        pass

    print("Creating the video")
    run_ffmpeg(fps)
    
    
if __name__ == '__main__':
    main()
