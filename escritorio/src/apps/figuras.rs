//! Figuras: mirar las figuras geométricas con su nombre y jugar "¿Dónde está el triángulo?".
//! El juego parte con 4 figuras fáciles y suma las demás a medida que gana estrellas.
//! Si se equivoca dos veces, la figura correcta brilla para ayudarla.

use super::*;
use crate::cx::Son;
use crate::efectos::Confeti;
use tiny_skia::{Path, PathBuilder};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Forma {
    Circulo,
    Cuadrado,
    Triangulo,
    Rectangulo,
    Estrella,
    Corazon,
    Ovalo,
    Rombo,
}

use Forma::*;

pub const TODAS: [Forma; 8] = [Circulo, Cuadrado, Triangulo, Rectangulo, Estrella, Corazon, Ovalo, Rombo];
/// Las primeras que se juegan (las más comunes).
const FACILES: [Forma; 4] = [Circulo, Cuadrado, Triangulo, Rectangulo];
/// Estrellas para pasar a todas las figuras, y luego a 4 opciones.
const NIVEL_2: usize = 6;
const NIVEL_3: usize = 14;

impl Forma {
    pub fn nombre(self) -> &'static str {
        match self {
            Circulo => "círculo",
            Cuadrado => "cuadrado",
            Triangulo => "triángulo",
            Rectangulo => "rectángulo",
            Estrella => "estrella",
            Corazon => "corazón",
            Ovalo => "óvalo",
            Rombo => "rombo",
        }
    }
    /// Con mayúscula, para mostrar y decir suelto.
    fn titulo(self) -> &'static str {
        match self {
            Circulo => "Círculo",
            Cuadrado => "Cuadrado",
            Triangulo => "Triángulo",
            Rectangulo => "Rectángulo",
            Estrella => "Estrella",
            Corazon => "Corazón",
            Ovalo => "Óvalo",
            Rombo => "Rombo",
        }
    }
    fn con_articulo(self) -> String {
        let art = if self == Estrella { "la" } else { "el" };
        format!("{art} {}", self.nombre())
    }
    pub fn ingles(self) -> &'static str {
        match self {
            Circulo => "circle",
            Cuadrado => "square",
            Triangulo => "triangle",
            Rectangulo => "rectangle",
            Estrella => "star",
            Corazon => "heart",
            Ovalo => "oval",
            Rombo => "diamond",
        }
    }
    fn titulo_en(self) -> String {
        let n = self.ingles();
        let mut c = n.chars();
        c.next().map(|p| p.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
    }
    /// Cómo es (se lee al tocarla en "Mirar").
    fn como_es(self) -> (&'static str, &'static str) {
        match self {
            Circulo => ("Es redondo, como una pelota.", "It is round, like a ball."),
            Cuadrado => ("Tiene cuatro lados iguales.", "It has four equal sides."),
            Triangulo => ("Tiene tres lados y tres puntas.", "It has three sides and three corners."),
            Rectangulo => ("Tiene cuatro lados: dos largos y dos cortos.", "It has four sides: two long and two short."),
            Estrella => ("Tiene cinco puntas.", "It has five points."),
            Corazon => ("Es la forma del cariño.", "It is the shape of love."),
            Ovalo => ("Es redondo y alargado, como un huevo.", "It is round and long, like an egg."),
            Rombo => ("Es como un cuadrado parado en una punta.", "It looks like a square standing on a corner."),
        }
    }
    /// Color propio en "Mirar" (en el juego se usan colores al azar).
    fn color(self) -> C {
        match self {
            Circulo => ROJO,
            Cuadrado => AZUL,
            Triangulo => AMARILLO,
            Rectangulo => VERDE,
            Estrella => NARANJO,
            Corazon => ROSADO,
            Ovalo => MORADO,
            Rombo => CELESTE,
        }
    }
}

