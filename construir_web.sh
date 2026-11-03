#!/usr/bin/env bash
# Versión web de Nidito (WebAssembly), lista para subir a itch.io como juego HTML.
# Uso: ./construir_web.sh            → build/web/ y salida/nidito-web.zip
# Requisitos (Arch): rust-wasm, wasm-bindgen (misma versión que en escritorio/Cargo.toml), binaryen.
# Las voces salen de voces/paquete-publico (perfil genérico): primero ./preparar.sh publico (con voces piper).
set -euo pipefail
RAIZ=$(cd "$(dirname "$0")" && pwd)
cd "$RAIZ"
PAQ=voces/paquete-publico
OUT=build/web

echo ">> Compilando para WebAssembly..."
(cd escritorio && cargo build --release --target wasm32-unknown-unknown --target-dir target)
(cd escritorio && cargo build --release --target-dir target)

rm -rf "$OUT"; mkdir -p "$OUT"
wasm-bindgen --target web --no-typescript --out-dir "$OUT" escritorio/target/wasm32-unknown-unknown/release/nidito.wasm
echo ">> Optimizando..."
wasm-opt -O3 --enable-bulk-memory --enable-nontrapping-float-to-int "$OUT/nidito_bg.wasm" -o "$OUT/nidito_bg.wasm"
cp web/index.html "$OUT/"

if compgen -G "$PAQ/*.ogg" > /dev/null; then
    motor=$(cat "$PAQ/.motor" 2>/dev/null || echo "?")
    [ "$motor" = piper ] || echo "OJO: las voces de $PAQ son '$motor', no piper: no las publiques (edge-tts es solo para uso personal)."
    escritorio/target/release/nidito --empaquetar-voces "$PAQ" "$OUT/voces.bin"
    [ -f "$PAQ/VOCES.txt" ] && cp "$PAQ/VOCES.txt" "$OUT/"
else
    echo "OJO: no hay voces en $PAQ (./preparar.sh publico): se usará la voz del navegador."
fi

mkdir -p salida
rm -f salida/nidito-web.zip
(cd "$OUT" && python3 -I -m zipfile -c "$RAIZ/salida/nidito-web.zip" ./*)
echo ">> Listo: $OUT ($(du -sh "$OUT" | cut -f1)) y salida/nidito-web.zip"
echo "   Probar:  python3 -m http.server -d $OUT 8000   y abrir http://localhost:8000"
