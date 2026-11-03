#!/usr/bin/env bash
# Baja las fotos de la enciclopedia desde Wikimedia Commons (licencias libres) y las recorta:
#   fotos/<id>.jpg  480x430 (página del animal)     fotos/<id>-mini.jpg  188x104 (grilla)
# y escribe CREDITOS.md con autor y licencia de cada foto.
# Uso: ./bajar_fotos.sh [lista.tsv]   (por defecto animales.tsv; requiere curl, jq, ImageMagick)
set -euo pipefail
cd "$(dirname "$0")"
LISTA=${1:-animales.tsv}
UA="Nidito/0.1 (https://github.com/Debaq/nidito)"
mkdir -p fotos
api() { curl -sfG -A "$UA" "$@"; }
CRED=CREDITOS.md
[ -f "$CRED" ] || printf '# Créditos de las fotos\n\nTodas las fotos vienen de Wikimedia Commons y se usan según su licencia (recortadas y reducidas).\n\n| Foto | Archivo original | Autor | Licencia |\n|---|---|---|---|\n' > "$CRED"
grep -v '^#' "$LISTA" | while IFS=$'\t' read -r id titulo archivo grav; do
    [ -n "$id" ] || continue
    if [ -s "fotos/$id.jpg" ]; then echo "= $id (ya está)"; continue; fi
    if [ -z "${archivo:-}" ]; then
        archivo=$(api "https://en.wikipedia.org/w/api.php" --data-urlencode "titles=$titulo" -d action=query -d prop=pageimages -d piprop=name -d format=json | jq -r '.query.pages[].pageimage // empty')
    fi
    [ -n "$archivo" ] || { echo "! $id: sin foto para '$titulo'"; continue; }
    info=$(api "https://commons.wikimedia.org/w/api.php" --data-urlencode "titles=File:$archivo" -d action=query -d prop=imageinfo -d iiprop='url|extmetadata' -d iiurlwidth=1200 -d format=json)
    url=$(jq -r '.query.pages[].imageinfo[0].thumburl' <<<"$info")
    pag=$(jq -r '.query.pages[].imageinfo[0].descriptionurl' <<<"$info")
    autor=$(jq -r '.query.pages[].imageinfo[0].extmetadata.Artist.value // "?"' <<<"$info" | sed 's/<[^>]*>//g' | tr -s ' \n' ' ' | sed 's/^ //;s/ $//' | cut -c1-80)
    lic=$(jq -r '.query.pages[].imageinfo[0].extmetadata.LicenseShortName.value // "?"' <<<"$info")
    tmp=$(mktemp --suffix=.img)
    curl -sf -A "$UA" -o "$tmp" "$url"
    magick "$tmp" -auto-orient -resize '480x430^' -gravity "${grav:-center}" -extent 480x430 -strip -quality 84 "fotos/$id.jpg"
    magick "$tmp" -auto-orient -resize '188x104^' -gravity "${grav:-center}" -extent 188x104 -strip -quality 85 "fotos/$id-mini.jpg"
    rm -f "$tmp"
    sed -i "/^| $id |/d" "$CRED"
    echo "| $id | [$archivo]($pag) | ${autor//|/-} | $lic |" >> "$CRED"
    echo "+ $id: $archivo ($lic)"
    sleep 1
done