/// Figura centrada en (x, y) dentro de un cuadrado de lado s.
fn ruta(f: Forma, x: f32, y: f32, s: f32) -> Option<Path> {
    match f {
        Circulo => PathBuilder::from_circle(x, y, s * 0.46),
        Cuadrado => ruta_poligono(&[(x - s * 0.4, y - s * 0.4), (x + s * 0.4, y - s * 0.4), (x + s * 0.4, y + s * 0.4), (x - s * 0.4, y + s * 0.4)]),
        Triangulo => ruta_poligono(&[(x, y - s * 0.46), (x + s * 0.5, y + s * 0.4), (x - s * 0.5, y + s * 0.4)]),
        Rectangulo => ruta_poligono(&[(x - s * 0.5, y - s * 0.28), (x + s * 0.5, y - s * 0.28), (x + s * 0.5, y + s * 0.28), (x - s * 0.5, y + s * 0.28)]),
        Estrella => ruta_estrella(x, y + s * 0.04, s * 0.52, s * 0.22, 5, -std::f32::consts::FRAC_PI_2),
        Corazon => ruta_corazon(x, y + s * 0.02, s * 1.02),
        Ovalo => tiny_skia::Rect::from_xywh(x - s * 0.31, y - s * 0.46, s * 0.62, s * 0.92).and_then(PathBuilder::from_oval),
        Rombo => ruta_poligono(&[(x, y - s * 0.5), (x + s * 0.34, y), (x, y + s * 0.5), (x - s * 0.34, y)]),
    }
}

pub fn dibujar_forma(l: &mut Lienzo, f: Forma, x: f32, y: f32, s: f32, c: C) {
    l.llenar(ruta(f, x + s * 0.03, y + s * 0.04, s), alfa(TINTA, 0.18));
    l.llenar(ruta(f, x, y, s), c);
    l.delinear(ruta(f, x, y, s), (s * 0.035).max(2.0), oscurecer(c, 0.3));
}

#[derive(PartialEq, Clone, Copy)]
enum Modo {
    Mirar,
    Jugar,
}

pub struct Figuras {
    modo: Modo,
    sel: Option<(usize, f32)>,
    opciones: Vec<(Forma, C, f32)>,
    meta: Forma,
    estrellas: usize,
    fallos: usize,
    celebrando: f32,
    sacudir: Option<(usize, f32)>,
    confeti: Confeti,
}

const B_MIRAR: R = R::new(20.0, 14.0, 190.0, 54.0);
const B_JUGAR: R = R::new(222.0, 14.0, 190.0, 54.0);
const ALTAVOZ: (f32, f32) = (940.0, 41.0);

fn tarjeta(i: usize) -> R {
    let (c, f) = (i % 4, i / 4);
    R::new(30.0 + c as f32 * 240.0, 92.0 + f as f32 * 212.0, 220.0, 198.0)
}

fn opcion(i: usize, n: usize) -> R {
    if n == 4 {
        R::new(30.0 + i as f32 * 240.0, 160.0, 220.0, 280.0)
    } else {
        R::new(80.0 + i as f32 * 290.0, 150.0, 260.0, 300.0)
    }
}

fn donde(f: Forma) -> (String, String) {
    (format!("¿Dónde está {}?", f.con_articulo()), format!("Where is the {}?", f.ingles()))
}
fn si_es(f: Forma) -> (String, String) {
    (format!("¡Sí! Es {}.", f.con_articulo()), format!("Yes! The {}!", f.ingles()))
}
fn ese_es(f: Forma) -> (String, String) {
    let ese = if f == Estrella { "Esa es" } else { "Ese es" };
    (format!("{ese} {}.", f.con_articulo()), format!("That is the {}.", f.ingles()))
}

pub fn frases(f: &mut Frases) {
    f.bil("Mirar", "Look");
    f.bil("Jugar", "Play");
    f.bil("Toca una figura para saber su nombre.", "Touch a shape to hear its name.");
    for o in TODAS {
        f.bil(o.titulo(), o.titulo_en());
        let (a, b) = o.como_es();
        f.bil(a, b);
        for (a, b) in [donde(o), si_es(o), ese_es(o)] {
            f.bil(a, b);
        }
    }
}

impl Figuras {
    pub fn nuevo(cx: &mut Cx) -> Box<dyn App> {
        let mut e = Figuras {
            modo: Modo::Mirar,
            sel: None,
            opciones: vec![],
            meta: Circulo,
            estrellas: 0,
            fallos: 0,
            celebrando: 0.0,
            sacudir: None,
            confeti: Confeti::default(),
        };
        e.nueva_pregunta(cx, false);
        cx.decir_bil("Toca una figura para saber su nombre.", "Touch a shape to hear its name.");
        Box::new(e)
    }

