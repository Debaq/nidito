//! Enciclopedia: temas (por ahora Animales) con fotos y textos cortos que lee la voz.
//! Todo cabe en pantalla, sin deslizar. Fotos de Wikimedia Commons (ver enciclopedia/CREDITOS.md).

use super::*;
use crate::cx::Son;
use tiny_skia::{FilterQuality, IntSize, Pixmap, PixmapPaint, Transform};

pub struct Ficha {
    pub nombre: &'static str,
    pub texto: [&'static str; 3],
    foto: &'static [u8],
    mini: &'static [u8],
    sonido: Option<&'static [u8]>,
}

pub struct Tema {
    pub nombre: &'static str,
    pub color: C,
    pub fichas: &'static [Ficha],
}

/// ficha!("id", "Nombre", "frase 1", "frase 2", "frase 3") usa enciclopedia/fotos/id.jpg e id-mini.jpg.
/// Con `@son` delante, además enciclopedia/sonidos/id.ogg.
macro_rules! ficha {
    ($id:literal, $nombre:literal, $a:literal, $b:literal, $c:literal) => {
        ficha!(@base $id, $nombre, $a, $b, $c, None)
    };
    (@son $id:literal, $nombre:literal, $a:literal, $b:literal, $c:literal) => {
        ficha!(@base $id, $nombre, $a, $b, $c, Some(include_bytes!(concat!("../../../enciclopedia/sonidos/", $id, ".ogg"))))
    };
    (@base $id:literal, $nombre:literal, $a:literal, $b:literal, $c:literal, $son:expr) => {
        Ficha {
            nombre: $nombre,
            texto: [$a, $b, $c],
            foto: include_bytes!(concat!("../../../enciclopedia/fotos/", $id, ".jpg")),
            mini: include_bytes!(concat!("../../../enciclopedia/fotos/", $id, "-mini.jpg")),
            sonido: $son,
        }
    };
}

const ANIMALES: [Ficha; 20] = [
    ficha!(@son "perro", "Perro",
        "El perro es el mejor amigo de las personas.",
        "Tiene un olfato increíble: huele muchísimo mejor que nosotros.",
        "Cuando está contento, mueve la cola."),
    ficha!(@son "gato", "Gato",
        "El gato es muy limpio: se lava con su lengua áspera.",
        "Puede ver bien aunque haya poca luz.",
        "Cuando está tranquilo y feliz, ronronea."),
    ficha!(@son "caballo", "Caballo",
        "El caballo es fuerte y puede correr muy rápido.",
        "Come pasto, heno y avena.",
        "¡Puede dormir un rato de pie!"),
    ficha!(@son "vaca", "Vaca",
        "La vaca pasa muchas horas al día comiendo pasto.",
        "Nos da leche, y con la leche se hacen el queso y el yogur.",
        "Su estómago tiene cuatro partes."),
    ficha!(@son "oveja", "Oveja",
        "La oveja tiene el cuerpo cubierto de lana.",
        "Con su lana se hacen chalecos, gorros y bufandas.",
        "Las ovejas viven en grupos llamados rebaños."),
    ficha!(@son "gallina", "Gallina",
        "La gallina es un ave que vive en la granja.",
        "Pone huevos, ¡muchos huevos!",
        "Al macho se le llama gallo, y canta en las mañanas."),
    ficha!("conejo", "Conejo",
        "El conejo tiene orejas largas para escuchar muy bien.",
        "Sus dientes nunca dejan de crecer, por eso roe mucho.",
        "Se mueve dando saltos."),
    ficha!(@son "leon", "León",
        "El león es un felino grande que vive en África.",
        "El macho tiene una gran melena alrededor de la cabeza.",
        "Su rugido se escucha desde muy lejos."),
    ficha!(@son "elefante", "Elefante",
        "El elefante es el animal terrestre más grande del mundo.",
        "Usa su trompa para oler, tomar agua y agarrar comida.",
        "Sus orejas grandes lo ayudan a refrescarse."),
    ficha!("jirafa", "Jirafa",
        "La jirafa es el animal más alto del mundo.",
        "Con su cuello largo alcanza las hojas de los árboles altos.",
        "Cada jirafa tiene manchas distintas."),
    ficha!(@son "mono", "Mono",
        "El mono capuchino vive en los bosques de América.",
        "Es muy inteligente: usa piedras para abrir semillas.",
        "Salta de árbol en árbol con mucha agilidad."),
    ficha!(@son "oso", "Oso",
        "El oso pardo es grande y muy fuerte.",
        "Come de todo: pescados, frutas, raíces y miel.",
        "En invierno duerme mucho tiempo en su cueva."),
    ficha!(@son "tigre", "Tigre",
        "El tigre es el felino más grande del mundo.",
        "Cada tigre tiene sus propias rayas.",
        "A diferencia de muchos gatos, ¡le gusta nadar!"),
    ficha!("cebra", "Cebra",
        "La cebra vive en las praderas de África.",
        "Sus rayas blancas y negras son distintas en cada cebra.",
        "Vive en grupos y come pasto."),
    ficha!(@son "pinguino", "Pingüino",
        "El pingüino de Humboldt vive en las costas de Chile y Perú.",
        "No puede volar, pero es un gran nadador.",
        "Come peces pequeños, como la anchoveta."),
    ficha!(@son "ballena", "Ballena",
        "La ballena jorobada es enorme y vive en el mar.",
        "Aunque vive en el agua, respira aire, igual que nosotros.",
        "Los machos cantan canciones largas bajo el agua."),
    ficha!(@son "delfin", "Delfín",
        "El delfín vive en el mar y respira aire.",
        "Es muy inteligente y juguetón.",
        "Se comunica con silbidos y chasquidos."),
    ficha!("tortuga", "Tortuga",
        "La tortuga verde vive en el mar y puede vivir muchos años.",
        "Su caparazón duro la protege.",
        "Las mamás ponen sus huevos en la arena de la playa."),
    ficha!(@son "condor", "Cóndor",
        "El cóndor andino vive en la cordillera de los Andes.",
        "Es una de las aves voladoras más grandes del mundo.",
        "Aparece en el escudo de Chile."),
    ficha!("pudu", "Pudú",
        "El pudú es uno de los ciervos más pequeños del mundo.",
        "Vive en los bosques del sur de Chile.",
        "Es tímido y se esconde entre las plantas."),
];

const CUERPO: [Ficha; 15] = [
    ficha!("cuerpo-cerebro", "Cerebro",
        "El cerebro está dentro de tu cabeza, protegido por el cráneo.",
        "Con él piensas, recuerdas, aprendes y sientes.",
        "Manda órdenes a todo tu cuerpo, ¡incluso cuando duermes!"),
    ficha!("cuerpo-corazon", "Corazón",
        "El corazón es un músculo que está en tu pecho.",
        "Bombea la sangre para que llegue a todo tu cuerpo.",
        "Pon tu mano en el pecho: ¿lo sientes latir?"),
    ficha!("cuerpo-pulmones", "Pulmones",
        "Tienes dos pulmones dentro del pecho.",
        "Cuando respiras, se llenan de aire como dos globos.",
        "Toman del aire el oxígeno que tu cuerpo necesita."),
    ficha!("cuerpo-sangre", "Sangre",
        "La sangre viaja por todo tu cuerpo dentro de unos tubos llamados venas y arterias.",
        "Lleva oxígeno y alimento a cada parte de tu cuerpo.",
        "Cuando te haces una herida, se seca y forma una costra."),
    ficha!("cuerpo-huesos", "Huesos",
        "Los huesos son duros y forman tu esqueleto.",
        "Sostienen tu cuerpo y protegen lo que hay adentro.",
        "Una persona adulta tiene más de doscientos huesos."),
    ficha!("cuerpo-musculos", "Músculos",
        "Los músculos te permiten moverte, correr y saltar.",
        "Para moverte, tiran de tus huesos.",
        "Jugar y hacer ejercicio los pone fuertes."),
    ficha!("cuerpo-ojos", "Ojos",
        "Con los ojos ves los colores, las formas y la luz.",
        "Los párpados y las pestañas los protegen del polvo.",
        "Cada vez que pestañeas, los limpias."),
    ficha!("cuerpo-oidos", "Oídos",
        "Con los oídos escuchas los sonidos y la música.",
        "Tienes uno a cada lado de la cabeza.",
        "Adentro están los huesos más pequeños de tu cuerpo."),
    ficha!("cuerpo-nariz", "Nariz",
        "Con la nariz hueles los aromas, como el de las flores o el pan.",
        "También te sirve para respirar.",
        "Sus pelitos atrapan el polvo para que no entre a tu cuerpo."),
    ficha!("cuerpo-boca", "Boca",
        "Con la boca comes, hablas, cantas y sonríes.",
        "Adentro están los dientes y la lengua.",
        "La saliva ayuda a ablandar la comida."),
    ficha!("cuerpo-dientes", "Dientes",
        "Los dientes sirven para morder y masticar la comida.",
        "Primero salen los dientes de leche, y después los definitivos.",
        "Hay que cepillarlos después de comer y antes de dormir."),
    ficha!("cuerpo-lengua", "Lengua",
        "Con la lengua sientes los sabores: dulce, salado, ácido y amargo.",
        "Mueve la comida para que la puedas tragar.",
        "Sin la lengua sería muy difícil hablar."),
    ficha!("cuerpo-manos", "Manos",
        "Cada mano tiene cinco dedos.",
        "Con las manos tomas cosas, dibujas, escribes y saludas.",
        "Lávalas con agua y jabón para sacar los gérmenes."),
    ficha!("cuerpo-piernas", "Piernas",
        "Las piernas son largas y fuertes.",
        "En la mitad tienen la rodilla, que les permite doblarse.",
        "Con ellas caminas, corres, saltas y pateas la pelota."),
    ficha!("cuerpo-pies", "Pies",
        "Los pies sostienen todo tu cuerpo cuando estás de pie.",
        "Te ayudan a caminar, correr y saltar.",
        "Cada pie tiene cinco dedos."),
];

pub const TEMAS: [Tema; 2] = [
    Tema { nombre: "Animales", color: rgb(60, 150, 90), fichas: &ANIMALES },
    Tema { nombre: "Cuerpo humano", color: rgb(210, 80, 110), fichas: &CUERPO },
];

pub fn frases(f: &mut Frases) {
    f.es("Enciclopedia");
    for t in &TEMAS {
        f.es(t.nombre);
        for a in t.fichas {
            f.es(a.nombre);
            for s in a.texto {
                f.es(s);
            }
        }
    }
}

fn jpeg(bytes: &[u8]) -> Option<Pixmap> {
    use zune_jpeg::zune_core::{colorspace::ColorSpace, options::DecoderOptions};
    let opciones = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
    let mut d = zune_jpeg::JpegDecoder::new_with_options(bytes, opciones);
    let datos = d.decode().ok()?;
    let (w, h) = d.dimensions()?;
    Pixmap::from_vec(datos, IntSize::from_wh(w as u32, h as u32)?)
}

/// Parte un texto en líneas que caben en `ancho`.
fn envolver(l: &mut Lienzo, texto: &str, tam: f32, ancho: f32) -> Vec<String> {
    let mut lineas = Vec::new();
    let mut actual = String::new();
    for p in texto.split_whitespace() {
        let prueba = if actual.is_empty() { p.to_string() } else { format!("{actual} {p}") };
        if l.medir(&prueba, tam) > ancho && !actual.is_empty() {
            lineas.push(std::mem::replace(&mut actual, p.to_string()));
        } else {
            actual = prueba;
        }
    }
    if !actual.is_empty() {
        lineas.push(actual);
    }
    lineas
}

#[derive(Clone, Copy, PartialEq)]
enum Vista {
    Temas,
    Lista(usize),
    Ficha(usize, usize),
}

pub struct Enciclopedia {
    vista: Vista,
    minis: Vec<Vec<Option<Pixmap>>>,
    portadas: Vec<Option<Pixmap>>,
    foto: Option<((usize, usize), Option<Pixmap>)>,
}

// --- geometría (coordenadas locales de la app: AW x AH) ---
const VOLVER: R = R::new(16.0, 12.0, 240.0, 46.0);
const COLS: usize = 5;
const CELDA_W: f32 = 188.0;
const CELDA_H: f32 = 104.0;
const SEP: f32 = 7.0;
const GRILLA_Y: f32 = 72.0;
const FOTO: R = R::new(20.0, 70.0, 480.0, 430.0);
const TEXTO_X: f32 = 528.0;
const TEXTO_W: f32 = 450.0;
const BOTONES_Y: f32 = 462.0;
const ANTERIOR: (f32, f32) = (580.0, BOTONES_Y);
const ESCUCHAR: (f32, f32) = (753.0, BOTONES_Y);
const SIGUIENTE: (f32, f32) = (926.0, BOTONES_Y);
/// Botón "¿cómo suena?" en la esquina de la foto (tocar cualquier parte de la foto también suena).
const SONIDO: (f32, f32) = (FOTO.x + FOTO.w - 46.0, FOTO.y + FOTO.h - 46.0);

/// Botón redondo con una nota musical.
fn boton_nota(l: &mut Lienzo, x: f32, y: f32, r: f32, c: C, cx: &Cx) {
    let sobre = dist(cx.rl, (x, y)) < r;
    let rr = if sobre { r * 1.1 } else { r };
    l.circulo(x, y + 3.0, rr + 3.0, alfa(NEGRO, 0.3));
    l.circulo(x, y, rr + 3.0, BLANCO);
    l.circulo(x, y, rr, c);
    let u = rr / 10.0;
    l.ovalo(x - 3.0 * u, y + 4.5 * u, 3.6 * u, 2.8 * u, BLANCO);
    l.rect(x - 0.2 * u, y - 6.5 * u, 1.6 * u, 11.0 * u, BLANCO);
    l.poligono(&[(x - 0.2 * u, y - 6.5 * u), (x + 5.0 * u, y - 3.5 * u), (x + 5.0 * u, y - 1.0 * u), (x + 1.4 * u, y - 3.0 * u)], BLANCO);
}

fn grilla_x0() -> f32 {
    (AW - (COLS as f32 * CELDA_W + (COLS - 1) as f32 * SEP)) / 2.0
}

fn celda(i: usize) -> R {
    let (c, f) = (i % COLS, i / COLS);
    R::new(grilla_x0() + c as f32 * (CELDA_W + SEP), GRILLA_Y + f as f32 * (CELDA_H + SEP), CELDA_W, CELDA_H)
}

fn tarjeta_tema(i: usize) -> R {
    let n = TEMAS.len() as f32;
    let (w, h, sep) = (300.0, 300.0, 30.0);
    let x0 = (AW - (n * w + (n - 1.0) * sep)) / 2.0;
    R::new(x0 + i as f32 * (w + sep), 120.0, w, h)
}

impl Enciclopedia {
    pub fn nuevo(cx: &mut Cx) -> Box<dyn App> {
        cx.decir("Enciclopedia");
        let minis = TEMAS.iter().map(|t| t.fichas.iter().map(|a| jpeg(a.mini)).collect()).collect();
        let vista = if TEMAS.len() == 1 { Vista::Lista(0) } else { Vista::Temas };
        let portadas = TEMAS.iter().map(|t| jpeg(t.fichas[0].foto)).collect();
        Box::new(Enciclopedia { vista, minis, portadas, foto: None })
    }

