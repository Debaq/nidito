#!/usr/bin/env bash
# Baja ilustraciones de Twemoji (CC BY 4.0) y las convierte en fotos de la enciclopedia:
#   fotos/<id>.jpg (480x430) y fotos/<id>-mini.jpg (188x104), dibujo centrado sobre un fondo de color.
# Uso: ./bajar_dibujos.sh [cuerpo.tsv]   (requiere curl, rsvg-convert e ImageMagick)
set -euo pipefail
cd "$(dirname "$0")"
LISTA=${1:-cuerpo.tsv}
VER=v15.1.0
mkdir -p fotos
CRED=CREDITOS.md
grep -q '^## Ilustraciones' "$CRED" || printf '\n## Ilustraciones\n\nIlustraciones de [Twemoji](https://github.com/jdecked/twemoji) %s, © Twitter, Inc. y colaboradores, licencia [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Se usan centradas sobre un fondo de color.\n\n| Ilustración | Código |\n|---|---|\n' "$VER" >> "$CRED"
grep -v '^#' "$LISTA" | while IFS=$'\t' read -r id cod fondo; do
    [ -n "$id" ] || continue
    if [ -s "fotos/$id.jpg" ]; then echo "= $id (ya está)"; continue; fi
    tmp=$(mktemp -d)
    curl -sf -o "$tmp/e.svg" "https://cdn.jsdelivr.net/gh/jdecked/twemoji@$VER/assets/svg/$cod.svg" </dev/null
    rsvg-convert -w 330 -h 330 "$tmp/e.svg" -o "$tmp/g.png"
    rsvg-convert -w 66 -h 66 "$tmp/e.svg" -o "$tmp/m.png"
    magick -size 480x430 "xc:$fondo" "$tmp/g.png" -gravity center -composite -quality 90 "fotos/$id.jpg" </dev/null
    magick -size 188x104 "xc:$fondo" "$tmp/m.png" -gravity north -geometry +0+6 -composite -quality 90 "fotos/$id-mini.jpg" </dev/null
    rm -rf "$tmp"
    sed -i "/^| $id |/d" "$CRED"
    echo "| $id | $cod |" >> "$CRED"
    echo "+ $id ($cod)"
done