    fn nueva_pregunta(&mut self, cx: &mut Cx, hablar: bool) {
        let mut pool: Vec<Forma> = if self.estrellas < NIVEL_2 { FACILES.to_vec() } else { TODAS.to_vec() };
        let n = if self.estrellas < NIVEL_3 { 3 } else { 4 };
        cx.rng.barajar(&mut pool);
        let mut elegidas: Vec<Forma> = pool[..n].to_vec();
        // que no se repita la misma pregunta dos veces seguidas
        let mut meta = elegidas[cx.rng.n(n)];
        if meta == self.meta {
            meta = elegidas[(elegidas.iter().position(|&f| f == meta).unwrap() + 1) % n];
        }
        cx.rng.barajar(&mut elegidas);
        // colores distintos entre sí y al azar: así aprende la forma, no el color
        let mut colores = ARCOIRIS.to_vec();
        cx.rng.barajar(&mut colores);
        self.opciones = elegidas.iter().enumerate().map(|(i, &f)| (f, colores[i], cx.rng.rango(0.78, 1.0))).collect();
        self.meta = meta;
        self.fallos = 0;
        if hablar {
            self.preguntar(cx);
        }
    }

    fn preguntar(&self, cx: &mut Cx) {
        let (es, en) = donde(self.meta);
        cx.decir_bil(&es, &en);
    }

    fn contar_figura(&self, cx: &mut Cx, f: Forma) {
        let (a, b) = f.como_es();
        cx.decir_v(&[f.titulo(), a], &[f.titulo_en().as_str(), b]);
    }
}

impl App for Figuras {
    fn animando(&self) -> bool {
        self.celebrando > 0.0
            || self.sacudir.is_some()
            || self.confeti.activo()
            || self.sel.is_some_and(|s| s.1 < 0.4)
            || (self.modo == Modo::Jugar && self.fallos >= 2)
    }

    fn evento(&mut self, ev: &Ev, cx: &mut Cx) {
        let Ev::Clic(x, y) = ev else { return };
        let p = (*x, *y);
        if B_MIRAR.dentro(p) && self.modo != Modo::Mirar {
            self.modo = Modo::Mirar;
            cx.sonar(Son::Clic);
            cx.decir_bil("Mirar", "Look");
            return;
        }
        if B_JUGAR.dentro(p) && self.modo != Modo::Jugar {
            self.modo = Modo::Jugar;
            cx.sonar(Son::Clic);
            self.nueva_pregunta(cx, true);
            return;
        }
        if dist(p, ALTAVOZ) < 30.0 {
            match self.modo {
                Modo::Jugar => self.preguntar(cx),
                Modo::Mirar => match self.sel {
                    Some((i, _)) => self.contar_figura(cx, TODAS[i]),
                    None => cx.decir_bil("Toca una figura para saber su nombre.", "Touch a shape to hear its name."),
                },
            }
            return;
        }
        match self.modo {
            Modo::Mirar => {
                for (i, &f) in TODAS.iter().enumerate() {
                    if tarjeta(i).dentro(p) {
                        self.sel = Some((i, 0.0));
                        cx.sonar(Son::Clic);
                        self.contar_figura(cx, f);
                    }
                }
            }
            Modo::Jugar => {
                if self.celebrando > 0.0 {
                    return;
                }
                let n = self.opciones.len();
                for i in 0..n {
                    if opcion(i, n).dentro(p) {
                        let f = self.opciones[i].0;
                        if f == self.meta {
                            self.estrellas += 1;
                            self.celebrando = 2.6;
                            cx.sonar(Son::Fanfarria);
                            let r = opcion(i, n);
                            self.confeti.lanzar(r.cx(), r.cy(), 70, &mut cx.rng);
                            let (a, b) = si_es(f);
                            cx.decir_bil(&a, &b);
                        } else {
                            cx.sonar(Son::Boop);
                            self.fallos += 1;
                            self.sacudir = Some((i, 0.6));
                            let (a, b) = ese_es(f);
                            let (c, d) = donde(self.meta);
                            cx.decir_v(&[a, c], &[b, d]);
                        }
                    }
                }
            }
        }
    }

    fn actualizar(&mut self, dt: f32, cx: &mut Cx) {
        self.confeti.actualizar(dt);
        if let Some((_, t)) = &mut self.sel {
            *t += dt;
        }
        if let Some((_, t)) = &mut self.sacudir {
            *t -= dt;
            if *t <= 0.0 {
                self.sacudir = None;
            }
        }
        if self.celebrando > 0.0 {
            self.celebrando -= dt;
            if self.celebrando <= 0.0 {
                self.nueva_pregunta(cx, true);
            }
        }
    }

