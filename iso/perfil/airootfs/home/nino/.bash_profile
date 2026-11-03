# Al entrar en la consola 1 se abre Nidito
if [ -z "$DISPLAY" ] && [ "$(tty)" = "/dev/tty1" ]; then
    exec startx /usr/local/bin/nidito-sesion -- -keeptty -nolisten tcp > "$HOME/.xorg-nidito.log" 2>&1
fi
