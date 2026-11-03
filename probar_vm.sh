#!/usr/bin/env bash
# Prueba la ISO en una máquina virtual parecida al NC110: BIOS, 1 GB de RAM, 1 núcleo,
# CPU x86_64 básica (qemu64, sin SSE4/AVX) y pantalla 1024x600.
# Uso: ./probar_vm.sh            → arranca desde la ISO (probar o instalar)
#      ./probar_vm.sh disco      → arranca el disco ya instalado
set -euo pipefail
RAIZ=$(cd "$(dirname "$0")" && pwd)
ISO=$(ls -t "$RAIZ"/salida/*.iso 2>/dev/null | head -1 || true)
DISCO="$RAIZ/build/vm-disco.qcow2"
[ -f "$DISCO" ] || qemu-img create -f qcow2 "$DISCO" 20G >/dev/null
ACEL=()
[ -w /dev/kvm ] && ACEL=(-enable-kvm)
ARR=(-boot d -cdrom "$ISO")
if [ "${1:-}" = "disco" ]; then ARR=(-boot c); else [ -n "$ISO" ] || { echo "No hay ISO en salida/"; exit 1; }; fi
exec qemu-system-x86_64 "${ACEL[@]}" -cpu qemu64 -smp 1 -m 1024 \
    -drive file="$DISCO",if=ide,format=qcow2 "${ARR[@]}" \
    -device VGA,xres=1024,yres=600 \
    -audiodev pipewire,id=snd0 -device intel-hda -device hda-duplex,audiodev=snd0 \
    -usb -device usb-tablet \
    -name "Nidito (NC110 simulado)"