    fn dibujar(&mut self, l: &mut Lienzo, cx: &Cx) {
        l.degradado_v(0.0, 0.0, AW, AH, rgb(255, 248, 232), rgb(255, 232, 210));
        l.rect(0.0, 0.0, AW, 82.0, rgb(255, 236, 214));
        boton_texto(l, B_MIRAR, "Mirar", NARANJO, self.modo == Modo::Mirar, cx);
        boton_texto(l, B_JUGAR, "Jugar", ROJO, self.modo == Modo::Jugar, cx);
        boton_altavoz(l, ALTAVOZ.0, ALTAVOZ.1, 28.0, NARANJO, cx);
        let ingles = cx.idioma != crate::cx::Idioma::Es;
        match self.modo {
            Modo::Mirar => {
                for (i, &f) in TODAS.iter().enumerate() {
                    let mut r = tarjeta(i);
                    let sel = self.sel.filter(|s| s.0 == i);
                    let mut s = 104.0;
                    if let Some((_, t)) = sel {
                        let k = ease_out_back((t / 0.35).min(1.0));
                        r = r.crecer(6.0 * k);
                        s += 14.0 * k;
                    } else if r.dentro(cx.rl) {
                        r = r.crecer(3.0);
                    }
                    l.rrect_r(r.mover(0.0, 6.0), 20.0, alfa(TINTA, 0.18));
                    l.rrect_r(r, 20.0, BLANCO);
                    if sel.is_some() {
                        l.rrect_borde(r, 20.0, 5.0, f.color());
                    }
                    dibujar_forma(l, f, r.cx(), r.y + 80.0, s, f.color());
                    let tc = oscurecer(f.color(), 0.35);
                    if ingles {
                        l.texto_c(f.titulo(), r.cx(), r.y + 152.0, 30.0, tc);
                        l.texto_c(&f.titulo_en(), r.cx(), r.y + 181.0, 21.0, GRIS);
                    } else {
                        l.texto_c(f.titulo(), r.cx(), r.y + 160.0, 34.0, tc);
                    }
                }
            }
            Modo::Jugar => {
                l.estrella(470.0, 42.0, 24.0, -std::f32::consts::FRAC_PI_2, AMARILLO);
                l.texto(&self.estrellas.to_string(), 500.0, 18.0, 40.0, rgb(200, 120, 0));
                let (es, en) = donde(self.meta);
                let pregunta = if cx.idioma == crate::cx::Idioma::En { en } else { es };
                l.texto_c(&pregunta, AW / 2.0, 116.0, 44.0, TINTA);
                let n = self.opciones.len();
                let ayuda = self.fallos >= 2 && self.celebrando <= 0.0;
                for (i, &(f, c, esc)) in self.opciones.iter().enumerate() {
                    let mut r = opcion(i, n);
                    if let Some((k, t)) = self.sacudir {
                        if k == i {
                            r = r.mover((t * 40.0).sin() * 10.0 * t, 0.0);
                        }
                    }
                    let es_meta = f == self.meta;
                    let gana = self.celebrando > 0.0 && es_meta;
                    if r.dentro(cx.rl) || gana {
                        r = r.crecer(5.0);
                    }
                    l.rrect_r(r.mover(0.0, 6.0), 24.0, alfa(TINTA, 0.18));
                    l.rrect_r(r, 24.0, BLANCO);
                    if gana {
                        l.rrect_borde(r, 24.0, 6.0, VERDE);
                    } else if ayuda && es_meta {
                        let a = 0.5 + 0.5 * (cx.t * 5.0).sin();
                        l.rrect_borde(r.crecer(2.0), 26.0, 7.0, alfa(AMARILLO, 0.4 + 0.6 * a));
                    }
                    let lado = r.w.min(r.h) * 0.72 * esc;
                    dibujar_forma(l, f, r.cx(), r.cy() - if gana { 14.0 } else { 0.0 }, lado, c);
                    if gana {
                        l.texto_c(f.titulo(), r.cx(), r.abajo() - 30.0, 32.0, oscurecer(c, 0.35));
                    }
                }
                if self.celebrando > 0.0 {
                    felicitar(l, "¡Muy bien!", AW / 2.0, 478.0, cx.t);
                }
            }
        }
        self.confeti.dibujar(l);
    }
}
