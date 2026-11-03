#!/usr/bin/env bash
# shellcheck disable=SC2034
iso_name="nidito"
iso_label="NIDITO_$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y%m)"
iso_publisher="Nidito"
iso_application="Nidito - computador para niños"
iso_version="$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y.%m.%d)"
install_dir="arch"
buildmodes=('iso')
bootmodes=('bios.syslinux')
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'zstd' '-Xcompression-level' '15' '-b' '1M')
file_permissions=(
  ["/etc/shadow"]="0:0:400"
  ["/etc/gshadow"]="0:0:400"
  ["/home/nino"]="1000:1000:750"
  ["/home/nino/.bash_profile"]="1000:1000:644"
  ["/usr/local/bin/nidito"]="0:0:755"
  ["/usr/local/bin/nidito-sesion"]="0:0:755"
  ["/usr/local/bin/nidito-instalar"]="0:0:755"
  ["/usr/local/bin/nidito-clave"]="0:0:755"
)