    fn leer(&self, cx: &mut Cx, t: usize, i: usize) {
        let a = &TEMAS[t].fichas[i];
        let mut partes = vec![a.nombre];
        partes.extend(a.texto);
        cx.audio.callar();
        cx.decir_es_v(&partes);
    }

    fn abrir(&mut self, cx: &mut Cx, t: usize, i: usize) {
        self.vista = Vista::Ficha(t, i);
        self.leer(cx, t, i);
    }

    fn mover(&mut self, cx: &mut Cx, d: i32) {
        if let Vista::Ficha(t, i) = self.vista {
            let n = TEMAS[t].fichas.len() as i32;
            cx.sonar(Son::Whoosh);
            self.abrir(cx, t, ((i as i32 + d).rem_euclid(n)) as usize);
        }
    }

    fn volver(&mut self, cx: &mut Cx) {
        self.vista = match self.vista {
            Vista::Ficha(t, _) => Vista::Lista(t),
            _ => Vista::Temas,
        };
        cx.audio.callar();
        cx.sonar(Son::Clic);
    }

    fn hay_volver(&self) -> bool {
        match self.vista {
            Vista::Temas => false,
            Vista::Lista(_) => TEMAS.len() > 1,
            Vista::Ficha(..) => true,
        }
    }

    fn dibujar_foto(l: &mut Lienzo, img: &Pixmap, r: R) {
        let esc = (r.w / img.width() as f32).min(r.h / img.height() as f32);
        let pp = PixmapPaint { quality: FilterQuality::Bilinear, ..Default::default() };
        let (w, h) = (img.width() as f32 * esc, img.height() as f32 * esc);
        let (x, y) = (r.cx() - w / 2.0, r.cy() - h / 2.0);
        l.pix.draw_pixmap(0, 0, img.as_ref(), &pp, Transform::from_row(esc, 0.0, 0.0, esc, x, y), None);
    }
}

impl App for Enciclopedia {
    fn animando(&self) -> bool {
        false
    }

