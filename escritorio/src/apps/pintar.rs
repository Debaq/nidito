//! Pintar: colores, pinceles, arcoíris, goma y stickers. Guarda en ~/Dibujos.

use super::*;
use crate::cx::Son;
use crate::dibujos::{self, Obj};
use tiny_skia::Pixmap;

pub const COLORES: [(&str, &str, C); 12] = [
    ("rojo", "red", ROJO),
    ("naranjo", "orange", NARANJO),
    ("amarillo", "yellow", AMARILLO),
    ("verde", "green", VERDE),
    ("celeste", "light blue", CELESTE),
    ("azul", "blue", AZUL),
    ("morado", "purple", MORADO),
    ("rosado", "pink", ROSADO),
    ("café", "brown", CAFE),
    ("negro", "black", TINTA),
    ("gris", "gray", GRIS),
    ("blanco", "white", BLANCO),
];

#[derive(Clone, Copy, PartialEq)]
enum Herr {
    Pincel,
    Arcoiris,
    Goma,
    Sello(Obj),
}

#[derive(Clone, Copy, PartialEq)]
enum Bot {
    Tam(usize),
    Herr(Herr),
    Nueva,
}

const TAMS: [(&str, &str); 3] = [("chico", "small"), ("mediano", "medium"), ("grande", "big")];

pub fn frases(f: &mut Frases) {
    for (a, b, _) in COLORES {
        f.bil(a, b);
    }
    for (a, b) in TAMS {
        f.bil(a, b);
    }
    f.bil("pincel", "brush");
    f.bil("goma", "eraser");
    f.bil("arcoíris", "rainbow");
    f.bil("¡Hoja nueva!", "New page!");
    f.bil("¿Hoja nueva? Toca otra vez.", "New page? Touch again.");
}

const HOJA: R = R::new(128.0, 8.0, 744.0, 502.0);
const ANCHOS: [f32; 3] = [8.0, 18.0, 36.0];
const SELLOS: [f32; 3] = [60.0, 95.0, 140.0];

pub struct Pintar {
    hoja: Pixmap,
    color: usize,
    herr: Herr,
    tam: usize,
    ultimo: Option<(f32, f32)>,
    trazando: bool,
    sucio: bool,
    borrar_t: f32,
    tono: f32,
    botones: Vec<(R, Bot)>,
    base: Option<Pixmap>,
    firma: Option<(usize, usize, Herr, bool)>,
}

fn color_pos(i: usize) -> (f32, f32) {
    (34.0 + (i % 2) as f32 * 58.0, 44.0 + (i / 2) as f32 * 82.0)
}

impl Pintar {
    pub fn nuevo(_cx: &mut Cx) -> Box<dyn App> {
        let mut hoja = Pixmap::new(HOJA.w as u32, HOJA.h as u32).unwrap();
        hoja.fill(tiny_skia::Color::WHITE);
        let mut botones = Vec::new();
        let lista = [
            Bot::Tam(0),
            Bot::Tam(1),
            Bot::Tam(2),
            Bot::Herr(Herr::Goma),
            Bot::Herr(Herr::Pincel),
            Bot::Herr(Herr::Arcoiris),
            Bot::Herr(Herr::Sello(Obj::Estrella)),
            Bot::Herr(Herr::Sello(Obj::Corazon)),
            Bot::Herr(Herr::Sello(Obj::Flor)),
            Bot::Herr(Herr::Sello(Obj::Sol)),
            Bot::Herr(Herr::Sello(Obj::Gato)),
            Bot::Herr(Herr::Sello(Obj::Pez)),
        ];
        for (i, b) in lista.iter().enumerate() {
            let x = 884.0 + (i % 2) as f32 * 58.0;
            let y = 14.0 + (i / 2) as f32 * 70.0;
            botones.push((R::new(x, y, 52.0, 56.0), *b));
        }
        botones.push((R::new(884.0, 440.0, 110.0, 64.0), Bot::Nueva));
        Box::new(Pintar {
            hoja,
            color: 5,
            herr: Herr::Pincel,
            tam: 1,
            ultimo: None,
            trazando: false,
            sucio: false,
            borrar_t: 0.0,
            tono: 0.0,
            botones,
            base: None,
            firma: None,
        })
    }

    fn guardar(&mut self) {
        if !self.sucio {
            return;
        }
        let dir = ruta_datos("Dibujos");
        let _ = std::fs::create_dir_all(&dir);
        let seg = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let _ = self.hoja.save_png(dir.join(format!("dibujo-{seg}.png")));
        self.sucio = false;
    }

