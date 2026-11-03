#!/usr/bin/env bash
# Crea (o actualiza) el perfil de un niño: su nombre, apodo, usuario del sistema y minutos de juego.
# Los perfiles quedan en perfiles/<usuario>.conf, que NO se suben a git (datos personales).
#
# Uso interactivo:   ./personalizar.sh
# Uso directo:       ./personalizar.sh "Nombre" "Apodo" [usuario] [minutos_juego]
#
# Después:           ./preparar.sh <usuario>   y luego   sudo ./construir_iso.sh
set -euo pipefail
RAIZ=$(cd "$(dirname "$0")" && pwd)
cd "$RAIZ"

# "José Ñúñez" -> "josenunez": minúsculas, sin tildes ni espacios, empieza con letra.
a_usuario() {
    local u
    u=$(printf '%s' "$1" | iconv -f utf-8 -t ascii//TRANSLIT 2>/dev/null | tr '[:upper:]' '[:lower:]' | tr -cd 'a-z0-9')
    u=${u#"${u%%[a-z]*}"}          # quitar números del comienzo
    printf '%s' "${u:0:16}"
}

if [ $# -ge 1 ]; then
    NOMBRE=$1; APODO=${2:-}; USUARIO=${3:-}; JUEGOS=${4:-20}
else
    echo "== Nuevo perfil de Nidito =="
    read -rp "Nombre del niño o niña (como quieres que lo salude): " NOMBRE
    read -rp "Apodo cariñoso (opcional, Enter para omitir): " APODO
    sugerido=$(a_usuario "$NOMBRE")
    read -rp "Usuario del sistema [${sugerido:-nino}]: " USUARIO
    read -rp "Minutos de juegos por día (0 = sin tope) [20]: " JUEGOS
    JUEGOS=${JUEGOS:-20}
fi

[ -n "${NOMBRE// /}" ] || { echo "Falta el nombre."; exit 1; }
USUARIO=${USUARIO:-$(a_usuario "$NOMBRE")}
USUARIO=${USUARIO:-nino}
[[ "$USUARIO" =~ ^[a-z][a-z0-9]{0,15}$ ]] || { echo "Usuario inválido: '$USUARIO' (solo a-z y 0-9, empieza con letra, máx. 16)."; exit 1; }
case "$USUARIO" in root|bin|daemon|nobody|mail|ftp|http) echo "Usuario reservado: $USUARIO"; exit 1 ;; esac
[[ "$JUEGOS" =~ ^[0-9]+$ ]] || { echo "Minutos inválidos: $JUEGOS"; exit 1; }

mkdir -p perfiles
ARCH="perfiles/$USUARIO.conf"
cat > "$ARCH" <<EOF
# Perfil de Nidito para $NOMBRE (creado con personalizar.sh; no se sube a git)
usuario=$USUARIO
nombre=$NOMBRE
apodo=$APODO
voz=es-419
voz_en=en-us
# minutos de juegos por día (0 = sin tope). Pintar, Letras, Escribir y Mi clase no cuentan.
juegos_min=$JUEGOS
EOF

echo
echo "Perfil guardado en $ARCH:"
grep -v '^#' "$ARCH" | sed 's/^/  /'
echo
echo "Para construir la ISO con este perfil:"
echo "  ./preparar.sh $USUARIO"
echo "  sudo ./construir_iso.sh"
