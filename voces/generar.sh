#!/usr/bin/env bash
# Genera el paquete de voces pregrabadas. Pregunta qué motor usar:
#   piper : voces libres, sin internet (licencias que permiten redistribuir). Para ISOs públicas y la versión web.
#   edge  : edge-tts (voces neuronales de Microsoft, necesita internet). Más naturales, pero solo para uso personal.
# Uso: voces/generar.sh <binario-nidito> <nidito.conf> <carpeta-salida>
# Sin terminal (o para no preguntar): NIDITO_MOTOR=piper|edge. Voces de piper: NIDITO_PIPER_ES / NIDITO_PIPER_EN.
#
# Los audios se guardan por motor en voces/cache/ (así cambiar de motor no obliga a regenerar todo)
# y la carpeta de salida queda solo con los del catálogo actual. El netbook solo reproduce los .ogg.
set -euo pipefail
BIN=$(realpath "$1"); CONF=$(realpath "$2"); OUT=$(realpath -m "$3")
AQUI="$(cd "$(dirname "$0")" && pwd)"
VENV="$AQUI/.venv"
FILTRO="$AQUI/filtro.txt"   # cadena ffmpeg: voces más fuertes para parlantes de netbook
CACHE="$AQUI/cache"
PIPER_ES=${NIDITO_PIPER_ES:-es_MX-claude-high}
PIPER_EN=${NIDITO_PIPER_EN:-en_US-ljspeech-high}

# --- elegir motor ---
MOTOR=${NIDITO_MOTOR:-}
if [ -z "$MOTOR" ]; then
    anterior=$(cat "$OUT/.motor" 2>/dev/null || true)
    # paquete hecho antes de que existiera la pregunta: era edge-tts
    [ -z "$anterior" ] && compgen -G "$OUT/*.ogg" > /dev/null && anterior=edge
    anterior=${anterior:-piper}
    if [ -r /dev/tty ] && [ -w /dev/tty ]; then
        {
            echo
            echo "¿Con qué voces?"
            echo "  1) piper  - libres y sin internet: para compartir (ISO pública, versión web)"
            echo "  2) edge   - edge-tts de Microsoft: más naturales, solo para uso personal (necesita internet)"
        } > /dev/tty
        def=1; [ "$anterior" = edge ] && def=2
        read -rp "Elige 1 o 2 [$def]: " r < /dev/tty || r=""
        case ${r:-$def} in 2|edge) MOTOR=edge ;; *) MOTOR=piper ;; esac
    else
        MOTOR=$anterior
    fi
fi
case $MOTOR in piper|edge) ;; *) echo "Motor desconocido: $MOTOR (usa piper o edge)"; exit 1 ;; esac

mkdir -p "$OUT" "$CACHE"
# paquete viejo (antes había solo edge-tts y sin caché): se aprovecha como caché de edge
if [ ! -d "$CACHE/edge" ] && [ ! -f "$OUT/.motor" ] && compgen -G "$OUT/*.ogg" > /dev/null; then
    echo ">> Guardando las voces edge-tts existentes en voces/cache/edge"
    mkdir -p "$CACHE/edge" && cp -l "$OUT"/*.ogg "$CACHE/edge/" 2>/dev/null || cp "$OUT"/*.ogg "$CACHE/edge/"
fi

"$BIN" --frases "$CONF" > "$OUT/catalogo.tsv"
total=$(wc -l < "$OUT/catalogo.tsv")
echo "frases en el catálogo: $total  (motor: $MOTOR)"

if [ "$MOTOR" = edge ]; then
    DIR_ES="$CACHE/edge"; DIR_EN="$CACHE/edge"
    [ -x "$VENV/bin/edge-tts" ] || { [ -x "$VENV/bin/python" ] || uv venv --python 3.12 "$VENV" -q; uv pip install --python "$VENV/bin/python" -q edge-tts; }
    mkdir -p "$DIR_ES"
    export DIR_ES VENV FILTRO
    gen() {
        local archivo=$1 idioma=$2 texto=$3
        [ -s "$DIR_ES/$archivo" ] && return 0
        local voz=es-CL-CatalinaNeural
        [ "$idioma" = en ] && voz=en-US-AnaNeural
        local tmp; tmp=$(mktemp --suffix=.mp3)
        for _ in 1 2 3 4; do
            "$VENV/bin/edge-tts" --voice "$voz" --rate=-8% --text "$texto" --write-media "$tmp" 2>/dev/null && [ -s "$tmp" ] && break
            sleep 3
        done
        ffmpeg -loglevel error -y -i "$tmp" -af "$(cat "$FILTRO")" -ac 1 -ar 22050 -c:a libvorbis -q:a 4 "$DIR_ES/$archivo.tmp.ogg" && mv "$DIR_ES/$archivo.tmp.ogg" "$DIR_ES/$archivo"
        rm -f "$tmp"
    }
    export -f gen
    while IFS=$'\t' read -r a i t; do printf '%s\0%s\0%s\0' "$a" "$i" "$t"; done < "$OUT/catalogo.tsv" \
        | xargs -0 -n3 -P8 bash -c 'gen "$@"' _
else
    DIR_ES="$CACHE/$PIPER_ES"; DIR_EN="$CACHE/$PIPER_EN"
    mkdir -p "$DIR_ES" "$DIR_EN" "$AQUI/modelos"
    "$VENV/bin/python" -c 'import piper' 2>/dev/null || { [ -x "$VENV/bin/python" ] || uv venv --python 3.12 "$VENV" -q; uv pip install --python "$VENV/bin/python" -q piper-tts; }
    for m in "$PIPER_ES" "$PIPER_EN"; do
        [ -s "$AQUI/modelos/$m.onnx" ] || "$VENV/bin/python" -m piper.download_voices --download-dir "$AQUI/modelos" "$m"
    done
    "$VENV/bin/python" -I "$AQUI/piper_gen.py" "$OUT/catalogo.tsv" "$DIR_ES" "$DIR_EN" \
        "$AQUI/modelos/$PIPER_ES.onnx" "$AQUI/modelos/$PIPER_EN.onnx" "$FILTRO" || true
fi

# la salida queda exactamente con los audios del catálogo, del motor elegido
rm -f "$OUT"/*.ogg
while IFS=$'\t' read -r a i _; do
    d=$DIR_ES; [ "$i" = en ] && d=$DIR_EN
    [ -s "$d/$a" ] && { ln -f "$d/$a" "$OUT/$a" 2>/dev/null || cp "$d/$a" "$OUT/$a"; }
done < "$OUT/catalogo.tsv"
echo "$MOTOR" > "$OUT/.motor"
if [ "$MOTOR" = piper ]; then
    cat > "$OUT/VOCES.txt" <<FIN
Voces generadas con Piper (https://github.com/OHF-Voice/piper1-gpl).
Español: $PIPER_ES   Inglés: $PIPER_EN
Licencias de cada voz: https://huggingface.co/rhasspy/piper-voices (MODEL_CARD de cada una).
FIN
else
    rm -f "$OUT/VOCES.txt"
fi
hechos=$(find "$OUT" -maxdepth 1 -name '*.ogg' | wc -l)
echo "audios: $hechos / $total  ($(du -sh "$OUT" | cut -f1))"
[ "$hechos" -eq "$total" ] || { echo "FALTAN audios, vuelve a ejecutar"; exit 1; }
