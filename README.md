# Nidito

Un computador para niños pequeños, hecho para darle vida a un netbook viejo.

Nidito es un escritorio propio escrito en Rust (sin gestor de ventanas, todo dibujado por CPU) sobre un Arch Linux mínimo, empaquetado como ISO instalable y sin internet. Está pensado para equipos modestos: se desarrolló y probó en un **Samsung NC110** (Intel Atom N455, 1 GB de RAM, pantalla de 1024×600, BIOS sin UEFI). También hay una versión para el navegador.

![Escritorio de Nidito](capturas/escritorio.jpg)

**Índice:** [Qué trae](#qué-trae) · [Descargas](#descargas) · [Instalar en un computador](#instalar-en-un-computador) · [Probar sin instalar](#probar-sin-instalar) · [Hacer tu propia ISO](#hacer-tu-propia-iso) · [Voces](#voces) · [Para desarrollar](#para-desarrollar) · [Licencias](#licencias-y-créditos)

## Qué trae

- **Escritorio con ventanas reales**, barra con botón de Inicio, hora e idioma (español, inglés o los dos).
- **15 apps**: Pintar, Letras, Escribir (tipo WordPad), Contar, Mi clase, Cocina, English, Memorice, Globos, Teclas, Mis dibujos, Enciclopedia, Figuras, Música y Cámara.
- **Figuras**: ocho figuras geométricas con su nombre y cómo son. En el juego ("¿Dónde está el triángulo?") parte con cuatro figuras fáciles y suma las demás a medida que gana estrellas; los colores son al azar para que aprenda la forma y no el color, y si se equivoca dos veces la figura correcta brilla para ayudar.
- **Música**: un xilófono de ocho barras (Do a Do) para tocar con el mouse, arrastrando o con el teclado, y dos canciones tradicionales para seguir (*Estrellita* y *Los pollitos*): la barra que toca brilla, y también se pueden escuchar enteras.
- **Enciclopedia**: temas con fotos reales y textos cortos que lee la voz, todo en una pantalla y sin deslizar: **Animales** (20, con el sonido de 15 de ellos, incluye fauna chilena: pudú, cóndor, pingüino de Humboldt) y **Cuerpo humano** (15 partes, con ilustraciones amables en vez de fotos).
- **Cámara**: se ve en espejo, cuenta 3-2-1 y saca la foto, que queda en "Mis dibujos". Solo aparece si el equipo tiene cámara.
- **Voz**: unas 1200 frases pregrabadas, procesadas para que se escuchen bien en los parlantes chicos de un netbook. Las frases con el nombre del niño o niña se graban al instalar.
- **Control de tiempo**: al encender, un adulto elige la duración de la sesión (20/30/45/60 min o sin límite). Solo cuenta el tiempo activo, y los juegos tienen un tope diario (20 min por defecto). Pintar, Letras, Escribir, Mi clase, Enciclopedia y Figuras no cuentan para ese tope.
- **Menú de adultos** (`Ctrl+Alt+P`, pide la clave de adultos): volumen, +10 minutos, bloquear, **Ajustes** (minutos de juego, pausa para los ojos, cumpleaños), terminal, reiniciar, apagar y **Actualizar** (desde un pendrive, sin reinstalar).
- **Pausa para los ojos**: cada 20 minutos de uso (configurable) aparece una pantalla de 20 segundos que pide mirar algo lejos (regla 20-20-20).
- **Cumpleaños**: ese día el escritorio tiene globos y banderines, saluda con "¡Feliz cumpleaños!" y suena "Cumpleaños feliz".
- **Pantalla de carga** con los gatitos y sin menús de arranque: se enciende y entra directo.
- **Instalador con pantallas**, en español o inglés: pregunta el nombre, el idioma, las voces y el tiempo de juego. Funciona aunque el disco traiga un Windows viejo o dañado.

## Capturas

Todas salen de la prueba automática (`nidito --captura`, en la resolución real del netbook: 1024×600) y del instalador (`nidito --instalador-capturas`).

### Escritorio

| | |
|---|---|
| ![Escritorio con los gatitos](capturas/escritorio.jpg) | ![Menú Inicio](capturas/menu-inicio.jpg) |
| Escritorio con los gatitos | Menú Inicio |
| ![Ventanas que se mueven, minimizan y maximizan](capturas/ventanas.jpg) | ![Apagar](capturas/apagar.jpg) |
| Ventanas que se mueven, minimizan y maximizan | Apagar |

### Apps

| | |
|---|---|
| ![Pintar](capturas/pintar.jpg) | ![Letras: copiar palabras](capturas/letras-copiar.jpg) |
| Pintar | Letras: copiar palabras |
| ![Letras: escribir libre](capturas/letras-libre.jpg) | ![Escribir (tipo WordPad)](capturas/escribir.jpg) |
| Letras: escribir libre | Escribir (tipo WordPad) |
| ![Contar](capturas/contar.jpg) | ![Figuras: mirar](capturas/figuras.jpg) |
| Contar | Figuras: mirar |
| ![Figuras: "¿Dónde está el cuadrado?"](capturas/figuras-jugar.jpg) | ![Música: tocar libre](capturas/musica-libre.jpg) |
| Figuras: "¿Dónde está el cuadrado?" | Música: tocar libre |
| ![Música: siguiendo *Estrellita*](capturas/musica.jpg) | ![Mi clase: armar láminas](capturas/clase.jpg) |
| Música: siguiendo *Estrellita* | Mi clase: armar láminas |
| ![Mi clase: presentar al curso](capturas/clase-presentar.jpg) | ![Cocina: elegir receta](capturas/cocina.jpg) |
| Mi clase: presentar al curso | Cocina: elegir receta |
| ![Cocina: poner los ingredientes](capturas/cocina-poner.jpg) | ![English: tarjetas](capturas/english.jpg) |
| Cocina: poner los ingredientes | English: tarjetas |
| ![English: "Where is the…?"](capturas/english-jugar.jpg) | ![Memorice](capturas/memorice.jpg) |
| English: "Where is the…?" | Memorice |
| ![Globos](capturas/globos.jpg) | ![Teclas](capturas/teclas.jpg) |
| Globos | Teclas |
| ![Enciclopedia: temas](capturas/enciclopedia.jpg) | ![Enciclopedia: animales](capturas/enciclopedia-animales.jpg) |
| Enciclopedia: temas | Enciclopedia: animales |
| ![Enciclopedia: ficha con foto y sonido](capturas/enciclopedia-perro.jpg) | ![Enciclopedia: cuerpo humano](capturas/enciclopedia-corazon.jpg) |
| Enciclopedia: ficha con foto y sonido | Enciclopedia: cuerpo humano |

### Tiempo de uso y adultos

| | |
|---|---|
| ![Al encender: clave de adultos](capturas/bloqueo.jpg) | ![Elegir el tiempo de la sesión](capturas/elegir-tiempo.jpg) |
| Al encender: clave de adultos | Elegir el tiempo de la sesión |
| ![Se acabaron los minutos de juego](capturas/juegos-dormidos.jpg) | ![Pausa para los ojos](capturas/pausa-ojos.jpg) |
| Se acabaron los minutos de juego | Pausa para los ojos |
| ![Menú de adultos (Ctrl+Alt+P)](capturas/padres.jpg) | ![Ajustes](capturas/ajustes.jpg) |
| Menú de adultos (Ctrl+Alt+P) | Ajustes |
| ![Día de cumpleaños](capturas/cumpleanos.jpg) | ![Pantalla de carga](capturas/arranque.jpg) |
| Día de cumpleaños | Pantalla de carga |

### Instalador

| | |
|---|---|
| ![Lo primero: el idioma](capturas/instalador-idioma.jpg) | ![Bienvenida](capturas/instalador-bienvenida.jpg) |
| Lo primero: el idioma | Bienvenida |
| ![Confirmar el disco (en inglés)](capturas/installer-erase.jpg) | ![Nombre y apodo](capturas/instalador-nombre.jpg) |
| Confirmar el disco (en inglés) | Nombre y apodo |
| ![Voces (con E se escuchan)](capturas/instalador-voces.jpg) | ![Tiempo de juego y cumpleaños](capturas/instalador-tiempo.jpg) |
| Voces (con E se escuchan) | Tiempo de juego y cumpleaños |
| ![Instalando](capturas/instalador-avance.jpg) | ![¡Listo!](capturas/instalador-listo.jpg) |
| Instalando | ¡Listo! |

## Descargas

En **[Releases](https://github.com/Debaq/nidito/releases)** hay tres archivos:

| Archivo | Para qué |
|---|---|
| `nidito-publico-AAAA.MM.DD-x86_64.iso` | Instalar Nidito en un computador. Se graba en un pendrive. |
| `nidito-web.zip` | La versión para el navegador (WebAssembly). |
| `nidito-linux-x86_64.tar.gz` | Probar en Linux, en una ventana, sin instalar nada. |
| `SHA256SUMS` | Para comprobar que las descargas llegaron bien (`sha256sum -c SHA256SUMS`). |

## Instalar en un computador

> ⚠️ **El instalador borra todo el disco del computador** (fotos, documentos, Windows). Úsalo en un equipo que se pueda borrar.

**Requisitos:** un computador x86 de 64 bits con 1 GB de RAM o más, que arranque en modo BIOS o "Legacy" (los UEFI modernos suelen tener esa opción en la configuración), y un pendrive de 2 GB o más.

1. **Descarga la ISO** desde [Releases](https://github.com/Debaq/nidito/releases).
2. **Grábala en el pendrive.** Sirve cualquier sistema:
   - **Windows, Mac o Linux:** [balenaEtcher](https://etcher.balena.io/) → *Flash from file* → elige la ISO → elige el pendrive → *Flash*.
   - **Windows:** [Rufus](https://rufus.ie/) → elige el pendrive y la ISO → si pregunta, modo **DD** → *Empezar*.
   - **Linux, en terminal** (revisa la letra con `lsblk`: `dd` borra el disco que indiques):
     ```bash
     sudo dd if=nidito-publico-*.iso of=/dev/sdX bs=4M status=progress conv=fsync
     ```
3. **Arranca el computador desde el pendrive.** Suele ser con una tecla al encender (`F12`, `Esc`, `F9` o `F2`; en el Samsung NC110 es `Esc`).
4. Elige **"Instalar Nidito"**. El instalador pregunta, en este orden:
   1. el idioma del instalador (español o inglés),
   2. el teclado,
   3. el disco (para confirmar hay que escribir **BORRAR**, o **ERASE** si el instalador está en inglés),
   4. el nombre y apodo del niño o niña (opcional: sin nombre saluda con "¡Hola!"),
   5. el idioma de Nidito (español, inglés o los dos),
   6. las voces (mujer u hombre, en español y en inglés; con **E** se escuchan),
   7. los minutos de juego, la pausa para los ojos y el cumpleaños,
   8. la **clave de adultos**.

   Después instala solo (unos 10 minutos) y graba con su voz las frases que llevan el nombre.
5. Saca el pendrive y presiona Enter. El computador entra directo a Nidito.

La clave de adultos se pide al encender (para elegir el tiempo de uso) y en el menú de adultos (`Ctrl+Alt+P`). También es la clave de `root`. Para cambiarla: menú de adultos → Terminal → `su -c nidito-clave`.

En vez de "Instalar" también se puede elegir **"Probar Nidito"**, que lo abre desde el pendrive sin tocar el disco.

### Actualizar sin reinstalar

Para llevar una versión nueva (programa, voces y scripts) a un Nidito ya instalado: copia la carpeta `nidito-actualizacion` (ver [Para desarrollar](#actualizar-un-nidito-instalado)) a la raíz de un pendrive (FAT32, exFAT o ext4; no el de la ISO), conéctalo y entra a **menú de adultos → Actualizar**. No toca la clave, la configuración ni los dibujos; el programa anterior queda guardado y se puede volver a él.

### Si el volumen queda bajo

En el menú de adultos → Terminal:

```bash
amixer sset PCM 100% unmute
amixer sset Speaker 100% unmute
amixer sset Master 100% unmute
```

## Probar sin instalar

- **En el navegador:** descomprime `nidito-web.zip` y sírvelo con cualquier servidor web (abrir el `index.html` directo no funciona):
  ```bash
  python3 -m http.server -d nidito-web 8000     # y abre http://localhost:8000
  ```
  Hay que tocar "Tocar para empezar" (los navegadores no dejan sonar audio antes). Los dibujos y documentos se guardan en el navegador. No trae cámara, control de tiempo ni menú de adultos. Funciona con pantallas táctiles.
- **En Linux:** descomprime `nidito-linux-x86_64.tar.gz` y ejecuta `./jugar.sh`. Se abre en una ventana; los dibujos quedan en `~/.local/share/nidito`. Necesita X11 o XWayland y una glibc reciente.
- **Desde el pendrive:** opción "Probar Nidito" del menú de arranque.

## Hacer tu propia ISO

La ISO de Releases ya sirve para cualquier niño o niña: el nombre, el idioma y las voces se eligen al instalar. Hacer una ISO propia solo hace falta para cambiar algo del código o para tener una ISO que ya traiga todo configurado.

Se construye en **Arch Linux** (o un derivado: EndeavourOS, CachyOS, Manjaro...). Requisitos:

```bash
sudo pacman -S --needed archiso rust clang ffmpeg uv
```

### Con la ventana (más fácil)

```bash
git clone https://github.com/Debaq/nidito && cd nidito
python3 crear_iso.py
```

Se llenan los datos (nombre, apodo, minutos de juego, pausa para los ojos, cumpleaños), se eligen las voces y se presiona **Crear ISO**. Al final pide la clave de administrador del computador (para `mkarchiso`). La ISO queda en `salida/`. Con "Versión pública" arma la ISO genérica, igual a la de Releases.

### Con los scripts

```bash
./personalizar.sh                  # pregunta nombre, apodo, usuario, minutos de juego, cumpleaños y pausa
./preparar.sh <usuario>            # compila, genera las voces y arma build/perfil
sudo ./construir_iso.sh            # crea salida/nidito-AAAA.MM.DD-x86_64.iso
```

- `./personalizar.sh "Sofía" "Sofi"` hace lo mismo sin preguntas (ver el comienzo del script para el resto de opciones).
- Los perfiles quedan en `perfiles/<usuario>.conf`, que **no se sube a git** (datos personales). Puedes tener varios.
- `./preparar.sh publico` arma la **versión pública** (sin nombre, con las cuatro voces para elegir al instalar). Sus voces e ISO van aparte (`voces/paquete-publico*`, `salida/nidito-publico-*.iso`), así que no pisan las de un perfil personal.
- `./probar_vm.sh` arranca la ISO en QEMU simulando el NC110 (BIOS, 1 GB, 1024×600); `./probar_vm.sh disco` arranca lo que se instaló en el disco virtual.

## Voces

`preparar.sh` pregunta con qué motor generar las voces:

| Motor | Cómo suena | Internet | Se puede compartir |
|---|---|---|---|
| **piper** (por defecto) | bien | no | **sí**: voces con licencias libres |
| **edge** (edge-tts) | muy natural | sí | **no**: son voces de Microsoft, solo para uso personal |

Para no preguntar: `NIDITO_MOTOR=piper` o `NIDITO_MOTOR=edge`. Las voces se guardan por motor en `voces/cache/`, así que cambiar de motor no obliga a regenerar todo.

Voces de Piper que trae la versión pública:

| | Mujer | Hombre |
|---|---|---|
| Español | `es_MX-claude-high` (Apache-2.0) | `es_MX-ald-medium` (Unlicense) |
| Inglés | `en_US-ljspeech-high` (dominio público) | `en_US-joe-medium` (CC0) |

Para generar con otras voces de [piper-voices](https://huggingface.co/rhasspy/piper-voices) en una ISO personal: `NIDITO_PIPER_ES=es_AR-daniela-high ./preparar.sh <usuario>` (revisa la licencia de cada voz en su `MODEL_CARD`).

Al instalar, el computador genera con Piper las ~10 frases que llevan el nombre (saludo, cumpleaños, fin de la clase...). Todo lo demás viene pregrabado.

## Para desarrollar

Requisitos: los de [Hacer tu propia ISO](#hacer-tu-propia-iso). Para la versión web, además: `sudo pacman -S rust-wasm wasm-bindgen binaryen` (la versión de `wasm-bindgen` tiene que coincidir con la que fija `escritorio/Cargo.toml`).

> Compila siempre con `--target-dir target` (como hacen los scripts) si tienes `CARGO_TARGET_DIR` configurado: si no, el binario no queda en `escritorio/target/`.

```bash
cd escritorio && cargo build --release --target-dir target && cd ..

# El escritorio en una ventana
HOME=/tmp/nidito-demo NIDITO_VENTANA=1 NIDITO_VOCES=$PWD/voces/paquete-publico \
  WAYLAND_DISPLAY= ./escritorio/target/release/nidito

# Prueba automática: simula uso, guarda capturas PNG y revisa que todas las frases estén en el catálogo
./escritorio/target/release/nidito --captura /tmp/cap

# El instalador en la terminal, sin tocar ningún disco
./escritorio/target/release/nidito --instalador --simular

# La versión web
./construir_web.sh                 # build/web y salida/nidito-web.zip
python3 -m http.server -d build/web 8000
```

Otros comandos de `nidito`: `--frases` (catálogo de frases), `--arranque <dir>` (imágenes de la pantalla de carga), `--empaquetar-voces` (voces para la web), `--voces-nombre` (lo usa el instalador), `--instalador-capturas <dir>` (las pantallas del instalador en texto), `--version`.

### Actualizar un Nidito instalado

```bash
./preparar.sh [perfil] --actualizacion                       # deja salida/nidito-actualizacion/
./preparar.sh [perfil] --actualizacion /run/media/$USER/USB  # o la copia directo al pendrive montado
```

Si el equipo se instaló con una ISO que aún no tiene el botón "Actualizar", la primera vez se hace a mano (menú de adultos → Terminal): `su`, montar el pendrive (`mount /dev/sdb1 /run/u`) y `bash /run/u/nidito-actualizacion/bin/nidito-actualizar`. Las instrucciones vienen en `LEEME.txt`.

### Publicar un release

```bash
NIDITO_MOTOR=piper ./preparar.sh publico
sudo ./construir_iso.sh
./construir_web.sh
./publicar_release.sh v2026.11.03          # sube un BORRADOR a GitHub para revisarlo
```

`publicar_release.sh` se niega a publicar si las voces no son de Piper.

### Estructura

| Ruta | Qué es |
|------|--------|
| `escritorio/` | El escritorio, las apps y el instalador (Rust: winit + softbuffer + tiny-skia + rodio + ratatui) |
| `escritorio/src/apps/` | Una app por archivo |
| `iso/perfil/` | Perfil de archiso: paquetes, servicios, sesión, pantalla de carga, `nidito-instalar` (la parte de disco del instalador) |
| `iso/nidito.conf` | Configuración genérica (se instala en `/etc/nidito.conf`) |
| `web/` | Página de la versión web |
| `voces/` | Generador de voces (`generar.sh`, `piper_gen.py`) y filtro de audio (`filtro.txt`) |
| `enciclopedia/` | Fotos, sonidos e ilustraciones de la enciclopedia, con scripts para bajarlos y sus créditos |
| `capturas/` | Imágenes del README |
| `crear_iso.py` | Ventana para crear la ISO |
| `personalizar.sh`, `preparar.sh`, `construir_iso.sh` | Construcción de la ISO (y del paquete de actualización) |
| `construir_web.sh`, `publicar_release.sh`, `probar_vm.sh` | Versión web, releases y prueba en QEMU |

## Licencias y créditos

- **Código:** [MIT](LICENSE).
- **Fuente:** Andika, [SIL Open Font License](escritorio/recursos/OFL.txt).
- **Fotos y sonidos** de la enciclopedia: Wikimedia Commons. **Ilustraciones:** Twemoji (CC BY 4.0). Autor y licencia de cada uno en [enciclopedia/CREDITOS.md](enciclopedia/CREDITOS.md).
- **Voces:** generadas con [Piper](https://github.com/rhasspy/piper) (MIT), con voces de [piper-voices](https://huggingface.co/rhasspy/piper-voices): ver la tabla de [Voces](#voces). La ISO incluye Piper (con onnxruntime y espeak-ng) para grabar las frases con el nombre al instalar. Las voces no se guardan en el repositorio: se generan al construir.
