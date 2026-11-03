#!/usr/bin/env bash
# Paso 1 (como usuario normal): compila Nidito, genera las voces y arma el perfil de la ISO en build/perfil.
# Uso: ./preparar.sh [perfil]
#   perfil = nombre de un archivo en perfiles/ (sin .conf), creado con ./personalizar.sh.
#   Sin perfil: si hay uno solo en perfiles/ se usa ese; si no hay ninguno se usa el genérico (iso/nidito.conf).
# Luego: sudo ./construir_iso.sh
set -euo pipefail
RAIZ=$(cd "$(dirname "$0")" && pwd)
cd "$RAIZ"

# --- elegir perfil ---
if [ $# -ge 1 ]; then
    PERFIL="perfiles/${1%.conf}.conf"
    [ -f "$PERFIL" ] || { echo "No existe $PERFIL. Créalo con ./personalizar.sh"; exit 1; }
else
    mapfile -t hay < <(ls perfiles/*.conf 2>/dev/null || true)
    if [ ${#hay[@]} -eq 1 ]; then
        PERFIL=${hay[0]}
    elif [ ${#hay[@]} -gt 1 ]; then
        echo "Hay varios perfiles; indica cuál:"
        for p in "${hay[@]}"; do echo "  ./preparar.sh $(basename "$p" .conf)"; done
        exit 1
    else
        PERFIL=iso/nidito.conf
        echo ">> Sin perfiles en perfiles/: se usa el genérico (sin nombre). Crea uno con ./personalizar.sh"
    fi
fi
USUARIO=$(sed -n 's/^usuario=//p' "$PERFIL" | head -1)
USUARIO=${USUARIO:-nino}
NOMBRE=$(sed -n 's/^nombre=//p' "$PERFIL" | head -1)
echo ">> Perfil: $PERFIL (usuario '$USUARIO'${NOMBRE:+, nombre '$NOMBRE'})"
CONF=$(mktemp); trap 'rm -f "$CONF"' EXIT
grep -v '^usuario=' "$PERFIL" > "$CONF"

echo ">> Compilando Nidito (release, x86_64 genérico: corre en el Atom)..."
(cd escritorio && cargo build --release --target-dir target)
BIN=escritorio/target/release/nidito

echo ">> Generando voces (edge-tts, solo frases que falten)..."
voces/generar.sh "$BIN" "$CONF" voces/paquete

echo ">> Armando el perfil..."
rm -rf build/perfil
mkdir -p build
cp -a iso/perfil build/perfil
A=build/perfil/airootfs
install -Dm755 "$BIN" "$A/usr/local/bin/nidito"
install -Dm644 "$CONF" "$A/etc/nidito.conf"
mkdir -p "$A/usr/share/nidito/voces"
cp voces/paquete/*.ogg "$A/usr/share/nidito/voces/"
chmod 755 "$A/usr/local/bin/"*

# El perfil base usa el usuario genérico 'nino'; se renombra al del perfil.
if [ "$USUARIO" != nino ]; then
    GECOS=$(printf '%s' "${NOMBRE:-$USUARIO}" | tr -d ':,|&\\')
    sed -i "s|^nino:x:1000:1000:[^:]*:/home/nino:|$USUARIO:x:1000:1000:$GECOS:/home/$USUARIO:|" "$A/etc/passwd"
    sed -i "s/^nino:/$USUARIO:/" "$A/etc/group" "$A/etc/gshadow" "$A/etc/shadow"
    sed -i "s/--autologin nino /--autologin $USUARIO /" "$A/etc/systemd/system/getty@tty1.service.d/autologin.conf"
    sed -i "s|/home/nino|/home/$USUARIO|g" build/perfil/profiledef.sh
    mv "$A/home/nino" "$A/home/$USUARIO"
fi

echo ">> Perfil listo en build/perfil ($(du -sh build/perfil | cut -f1))."
echo "   Ahora: sudo ./construir_iso.sh"
