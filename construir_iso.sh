#!/usr/bin/env bash
# Paso 2 (como root): construye la ISO con mkarchiso a partir de build/perfil.
# Uso: sudo ./construir_iso.sh      → salida/nidito-AAAA.MM.DD-x86_64.iso
set -euo pipefail
RAIZ=$(cd "$(dirname "$0")" && pwd)
[ "$EUID" -eq 0 ] || { echo "Ejecuta con sudo: sudo ./construir_iso.sh"; exit 1; }
command -v mkarchiso > /dev/null || { echo "Falta archiso: sudo pacman -S archiso"; exit 1; }
[ -x "$RAIZ/build/perfil/airootfs/usr/local/bin/nidito" ] || { echo "Primero ejecuta ./preparar.sh (sin sudo)"; exit 1; }
TRABAJO=/var/tmp/nidito-iso
rm -rf "$TRABAJO"
mkdir -p "$RAIZ/salida"
mkarchiso -v -r -w "$TRABAJO" -o "$RAIZ/salida" "$RAIZ/build/perfil"
# devolver la ISO al usuario que la pidió (sudo o pkexec, p. ej. desde crear_iso.py)
if [ -n "${SUDO_USER:-}" ]; then chown -R "$SUDO_USER": "$RAIZ/salida"
elif [ -n "${PKEXEC_UID:-}" ]; then chown -R "$PKEXEC_UID": "$RAIZ/salida"; fi
ls -lh "$RAIZ/salida"
echo
echo "Para grabarla en un pendrive (OJO: borra el pendrive; revisa la letra con lsblk):"
echo "  sudo dd if=$(ls -t "$RAIZ"/salida/*.iso | head -1) of=/dev/sdX bs=4M status=progress conv=fsync"
