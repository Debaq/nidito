# Uso: python3 tramo.py <archivo> <segundos>  -> imprime el inicio (s) del tramo más fuerte
import subprocess, sys, array
arch, dur = sys.argv[1], float(sys.argv[2])
sr = 8000
raw = subprocess.run(["ffmpeg", "-loglevel", "error", "-i", arch, "-ac", "1", "-ar", str(sr), "-f", "s16le", "-"], capture_output=True, stdin=subprocess.DEVNULL).stdout
x = array.array("h", raw)
n, w = len(x), int(dur * sr)
if n <= w:
    print(0.0); sys.exit()
blk = sr // 10                                      # energía por bloques de 0,1 s
e = [sum(v * v for v in x[i:i + blk]) for i in range(0, n, blk)]
k = w // blk
mejor, ini, s = -1, 0, sum(e[:k])
for i in range(len(e) - k + 1):
    if i: s += e[i + k - 1] - e[i - 1]
    if s > mejor: mejor, ini = s, i
print(round(max(0, ini * 0.1 - 0.1), 2))
