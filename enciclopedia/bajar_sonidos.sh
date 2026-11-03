#!/usr/bin/env bash
# Baja los sonidos de animales desde Wikimedia Commons (licencias libres), toma el tramo
# más fuerte (máx. N segundos), lo normaliza y lo guarda en sonidos/<id>.ogg.
# Agrega autor y licencia a CREDITOS.md.  Uso: ./bajar_sonidos.sh [sonidos.tsv]
set -euo pipefail
cd "$(dirname "$0")"
LISTA=${1:-sonidos.tsv}
UA="Nidito/0.1 (https://github.com/Debaq/nidito)"
mkdir -p sonidos
CRED=CREDITOS.md
grep -q '^## Sonidos' "$CRED" || printf '\n## Sonidos\n\n| Sonido | Archivo original | Autor | Licencia |\n|---|---|---|---|\n' >> "$CRED"
grep -v '^#' "$LISTA" | while IFS=$'\t' read -r id archivo seg; do
    [ -n "$id" ] || continue
    if [ -s "sonidos/$id.ogg" ]; then echo "= $id (ya está)"; continue; fi
    info=$(curl -sfG -A "$UA" "https://commons.wikimedia.org/w/api.php" --data-urlencode "titles=File:$archivo" -d action=query -d prop=imageinfo -d iiprop='url|extmetadata' -d format=json)
    url=$(jq -r '.query.pages[].imageinfo[0].url // empty' <<<"$info")
    [ -n "$url" ] || { echo "! $id: no existe '$archivo'"; continue; }
    pag=$(jq -r '.query.pages[].imageinfo[0].descriptionurl' <<<"$info")
    autor=$(jq -r '.query.pages[].imageinfo[0].extmetadata.Artist.value // "?"' <<<"$info" | sed 's/<[^>]*>//g' | tr -s ' \n' ' ' | sed 's/^ //;s/ $//' | cut -c1-80)
    lic=$(jq -r '.query.pages[].imageinfo[0].extmetadata.LicenseShortName.value // "?"' <<<"$info")
    tmp=$(mktemp -d)
    curl -sf -A "$UA" -o "$tmp/orig" "$url"
    dur=${seg:-6}
    ini=$(python3 -I tramo.py "$tmp/orig" "$dur")
    fin=$(python3 -c "print(max(0.05, $dur - 0.3))")
    ffmpeg -nostdin -loglevel error -y -ss "$ini" -t "$dur" -i "$tmp/orig" \
        -af "highpass=f=60,afade=t=in:d=0.05,afade=t=out:st=$fin:d=0.3,loudnorm=I=-14:TP=-1.5,alimiter=limit=0.89:level=false" \
        -ac 1 -ar 22050 -c:a libvorbis -q:a 4 "sonidos/$id.ogg"
    rm -rf "$tmp"
    sed -i "/^| son-$id |/d" "$CRED"
    echo "| son-$id | [${archivo//|/-}]($pag) | ${autor//|/-} | $lic |" >> "$CRED"
    echo "+ $id: desde ${ini}s, ${dur}s ($lic)"
    sleep 1
done
