#!/usr/bin/env bash
# Paso 1 (como usuario normal): compila Nidito, genera las voces y arma el perfil de la ISO en build/perfil.
# Uso: ./preparar.sh [perfil] [--actualizacion [carpeta-del-pendrive]]
#   perfil = nombre de un archivo en perfiles/ (sin .conf), creado con ./personalizar.sh.
#   Sin perfil: si hay uno solo en perfiles/ se usa ese; si no hay ninguno se usa el genérico (iso/nidito.conf).
# Luego: sudo ./construir_iso.sh
#
# Con --actualizacion no se arma la ISO: deja en salida/nidito-actualizacion/ el programa, las voces
# y los scripts, para actualizar un Nidito ya instalado sin reinstalar (menú de adultos → Actualizar).
# Si se indica la carpeta donde está montado el pendrive, se copia ahí directamente.
set -euo pipefail
RAIZ=$(cd "$(dirname "$0")" && pwd)
cd "$RAIZ"

ACTUALIZACION=""; DESTINO=""
args=()
while [ $# -gt 0 ]; do
    case $1 in
        --actualizacion) ACTUALIZACION=1
            if [ $# -ge 2 ] && [ "${2#-}" = "$2" ] && [ -d "$2" ]; then DESTINO=$2; shift; fi ;;
        *) args+=("$1") ;;
    esac
    shift
done
set -- "${args[@]+"${args[@]}"}"

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

if [ -n "$ACTUALIZACION" ]; then
    echo ">> Armando la actualización..."
    ACT=salida/nidito-actualizacion
    rm -rf "$ACT"
    mkdir -p "$ACT/voces" "$ACT/bin"
    install -m755 "$BIN" "$ACT/nidito"
    cp voces/paquete/*.ogg "$ACT/voces/"
    for s in nidito-sesion nidito-clave nidito-actualizar; do
        install -m755 "iso/perfil/airootfs/usr/local/bin/$s" "$ACT/bin/$s"
    done
    cat > "$ACT/LEEME.txt" <<FIN
Actualización de Nidito ($("$BIN" --version))

Copia esta carpeta (nidito-actualizacion) a la raíz de un pendrive (FAT32, exFAT o ext4),
conéctalo al netbook y entra a: menú de adultos (Ctrl+Alt+P) → Actualizar.
Pide la clave de adultos. No toca la clave, la configuración ni los dibujos.

Si el Nidito instalado todavía no tiene el botón "Actualizar":
  menú de adultos → Terminal →
    su                                  (clave de adultos)
    lsblk                               (busca el pendrive, por ejemplo sdb1)
    mkdir -p /run/u && mount /dev/sdb1 /run/u
    bash /run/u/nidito-actualizacion/bin/nidito-actualizar
    umount /run/u; exit; exit
FIN
    echo "   Lista en $ACT ($(du -sh "$ACT" | cut -f1)), $("$BIN" --version)."
    if [ -n "$DESTINO" ]; then
        rm -rf "${DESTINO%/}/nidito-actualizacion"
        cp -r "$ACT" "${DESTINO%/}/" && sync
        echo "   Copiada a ${DESTINO%/}/nidito-actualizacion. Ya puedes sacar el pendrive."
    else
        echo "   Copia la carpeta a la raíz de un pendrive (o: ./preparar.sh ${1:-} --actualizacion /run/media/\$USER/PENDRIVE)."
    fi
    exit 0
fi

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
