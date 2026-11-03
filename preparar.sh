#!/usr/bin/env bash
# Paso 1 (como usuario normal): compila Nidito, genera las voces y arma el perfil de la ISO en build/perfil.
# Uso: ./preparar.sh [perfil] [--actualizacion [carpeta-del-pendrive]]
#   perfil = nombre de un archivo en perfiles/ (sin .conf), creado con ./personalizar.sh.
#   Sin perfil: si hay uno solo en perfiles/ se usa ese; si no hay ninguno se usa el genérico (iso/nidito.conf).
#   perfil = publico: versión para compartir (perfil genérico, voces en voces/paquete-publico, ISO nidito-publico-*.iso).
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
if [ "${1:-}" = publico ]; then
    PERFIL=iso/nidito.conf
elif [ $# -ge 1 ]; then
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
# El perfil genérico es la versión pública: sus voces y su ISO van aparte, para no tocar las de un perfil personal.
if [ "$PERFIL" = iso/nidito.conf ]; then
    PAQ=voces/paquete-publico; ISO_NOMBRE=nidito-publico
else
    PAQ=voces/paquete; ISO_NOMBRE=nidito
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

echo ">> Generando voces (piper o edge-tts, solo frases que falten)..."
# Versión pública: voces de Piper para elegir al instalar (mujer u hombre, en cada idioma).
# Tienen que ser las mismas que VOCES_ES / VOCES_EN en escritorio/src/instalador.rs.
VOCES_MUJER=(es_MX-claude-high en_US-ljspeech-high)
VOCES_HOMBRE=(es_MX-ald-medium en_US-joe-medium)
if [ "$PAQ" = voces/paquete-publico ]; then
    export NIDITO_PIPER_ES=${VOCES_MUJER[0]} NIDITO_PIPER_EN=${VOCES_MUJER[1]}
fi
voces/generar.sh "$BIN" "$CONF" "$PAQ"
CON_PIPER=""
if [ "$PAQ" = voces/paquete-publico ] && [ "$(cat "$PAQ/.motor" 2>/dev/null)" = piper ]; then
    CON_PIPER=1
    echo ">> Voces de hombre (para elegir al instalar)..."
    NIDITO_MOTOR=piper NIDITO_PIPER_ES=${VOCES_HOMBRE[0]} NIDITO_PIPER_EN=${VOCES_HOMBRE[1]} \
        voces/generar.sh "$BIN" "$CONF" voces/paquete-publico-m
    # Piper para el computador (genera las frases con el nombre al instalar). Versión C++ (MIT):
    # trae onnxruntime y espeak-ng y funciona en procesadores viejos (probado sin SSE4.1).
    PIPER_URL=https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_linux_x86_64.tar.gz
    PIPER_SHA=a50cb45f355b7af1f6d758c1b360717877ba0a398cc8cbe6d2a7a3a26e225992
    if [ ! -x voces/piper-bin/piper/piper ]; then
        echo ">> Descargando Piper..."
        mkdir -p voces/piper-bin
        curl -fL --retry 3 -o voces/piper-bin/piper.tar.gz "$PIPER_URL"
        echo "$PIPER_SHA  voces/piper-bin/piper.tar.gz" | sha256sum -c --quiet || { echo "La descarga de Piper no coincide (sha256)."; exit 1; }
        tar -C voces/piper-bin -xzf voces/piper-bin/piper.tar.gz
    fi
fi

if [ -n "$ACTUALIZACION" ]; then
    echo ">> Armando la actualización..."
    ACT=salida/nidito-actualizacion
    rm -rf "$ACT"
    mkdir -p "$ACT/voces" "$ACT/bin"
    install -m755 "$BIN" "$ACT/nidito"
    cp "$PAQ"/*.ogg "$ACT/voces/"
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
cp "$PAQ"/*.ogg "$A/usr/share/nidito/voces/"
[ -f "$PAQ/VOCES.txt" ] && cp "$PAQ/VOCES.txt" "$A/usr/share/nidito/voces/"
sed -i "s/^iso_name=.*/iso_name=\"$ISO_NOMBRE\"/" build/perfil/profiledef.sh
# pantalla de carga (plymouth): imágenes dibujadas por Nidito
"$BIN" --arranque "$A/usr/share/plymouth/themes/nidito"
if [ -n "$CON_PIPER" ]; then
    N=$A/usr/share/nidito
    mkdir -p "$N/modelos"
    for par in "voces/paquete-publico ${VOCES_MUJER[*]}" "voces/paquete-publico-m ${VOCES_HOMBRE[*]}"; do
        read -r dir es en <<< "$par"
        mkdir -p "$N/voces-$es" "$N/voces-$en"
        cp "$dir"/es-*.ogg "$N/voces-$es/"
        cp "$dir"/en-*.ogg "$N/voces-$en/"
        for m in "$es" "$en"; do cp "voces/modelos/$m.onnx" "voces/modelos/$m.onnx.json" "$N/modelos/"; done
    done
    cp -a voces/piper-bin/piper "$N/piper"
    rm -f "$N/piper/libtashkeel_model.ort"   # solo para árabe
    cat > "$N/voces/VOCES.txt" <<FIN
Voces generadas con Piper (https://github.com/rhasspy/piper, MIT), voces de https://huggingface.co/rhasspy/piper-voices
Mujer:  ${VOCES_MUJER[0]} (Apache-2.0), ${VOCES_MUJER[1]} (dominio público)
Hombre: ${VOCES_HOMBRE[0]} (Unlicense), ${VOCES_HOMBRE[1]} (CC0)
FIN
fi
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

echo ">> Perfil listo en build/perfil ($(du -sh build/perfil | cut -f1)), ISO: salida/$ISO_NOMBRE-<fecha>-x86_64.iso"
echo "   Ahora: sudo ./construir_iso.sh"