    fn evento(&mut self, ev: &Ev, cx: &mut Cx) {
        if let Ev::Clic(x, y) = ev {
            let p = (*x, *y);
            if self.hay_volver() && VOLVER.dentro(p) {
                self.volver(cx);
                return;
            }
            match self.vista {
                Vista::Temas => {
                    if let Some(t) = (0..TEMAS.len()).find(|&t| tarjeta_tema(t).dentro(p)) {
                        cx.sonar(Son::Clic);
                        cx.decir(TEMAS[t].nombre);
                        self.vista = Vista::Lista(t);
                    }
                }
                Vista::Lista(t) => {
                    if let Some(i) = (0..TEMAS[t].fichas.len()).find(|&i| celda(i).dentro(p)) {
                        cx.sonar(Son::Pop);
                        self.abrir(cx, t, i);
                    }
                }
                Vista::Ficha(t, i) => {
                    if dist(p, ANTERIOR) < 34.0 {
                        self.mover(cx, -1);
                    } else if dist(p, SIGUIENTE) < 34.0 {
                        self.mover(cx, 1);
                    } else if dist(p, ESCUCHAR) < 38.0 {
                        self.leer(cx, t, i);
                    } else if FOTO.dentro(p) {
                        match TEMAS[t].fichas[i].sonido {
                            Some(son) => cx.audio.reproducir(son),
                            None => cx.decir(TEMAS[t].fichas[i].nombre),
                        }
                    }
                }
            }
            return;
        }
        match (ev, self.vista) {
            (Ev::Tecla(T::Izq), Vista::Ficha(..)) => self.mover(cx, -1),
            (Ev::Tecla(T::Der), Vista::Ficha(..)) => self.mover(cx, 1),
            (Ev::Tecla(T::Espacio), Vista::Ficha(t, i)) => self.leer(cx, t, i),
            (Ev::Tecla(T::Escape), _) if self.hay_volver() => self.volver(cx),
            _ => {}
        }
    }