    fn trazar(&mut self, a: (f32, f32), b: (f32, f32)) {
        let (a, b) = ((a.0 - HOJA.x, a.1 - HOJA.y), (b.0 - HOJA.x, b.1 - HOJA.y));
        let ancho = ANCHOS[self.tam];
        match self.herr {
            Herr::Pincel => self.hoja.linea(a.0, a.1, b.0, b.1, ancho, COLORES[self.color].2),
            Herr::Goma => self.hoja.linea(a.0, a.1, b.0, b.1, ancho * 2.0, BLANCO),
            Herr::Arcoiris => {
                // segmentos cortos para que el color cambie a lo largo del trazo
                let d = dist(a, b);
                let pasos = (d / 6.0).ceil().max(1.0) as usize;
                let mut prev = a;
                for i in 1..=pasos {
                    let t = i as f32 / pasos as f32;
                    let p = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
                    self.tono += dist(prev, p) * 0.8;
                    self.hoja.linea(prev.0, prev.1, p.0, p.1, ancho, hsv(self.tono, 0.85, 0.95));
                    prev = p;
                }
                if pasos == 1 && d < 0.5 {
                    self.hoja.circulo(a.0, a.1, ancho / 2.0, hsv(self.tono, 0.85, 0.95));
                }
            }
            Herr::Sello(_) => {}
        }
        self.sucio = true;
    }
}

impl App for Pintar {
    fn animando(&self) -> bool {
        self.borrar_t > 0.0
    }

    fn evento(&mut self, ev: &Ev, cx: &mut Cx) {
        match ev {
            Ev::Clic(x, y) => {
                let p = (*x, *y);
                if HOJA.dentro(p) {
                    if let Herr::Sello(o) = self.herr {
                        dibujos::dibujar(&mut self.hoja, o, p.0 - HOJA.x, p.1 - HOJA.y, SELLOS[self.tam]);
                        self.sucio = true;
                        cx.sonar(Son::Sello);
                    } else {
                        self.trazando = true;
                        self.ultimo = Some(p);
                        self.trazar(p, p);
                    }
                    return;
                }
                for i in 0..COLORES.len() {
                    let (px, py) = color_pos(i);
                    if dist(p, (px, py)) < 26.0 {
                        self.color = i;
                        if matches!(self.herr, Herr::Goma | Herr::Arcoiris) {
                            self.herr = Herr::Pincel;
                        }
                        cx.sonar(Son::Clic);
                        cx.decir_bil(COLORES[i].0, COLORES[i].1);
                        return;
                    }
                }
                let botones = self.botones.clone();
                for (r, b) in botones {
                    if !r.dentro(p) {
                        continue;
                    }
                    cx.sonar(Son::Clic);
                    match b {
                        Bot::Tam(t) => {
                            self.tam = t;
                            cx.decir_bil(TAMS[t].0, TAMS[t].1);
                        }
                        Bot::Herr(h) => {
                            self.herr = h;
                            match h {
                                Herr::Pincel => cx.decir_bil("pincel", "brush"),
                                Herr::Goma => cx.decir_bil("goma", "eraser"),
                                Herr::Arcoiris => cx.decir_bil("arcoíris", "rainbow"),
                                Herr::Sello(o) => cx.decir_obj(o),
                            }
                        }
                        Bot::Nueva => {
                            if self.borrar_t > 0.0 {
                                self.guardar();
                                self.hoja.fill(tiny_skia::Color::WHITE);
                                self.borrar_t = 0.0;
                                cx.sonar(Son::Whoosh);
                                cx.decir_bil("¡Hoja nueva!", "New page!");
                            } else {
                                self.borrar_t = 2.5;
                                cx.decir_bil("¿Hoja nueva? Toca otra vez.", "New page? Touch again.");
                            }
                        }
                    }
                    return;
                }
            }
            Ev::Mueve(x, y) => {
                if self.trazando {
                    let p = (*x, *y);
                    if let Some(u) = self.ultimo {
                        self.trazar(u, p);
                    }
                    self.ultimo = Some(p);
                }
            }
            Ev::Suelta(..) => {
                self.trazando = false;
                self.ultimo = None;
            }
            _ => {}
        }
    }

    fn actualizar(&mut self, dt: f32, _cx: &mut Cx) {
        if self.borrar_t > 0.0 {
            self.borrar_t -= dt;
        }
    }

    fn cerrar(&mut self, _cx: &mut Cx) {
        self.guardar();
    }

