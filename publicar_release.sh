#!/usr/bin/env bash
# Arma los archivos de un release y lo sube a GitHub (como BORRADOR, para revisarlo antes de publicar).
# Uso: ./publicar_release.sh [etiqueta]        (por defecto vAAAA.MM.DD)
#      ./publicar_release.sh [etiqueta] --solo-armar   (deja todo en salida/release/ sin subir)
#
# Antes:  ./preparar.sh publico (con voces piper)  →  sudo ./construir_iso.sh  →  ./construir_web.sh
# Sube:   ISO pública, versión web (zip), versión para Linux (tar.gz) y SHA256SUMS.
# Nunca sube voces edge-tts ni perfiles personales.
set -euo pipefail
RAIZ=$(cd "$(dirname "$0")" && pwd)
cd "$RAIZ"
TAG=${1:-v$(date +%Y.%m.%d)}
SOLO_ARMAR=""; [ "${2:-}" = --solo-armar ] && SOLO_ARMAR=1
PAQ=voces/paquete-publico
REL=salida/release

# --- comprobaciones: solo material público ---
[ "$(cat "$PAQ/.motor" 2>/dev/null)" = piper ] || { echo "Las voces de $PAQ no son piper: ejecuta ./preparar.sh publico y elige piper."; exit 1; }
ISO=$(ls -t salida/nidito-publico-*.iso 2>/dev/null | head -1 || true)
[ -n "$ISO" ] || { echo "Falta la ISO pública: ./preparar.sh publico y luego sudo ./construir_iso.sh"; exit 1; }
[ -f salida/nidito-web.zip ] || { echo "Falta la versión web: ./construir_web.sh"; exit 1; }
BIN=escritorio/target/release/nidito
[ -x "$BIN" ] || (cd escritorio && cargo build --release --target-dir target)

rm -rf "$REL"; mkdir -p "$REL"

# --- versión para Linux (sin instalar, en ventana) ---
L=build/linux/nidito-linux-x86_64
rm -rf "$L"; mkdir -p "$L/voces"
install -m755 "$BIN" "$L/nidito"
cp "$PAQ"/*.ogg "$L/voces/"
cp "$PAQ/VOCES.txt" LICENSE enciclopedia/CREDITOS.md escritorio/recursos/OFL.txt "$L/"
cat > "$L/jugar.sh" <<'FIN'
#!/bin/sh
# Abre Nidito en una ventana. Los dibujos y documentos quedan en ~/.local/share/nidito
DIR=$(cd "$(dirname "$0")" && pwd)
export NIDITO_VENTANA=1 NIDITO_VOCES="$DIR/voces" NIDITO_DATOS="${XDG_DATA_HOME:-$HOME/.local/share}/nidito"
unset WAYLAND_DISPLAY   # Nidito usa X11 (en Wayland funciona con XWayland)
exec "$DIR/nidito" "$@"
FIN
chmod 755 "$L/jugar.sh"
cat > "$L/LEEME.txt" <<FIN
Nidito $TAG para Linux (x86_64), sin instalar.

Para jugar:  ./jugar.sh
Necesita X11 o XWayland, ALSA (o PipeWire/PulseAudio con su capa ALSA) y una glibc reciente.
espeak-ng es opcional (se usa para leer palabras que no están grabadas).
Los dibujos y documentos se guardan en ~/.local/share/nidito

Código, ISO y versión web: https://github.com/Debaq/nidito
FIN
tar -C build/linux -czf "$REL/nidito-linux-x86_64.tar.gz" nidito-linux-x86_64

# --- ISO y web ---
cp "$ISO" "$REL/"
cp salida/nidito-web.zip "$REL/"
(cd "$REL" && sha256sum ./* > SHA256SUMS)

cat > "$REL/NOTAS.md" <<FIN
Nidito: un computador para niños pequeños, hecho para darle vida a un netbook viejo.

**Descargas**
- \`$(basename "$ISO")\`: instalable en un computador (BIOS o UEFI en modo legacy, 1 GB de RAM basta). **El instalador borra todo el disco.** Versión genérica: saluda con «¡Hola!»; para una con el nombre de tu hijo o hija, mira «Hacer tu propia ISO» en el README.
- \`nidito-web.zip\`: versión para el navegador (WebAssembly). Descomprime y sirve la carpeta con cualquier servidor web; no funciona abriendo el archivo directamente.
- \`nidito-linux-x86_64.tar.gz\`: para probar en Linux sin instalar (\`./jugar.sh\`).
- \`SHA256SUMS\`: para comprobar las descargas (\`sha256sum -c SHA256SUMS\`).

Voces generadas con Piper ($(sed -n 's/^Español: //p' "$PAQ/VOCES.txt")). Créditos de fotos, sonidos e ilustraciones en CREDITOS.md.
FIN
ls -lh "$REL"

if [ -n "$SOLO_ARMAR" ]; then
    echo "Armado en $REL (sin subir)."
    exit 0
fi
command -v gh > /dev/null || { echo "Falta gh (GitHub CLI): sudo pacman -S github-cli"; exit 1; }
gh release create "$TAG" --draft --title "Nidito $TAG" --notes-file "$REL/NOTAS.md" \
    "$REL/$(basename "$ISO")" "$REL/nidito-web.zip" "$REL/nidito-linux-x86_64.tar.gz" "$REL/SHA256SUMS"
echo "Borrador creado. Revísalo y publícalo en GitHub (Releases) o con: gh release edit $TAG --draft=false"