    fn dibujar(&mut self, l: &mut Lienzo, cx: &Cx) {
        l.degradado_v(0.0, 0.0, AW, AH, rgb(236, 246, 238), rgb(206, 230, 212));
        let titulo = match self.vista {
            Vista::Temas => "Enciclopedia",
            Vista::Lista(t) | Vista::Ficha(t, _) => TEMAS[t].nombre,
        };
        let color = match self.vista {
            Vista::Temas => rgb(60, 150, 90),
            Vista::Lista(t) | Vista::Ficha(t, _) => TEMAS[t].color,
        };
        if self.hay_volver() {
            let destino = match self.vista {
                Vista::Ficha(t, _) => TEMAS[t].nombre,
                _ => "Temas",
            };
            boton_texto(l, VOLVER, &format!("   {destino}"), color, false, cx);
            let (tx, ty) = (VOLVER.x + 30.0, VOLVER.cy());
            l.poligono(&[(tx - 9.0, ty), (tx + 6.0, ty - 9.0), (tx + 6.0, ty + 9.0)], oscurecer(color, 0.35));
        }

        match self.vista {
            Vista::Temas => {
                l.texto_c(titulo, AW / 2.0, 50.0, 46.0, oscurecer(color, 0.3));
                for (k, tema) in TEMAS.iter().enumerate() {
                    let r = tarjeta_tema(k);
                    let r = if r.dentro(cx.rl) { r.crecer(4.0) } else { r };
                    l.rrect_r(r.mover(0.0, 5.0), 24.0, oscurecer(tema.color, 0.3));
                    l.rrect_r(r, 24.0, tema.color);
                    if let Some(img) = &self.portadas[k] {
                        Self::dibujar_foto(l, img, R::new(r.x + 20.0, r.y + 20.0, r.w - 40.0, r.h - 100.0));
                    }
                    let tam = (40.0 * (r.w - 36.0) / l.medir(tema.nombre, 40.0)).min(40.0);
                    l.texto_c(tema.nombre, r.cx(), r.abajo() - 42.0, tam, BLANCO);
                }
            }
            Vista::Lista(t) => {
                l.texto_c(titulo, AW / 2.0, 36.0, 40.0, oscurecer(color, 0.3));
                for (i, a) in TEMAS[t].fichas.iter().enumerate() {
                    let r = celda(i);
                    let sobre = r.dentro(cx.rl);
                    l.rrect_r(r.mover(0.0, 3.0).crecer(3.0), 12.0, alfa(NEGRO, 0.18));
                    l.rrect_r(r.crecer(3.0), 12.0, if sobre { AMARILLO } else { BLANCO });
                    if let Some(img) = &self.minis[t][i] {
                        Self::dibujar_foto(l, img, r);
                    }
                    l.rect(r.x, r.abajo() - 30.0, r.w, 30.0, alfa(NEGRO, 0.5));
                    l.texto_c(a.nombre, r.cx(), r.abajo() - 15.0, 24.0, BLANCO);
                }
            }
            Vista::Ficha(t, i) => {
                let a = &TEMAS[t].fichas[i];
                if self.foto.as_ref().map(|f| f.0) != Some((t, i)) {
                    self.foto = Some(((t, i), jpeg(a.foto)));
                }
                // foto con marco
                l.rect(FOTO.x + 6.0, FOTO.y + 8.0, FOTO.w, FOTO.h, alfa(NEGRO, 0.25));
                l.rect(FOTO.x - 6.0, FOTO.y - 6.0, FOTO.w + 12.0, FOTO.h + 12.0, BLANCO);
                if let Some((_, Some(img))) = &self.foto {
                    Self::dibujar_foto(l, img, FOTO);
                }
                if a.sonido.is_some() {
                    boton_nota(l, SONIDO.0, SONIDO.1, 34.0, NARANJO, cx);
                }
                // nombre y texto
                l.texto(a.nombre, TEXTO_X, 62.0, 56.0, oscurecer(color, 0.35));
                let mut y = 142.0;
                for s in a.texto {
                    let lineas = envolver(l, s, 27.0, TEXTO_W - 26.0);
                    l.circulo(TEXTO_X + 8.0, y + 17.0, 6.0, color);
                    for ln in lineas {
                        l.texto(&ln, TEXTO_X + 26.0, y, 27.0, TINTA);
                        y += 36.0;
                    }
                    y += 14.0;
                }
                // botones
                boton_flecha(l, ANTERIOR.0, ANTERIOR.1, 32.0, -1.0, NARANJO, cx);
                boton_altavoz(l, ESCUCHAR.0, ESCUCHAR.1, 36.0, color, cx);
                boton_flecha(l, SIGUIENTE.0, SIGUIENTE.1, 32.0, 1.0, NARANJO, cx);
                l.texto_c(&format!("{} / {}", i + 1, TEMAS[t].fichas.len()), (ANTERIOR.0 + ESCUCHAR.0) / 2.0, BOTONES_Y, 20.0, GRIS);
            }
        }
    }
}