    fn dibujar(&mut self, l: &mut Lienzo, cx: &Cx) {
        // capa base (paneles, colores y botones) en caché; se rehace solo si cambia algo
        let parpadeo = self.borrar_t > 0.0 && (cx.t * 6.0).sin() > 0.0;
        let firma = (self.color, self.tam, self.herr, parpadeo);
        if self.firma != Some(firma) || self.base.is_none() {
            let mut base = self.base.take().unwrap_or_else(|| Pixmap::new(AW as u32, AH as u32).unwrap());
            l.con(&mut base, |l| self.dibujar_base(l, parpadeo));
            self.base = Some(base);
            self.firma = Some(firma);
        }
        if let Some(b) = &self.base {
            copiar_opaco(&mut l.pix, b, 0, 0);
        }
        copiar_opaco(&mut l.pix, &self.hoja, HOJA.x as i32, HOJA.y as i32);
        // resaltado bajo el ratón
        for i in 0..COLORES.len() {
            let (x, y) = color_pos(i);
            if dist(cx.rl, (x, y)) < 26.0 && i != self.color {
                l.anillo(x, y, 26.0, 3.0, rgb(150, 160, 190));
            }
        }
        for (r, _) in &self.botones {
            if r.dentro(cx.rl) {
                l.rrect_borde(*r, 14.0, 3.0, rgb(120, 150, 230));
            }
        }
        // vista previa del pincel
        if HOJA.dentro(cx.rl) {
            let w = match self.herr {
                Herr::Sello(_) => SELLOS[self.tam] * 1.1,
                Herr::Goma => ANCHOS[self.tam] * 2.0,
                _ => ANCHOS[self.tam],
            };
            l.anillo(cx.rl.0, cx.rl.1, w / 2.0 + 1.0, 2.0, alfa(TINTA, 0.45));
        }
    }
}

impl Pintar {
    fn dibujar_base(&self, l: &mut Lienzo, parpadeo: bool) {
        l.rect(0.0, 0.0, AW, AH, rgb(225, 232, 245));
        l.rrect(6.0, 6.0, 116.0, AH - 12.0, 18.0, rgb(245, 247, 252));
        l.rrect(878.0, 6.0, 116.0, AH - 12.0, 18.0, rgb(245, 247, 252));
        l.rrect(HOJA.x - 4.0, HOJA.y - 2.0, HOJA.w + 8.0, HOJA.h + 8.0, 6.0, rgb(180, 190, 210));
        for (i, (_, _, c)) in COLORES.iter().enumerate() {
            let (x, y) = color_pos(i);
            let sel = i == self.color && self.herr != Herr::Goma && self.herr != Herr::Arcoiris;
            let r = if sel { 27.0 } else { 22.0 };
            if sel {
                l.circulo(x, y, r + 5.0, TINTA);
            }
            l.circulo(x, y, r, *c);
            if *c == BLANCO {
                l.anillo(x, y, r, 2.0, GRIS);
            }
        }
        let color = COLORES[self.color].2;
        for (r, b) in &self.botones {
            let sel = match b {
                Bot::Tam(t) => *t == self.tam,
                Bot::Herr(h) => *h == self.herr,
                Bot::Nueva => false,
            };
            let fondo = if sel { rgb(255, 240, 180) } else { BLANCO };
            l.rrect_r(*r, 14.0, fondo);
            l.rrect_borde(*r, 14.0, if sel { 4.0 } else { 2.0 }, if sel { NARANJO } else { rgb(200, 205, 220) });
            let (x, y) = (r.cx(), r.cy());
            match b {
                Bot::Tam(t) => l.circulo(x, y, ANCHOS[*t] / 2.0 + 1.0, if self.herr == Herr::Goma { GRIS } else { color }),
                Bot::Herr(Herr::Pincel) => {
                    l.linea(x - 12.0, y + 14.0, x + 10.0, y - 10.0, 7.0, CAFE);
                    l.circulo(x - 13.0, y + 15.0, 7.0, color);
                }
                Bot::Herr(Herr::Arcoiris) => {
                    for (k, c) in [ROJO, AMARILLO, VERDE, AZUL].iter().enumerate() {
                        l.anillo(x, y + 12.0, 20.0 - k as f32 * 5.0, 4.5, *c);
                    }
                    l.rect(r.x + 3.0, y + 13.0, r.w - 6.0, r.h / 2.0 - 16.0, fondo);
                }
                Bot::Herr(Herr::Goma) => {
                    l.rrect(x - 16.0, y - 10.0, 32.0, 20.0, 5.0, ROSADO);
                    l.rect(x + 2.0, y - 10.0, 14.0, 20.0, rgb(120, 150, 230));
                }
                Bot::Herr(Herr::Sello(o)) => dibujos::dibujar(l, *o, x, y, 40.0),
                Bot::Nueva => {
                    l.rrect_r(*r, 14.0, if parpadeo { rgb(255, 200, 200) } else { fondo });
                    l.rrect(x - 30.0, y - 22.0, 34.0, 44.0, 4.0, BLANCO);
                    l.rrect_borde(R::new(x - 30.0, y - 22.0, 34.0, 44.0), 4.0, 2.0, GRIS);
                    l.circulo(x + 18.0, y + 8.0, 15.0, VERDE);
                    l.rect(x + 11.0, y + 6.0, 14.0, 4.0, BLANCO);
                    l.rect(x + 16.0, y + 1.0, 4.0, 14.0, BLANCO);
                }
            }
        }
    }
}
