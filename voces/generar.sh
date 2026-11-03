#!/usr/bin/env bash
# Genera el paquete de voces pregrabadas con edge-tts (voces neuronales de Microsoft).
# Uso: voces/generar.sh <binario-nidito> <nidito.conf> <carpeta-salida>
# Solo se ejecuta al construir (necesita internet); el netbook solo reproduce los .ogg.
set -euo pipefail
BIN=$(realpath "$1"); CONF=$(realpath "$2"); OUT=$(realpath -m "$3")
VENV="$(cd "$(dirname "$0")" && pwd)/.venv"
FILTRO="$(cd "$(dirname "$0")" && pwd)/filtro.txt"   # cadena ffmpeg: voces más fuertes para parlantes de netbook
[ -x "$VENV/bin/edge-tts" ] || { uv venv --python 3.12 "$VENV" -q && uv pip install --python "$VENV/bin/python" -q edge-tts; }
mkdir -p "$OUT"
"$BIN" --frases "$CONF" > "$OUT/catalogo.tsv"
export OUT VENV FILTRO
gen() {
    local archivo=$1 idioma=$2 texto=$3
    [ -s "$OUT/$archivo" ] && return 0
    local voz=es-CL-CatalinaNeural
    [ "$idioma" = en ] && voz=en-US-AnaNeural
    local tmp; tmp=$(mktemp --suffix=.mp3)
    for _ in 1 2 3 4; do
        "$VENV/bin/edge-tts" --voice "$voz" --rate=-8% --text "$texto" --write-media "$tmp" 2>/dev/null && [ -s "$tmp" ] && break
        sleep 3
    done
    ffmpeg -loglevel error -y -i "$tmp" -af "$(cat "$FILTRO")" -ac 1 -ar 22050 -c:a libvorbis -q:a 4 "$OUT/$archivo.tmp.ogg" && mv "$OUT/$archivo.tmp.ogg" "$OUT/$archivo"
    rm -f "$tmp"
}
export -f gen
total=$(wc -l < "$OUT/catalogo.tsv")
echo "frases en el catálogo: $total"
while IFS=$'\t' read -r a i t; do printf '%s\0%s\0%s\0' "$a" "$i" "$t"; done < "$OUT/catalogo.tsv" \
    | xargs -0 -n3 -P8 bash -c 'gen "$@"' _
# quitar audios que ya no están en el catálogo
cut -f1 "$OUT/catalogo.tsv" | sort > "$OUT/.validos"
for f in "$OUT"/*.ogg; do grep -qxF "$(basename "$f")" "$OUT/.validos" || rm -f "$f"; done
rm -f "$OUT/.validos"
hechos=$(ls "$OUT"/*.ogg | wc -l)
echo "audios: $hechos / $total  ($(du -sh "$OUT" | cut -f1))"
[ "$hechos" -eq "$total" ] || { echo "FALTAN audios, vuelve a ejecutar"; exit 1; }
