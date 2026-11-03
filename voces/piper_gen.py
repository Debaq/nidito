"""Genera voces con Piper (sin internet, voces con licencia libre).

Uso: python piper_gen.py <catalogo.tsv> <salida-es> <salida-en> <modelo-es.onnx> <modelo-en.onnx> <filtro.txt>
     python piper_gen.py --muestra <modelo.onnx> <salida.ogg> <filtro.txt> <texto>

El catálogo tiene líneas "archivo<TAB>idioma<TAB>texto". Solo genera los archivos que faltan.
Cada modelo se carga una sola vez; el audio pasa por el mismo filtro de ffmpeg que edge-tts.
"""

import os
import subprocess
import sys
import tempfile
import wave

from piper import PiperVoice, SynthesisConfig

# un poco más lento que lo normal: es para niños pequeños
CONFIG = SynthesisConfig(length_scale=1.12)


def a_ogg(voz: PiperVoice, texto: str, salida: str, filtro: str) -> bool:
    with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as t:
        tmp = t.name
    try:
        with wave.open(tmp, "wb") as w:
            voz.synthesize_wav(texto, w, syn_config=CONFIG)
        parcial = salida + ".tmp.ogg"
        r = subprocess.run(
            ["ffmpeg", "-loglevel", "error", "-y", "-i", tmp, "-af", filtro,
             "-ac", "1", "-ar", "22050", "-c:a", "libvorbis", "-q:a", "4", parcial],
            check=False,
        )
        if r.returncode != 0 or not os.path.getsize(parcial):
            return False
        os.replace(parcial, salida)
        return True
    finally:
        os.unlink(tmp)


def main() -> int:
    if sys.argv[1] == "--muestra":
        _, _, modelo, salida, filtro_txt, texto = sys.argv
        filtro = open(filtro_txt, encoding="utf-8").read().strip()
        return 0 if a_ogg(PiperVoice.load(modelo), texto, salida, filtro) else 1

    catalogo, out_es, out_en, modelo_es, modelo_en, filtro_txt = sys.argv[1:7]
    salidas = {"es": out_es, "en": out_en}
    filtro = open(filtro_txt, encoding="utf-8").read().strip()
    pendientes = {"es": [], "en": []}
    with open(catalogo, encoding="utf-8") as f:
        for linea in f:
            partes = linea.rstrip("\n").split("\t", 2)
            if len(partes) != 3:
                continue
            archivo, idioma, texto = partes
            if idioma not in salidas:
                continue
            ruta = os.path.join(salidas[idioma], archivo)
            if not (os.path.exists(ruta) and os.path.getsize(ruta)):
                pendientes.setdefault(idioma, []).append((ruta, texto))
    total = sum(len(v) for v in pendientes.values())
    if not total:
        return 0
    print(f"piper: {total} frases por generar", flush=True)
    hechas = fallas = 0
    for idioma, modelo in (("es", modelo_es), ("en", modelo_en)):
        if not pendientes.get(idioma):
            continue
        voz = PiperVoice.load(modelo)
        for ruta, texto in pendientes[idioma]:
            if a_ogg(voz, texto, ruta, filtro):
                hechas += 1
            else:
                fallas += 1
                print(f"  falló: {idioma} {texto}", file=sys.stderr)
            if (hechas + fallas) % 100 == 0:
                print(f"  {hechas + fallas} / {total}", flush=True)
    return 1 if fallas else 0


if __name__ == "__main__":
    sys.exit(main())
