//! Gatitos del fondo del escritorio, dibujados con figuras (sin imágenes).
//! Cada gato se dibuja en su propia capa: primero la silueta, después las manchas
//! (solo donde hay gato) y al final ojos, nariz y bigotes.

use crate::gfx::*;
use tiny_skia::*;

#[derive(Clone, Copy)]
pub enum Pelaje {
    /// negro con pechera, hocico y patitas blancas
    Smoking,
    /// gris oscuro casi negro, con rayitas y manchas blancas
    GrisRayado,
    /// naranjo atigrado
    Naranjo,
    /// naranjo pálido parejo, sin rayas (panza del mismo color)
    NaranjoPalido,
    /// calico más naranja que blanca (pecho, hocico y patitas blancas, manchas negras)
    CalicoNaranja,
    /// calico más negra que blanca (manchas naranjas, pecho y patitas blancas)
    CalicoNegra,
    /// gris con rayitas
    Plomo,
    /// blanco con una mancha negra en la cola
    Blanco,
}

#[derive(Clone, Copy)]
pub struct Gato {
    pub pelaje: Pelaje,
    pub peludo: bool,
    /// true = mira hacia la izquierda (cola al otro lado)
    pub espejo: bool,
    pub ojos: C,
    /// ancho del cuerpo: < 1 flaquito, > 1 gordito
    pub ancho: f32,
}

const NARANJO_G: C = rgb(242, 152, 62);
const NARANJO_O: C = rgb(206, 112, 38);
const NEGRO_G: C = rgb(48, 46, 54);
const BLANCO_G: C = rgb(250, 248, 244);
const PLOMO_G: C = rgb(150, 156, 168);
const ROSA: C = rgb(255, 160, 175);

/// Capa de dibujo de un gato, con coordenadas en "unidades de gato" (alto ~125).
struct Capa {
    pix: Pixmap,
    s: f32,
    ox: f32,
    oy: f32,
    m: f32,
}

impl Capa {
    fn t(&self) -> Transform {
        Transform::from_row(self.s * self.m, 0.0, 0.0, self.s, self.ox, self.oy)
    }
    fn pintar(&mut self, ruta: Option<Path>, c: C, modo: BlendMode) {
        if let Some(p) = ruta {
            let mut paint = Paint::default();
            paint.set_color_rgba8(c[0], c[1], c[2], c[3]);
            paint.anti_alias = true;
            paint.blend_mode = modo;
            let t = self.t();
            self.pix.fill_path(&p, &paint, FillRule::Winding, t, None);
        }
    }
    fn trazar(&mut self, pts: &[(f32, f32)], ancho: f32, c: C, modo: BlendMode) {
        let mut pb = PathBuilder::new();
        if let Some(&(x, y)) = pts.first() {
            pb.move_to(x, y);
        }
        if pts.len() == 3 {
            pb.quad_to(pts[1].0, pts[1].1, pts[2].0, pts[2].1);
        } else {
            for &(x, y) in &pts[1..] {
                pb.line_to(x, y);
            }
        }
        if let Some(p) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color_rgba8(c[0], c[1], c[2], c[3]);
            paint.anti_alias = true;
            paint.blend_mode = modo;
            let st = Stroke { width: ancho, line_cap: LineCap::Round, line_join: LineJoin::Round, ..Default::default() };
            let t = self.t();
            self.pix.stroke_path(&p, &paint, &st, t, None);
        }
    }
    fn ovalo(&mut self, x: f32, y: f32, rx: f32, ry: f32, c: C, modo: BlendMode) {
        let r = Rect::from_xywh(x - rx, y - ry, rx * 2.0, ry * 2.0).map(PathBuilder::from_oval).flatten();
        self.pintar(r, c, modo);
    }
    fn circulo(&mut self, x: f32, y: f32, r: f32, c: C, modo: BlendMode) {
        self.ovalo(x, y, r, r, c, modo);
    }
    fn poligono(&mut self, pts: &[(f32, f32)], c: C, modo: BlendMode) {
        self.pintar(ruta_poligono(pts), c, modo);
    }
}

const N: BlendMode = BlendMode::SourceOver;
/// Pinta solo donde ya hay gato.
const ENCIMA: BlendMode = BlendMode::SourceAtop;

fn pelo_base(p: Pelaje) -> C {
    match p {
        Pelaje::Smoking => NEGRO_G,
        Pelaje::Naranjo | Pelaje::CalicoNaranja => NARANJO_G,
        Pelaje::NaranjoPalido => rgb(247, 192, 128),
        Pelaje::CalicoNegra => NEGRO_G,
        Pelaje::Plomo => PLOMO_G,
        Pelaje::GrisRayado => rgb(70, 70, 78),
        Pelaje::Blanco => BLANCO_G,
    }
}

/// Cola: curva desde el costado hacia arriba.
const COLA: [(f32, f32); 3] = [(24.0, -10.0), (62.0, -14.0), (50.0, -56.0)];

/// Silueta completa (cola, cuerpo, patas, cabeza, orejas), agrandada en `g` (para el contorno).
fn silueta(c: &mut Capa, g: &Gato, color: C, g_: f32) {
    let peludo = g.peludo;
    let cola_w = if peludo { 19.0 } else { 11.0 };
    c.trazar(&COLA, cola_w + g_ * 2.0, color, N);
    if peludo {
        for k in 0..6 {
            let t = k as f32 / 5.0;
            let (x, y) = bezier(t);
            c.circulo(x + if k % 2 == 0 { 3.0 } else { -3.0 }, y, 7.0 + g_, color, N);
        }
    }
    // cuerpo y patas
    let w = g.ancho;
    c.ovalo(0.0, -32.0, 30.0 * w + g_, 34.0 + g_, color, N);
    c.ovalo(-11.0 * w, -4.0, 10.0 + g_, 6.5 + g_, color, N);
    c.ovalo(11.0 * w, -4.0, 10.0 + g_, 6.5 + g_, color, N);
    if peludo {
        for k in 0..9 {
            let a = 0.5 + k as f32 * 0.32; // contorno del cuerpo, lados y abajo
            for lado in [-1.0, 1.0] {
                c.circulo(lado * a.sin() * 29.0 * w, -32.0 + a.cos() * -31.0 + 6.0, 7.0 + g_, color, N);
            }
        }
        // melena del pecho
        for k in 0..5 {
            c.circulo(-14.0 + k as f32 * 7.0, -52.0, 8.0 + g_, color, N);
        }
    }
    // orejas
    c.poligono(&[(-30.0 - g_, -84.0), (-25.0, -116.0 - g_ * 1.5), (-5.0 + g_, -100.0)], color, N);
    c.poligono(&[(30.0 + g_, -84.0), (25.0, -116.0 - g_ * 1.5), (5.0 - g_, -100.0)], color, N);
    // cabeza
    c.ovalo(0.0, -79.0, 32.0 + g_, 27.0 + g_, color, N);
    if peludo {
        for lado in [-1.0, 1.0] {
            c.poligono(&[(lado * 28.0, -84.0), (lado * (38.0 + g_), -76.0), (lado * 31.0, -72.0), (lado * (36.0 + g_), -65.0), (lado * 24.0, -61.0)], color, N);
        }
    }
}

fn bezier(t: f32) -> (f32, f32) {
    let [a, b, c] = COLA;
    let u = 1.0 - t;
    (u * u * a.0 + 2.0 * u * t * b.0 + t * t * c.0, u * u * a.1 + 2.0 * u * t * b.1 + t * t * c.1)
}

fn manchas(c: &mut Capa, g: &Gato) {
    match g.pelaje {
        Pelaje::Smoking => {
            c.ovalo(0.0, -40.0, 15.0, 22.0, BLANCO_G, ENCIMA); // pechera
            c.ovalo(-11.0, -4.0, 11.0, 8.0, BLANCO_G, ENCIMA); // patitas
            c.ovalo(11.0, -4.0, 11.0, 8.0, BLANCO_G, ENCIMA);
            c.poligono(&[(0.0, -88.0), (-13.0, -60.0), (13.0, -60.0)], BLANCO_G, ENCIMA); // hocico
        }
        Pelaje::GrisRayado => {
            // rayitas casi negras (frente, costados y anillos en la cola)
            let raya = rgb(36, 36, 42);
            for k in [-1.0, 0.0, 1.0] {
                c.trazar(&[(k * 8.0, -104.0), (k * 7.0, -93.0)], 3.0, raya, ENCIMA);
            }
            for k in 0..3 {
                let y = -48.0 + k as f32 * 12.0;
                c.trazar(&[(-36.0, y), (-28.0, y + 3.0), (-21.0, y + 1.0)], 3.5, raya, ENCIMA);
                c.trazar(&[(36.0, y), (28.0, y + 3.0), (21.0, y + 1.0)], 3.5, raya, ENCIMA);
            }
            for t in [0.45, 0.7, 0.92] {
                let (x, y) = bezier(t);
                c.circulo(x, y, 5.0, raya, ENCIMA);
            }
            // manchas blancas: hocico, pecho, un costado y patitas
            c.ovalo(0.0, -67.0, 12.0, 8.0, BLANCO_G, ENCIMA);
            c.ovalo(0.0, -42.0, 12.0, 17.0, BLANCO_G, ENCIMA);
            c.circulo(-21.0, -22.0, 10.0, BLANCO_G, ENCIMA);
            c.ovalo(-11.0, -4.0, 11.0, 8.0, BLANCO_G, ENCIMA);
            c.ovalo(11.0, -4.0, 11.0, 8.0, BLANCO_G, ENCIMA);
        }
        Pelaje::Naranjo => {
            c.ovalo(0.0, -42.0, 14.0, 20.0, rgb(252, 214, 160), ENCIMA); // pecho claro
            c.ovalo(0.0, -68.0, 13.0, 8.0, rgb(252, 214, 160), ENCIMA); // hocico claro
            for k in [-1.0, 0.0, 1.0] {
                c.trazar(&[(k * 8.0, -104.0), (k * 7.0, -92.0)], 3.5, NARANJO_O, ENCIMA); // frente
            }
            for k in 0..3 {
                let y = -46.0 + k as f32 * 12.0;
                c.trazar(&[(-31.0, y), (-24.0, y + 3.0), (-18.0, y + 1.0)], 4.0, NARANJO_O, ENCIMA);
                c.trazar(&[(31.0, y), (24.0, y + 3.0), (18.0, y + 1.0)], 4.0, NARANJO_O, ENCIMA);
            }
            for t in [0.45, 0.7, 0.92] {
                let (x, y) = bezier(t);
                c.circulo(x, y, 5.5, NARANJO_O, ENCIMA);
            }
        }
        Pelaje::NaranjoPalido => {}
        Pelaje::CalicoNaranja => {
            // mucho naranjo, un poco de negro y blanco en pecho, hocico y patitas
            c.circulo(19.0, -92.0, 13.0, NEGRO_G, ENCIMA);
            c.circulo(-18.0, -24.0, 13.0, NEGRO_G, ENCIMA);
            let (x, y) = bezier(0.85);
            c.circulo(x, y, 10.0, NEGRO_G, ENCIMA);
            c.ovalo(0.0, -42.0, 13.0, 19.0, BLANCO_G, ENCIMA);
            c.ovalo(0.0, -66.0, 11.0, 7.0, BLANCO_G, ENCIMA);
            c.ovalo(-11.0, -4.0, 11.0, 8.0, BLANCO_G, ENCIMA);
            c.ovalo(11.0, -4.0, 11.0, 8.0, BLANCO_G, ENCIMA);
        }
        Pelaje::CalicoNegra => {
            // mucho negro, manchas naranjas y un poco de blanco
            c.circulo(-18.0, -96.0, 15.0, NARANJO_G, ENCIMA);
            c.circulo(20.0, -30.0, 14.0, NARANJO_G, ENCIMA);
            let (x, y) = bezier(0.6);
            c.circulo(x, y, 11.0, NARANJO_G, ENCIMA);
            c.ovalo(0.0, -44.0, 12.0, 17.0, BLANCO_G, ENCIMA);
            c.poligono(&[(0.0, -84.0), (-10.0, -62.0), (10.0, -62.0)], BLANCO_G, ENCIMA);
            c.ovalo(-11.0, -4.0, 11.0, 8.0, BLANCO_G, ENCIMA);
            c.ovalo(11.0, -4.0, 11.0, 8.0, BLANCO_G, ENCIMA);
        }
        Pelaje::Plomo => {
            let raya = rgb(112, 117, 130);
            c.ovalo(0.0, -68.0, 12.0, 7.0, rgb(196, 200, 210), ENCIMA);
            c.ovalo(0.0, -42.0, 13.0, 17.0, rgb(176, 181, 192), ENCIMA);
            for k in [-1.0, 0.0, 1.0] {
                c.trazar(&[(k * 8.0, -104.0), (k * 7.0, -93.0)], 3.0, raya, ENCIMA);
            }
            for k in 0..3 {
                let y = -48.0 + k as f32 * 12.0;
                c.trazar(&[(-38.0, y), (-30.0, y + 3.0), (-23.0, y + 1.0)], 3.5, raya, ENCIMA);
                c.trazar(&[(38.0, y), (30.0, y + 3.0), (23.0, y + 1.0)], 3.5, raya, ENCIMA);
            }
            for t in [0.45, 0.7, 0.92] {
                let (x, y) = bezier(t);
                c.circulo(x, y, 5.0, raya, ENCIMA);
            }
        }
        Pelaje::Blanco => {
            // mancha negra en la cola
            let (x, y) = bezier(0.7);
            c.circulo(x, y, 10.0, NEGRO_G, ENCIMA);
        }
    }
}

fn cara(c: &mut Capa, g: &Gato) {
    let oscuro = matches!(g.pelaje, Pelaje::Smoking | Pelaje::CalicoNegra | Pelaje::GrisRayado);
    // interior de las orejas (en el smoking y la calico una oreja es de color)
    c.poligono(&[(-25.0, -90.0), (-23.0, -108.0), (-12.0, -98.0)], ROSA, ENCIMA);
    c.poligono(&[(25.0, -90.0), (23.0, -108.0), (12.0, -98.0)], ROSA, ENCIMA);
    // mejillas
    c.ovalo(-19.0, -68.0, 6.0, 3.5, alfa(ROSA, 0.6), N);
    c.ovalo(19.0, -68.0, 6.0, 3.5, alfa(ROSA, 0.6), N);
    // ojos: iris, pupila, brillo
    for lado in [-1.0, 1.0] {
        let (x, y) = (lado * 11.5, -80.0);
        c.ovalo(x, y, 6.2, 7.2, rgb(30, 30, 38), N);
        c.ovalo(x, y, 5.0, 6.0, g.ojos, N);
        c.ovalo(x, y + 0.5, 2.4, 5.0, rgb(20, 20, 26), N);
        c.circulo(x + 1.8, y - 2.8, 1.8, BLANCO, N);
    }
    // nariz y boca
    c.poligono(&[(-3.5, -72.0), (3.5, -72.0), (0.0, -68.0)], rgb(240, 120, 140), N);
    let boca = if oscuro { rgb(120, 120, 130) } else { rgb(110, 80, 80) };
    c.trazar(&[(0.0, -68.0), (-2.5, -63.5), (-6.0, -66.5)], 1.5, boca, N);
    c.trazar(&[(0.0, -68.0), (2.5, -63.5), (6.0, -66.5)], 1.5, boca, N);
    // bigotes
    let bigote = if oscuro { alfa(BLANCO, 0.85) } else { alfa(rgb(90, 90, 100), 0.7) };
    for lado in [-1.0, 1.0] {
        for (dy, dy2) in [(-1.0, -5.0), (1.5, 1.5), (4.0, 7.0)] {
            c.trazar(&[(lado * 12.0, -68.0 + dy), (lado * 34.0, -68.0 + dy2)], 1.0, bigote, N);
        }
    }
}

/// Dibuja un gato sentado con las patas en (x, y) y alto aproximado 125*s.
pub fn dibujar(l: &mut Lienzo, g: &Gato, x: f32, y: f32, s: f32) {
    // sombra en el pasto
    l.ovalo(x + if g.espejo { -8.0 } else { 8.0 } * s, y, 40.0 * s, 7.0 * s, alfa(rgb(30, 80, 30), 0.25));
    let (w, h) = ((150.0 * s).ceil() as u32, (150.0 * s).ceil() as u32);
    let Some(pix) = Pixmap::new(w, h) else { return };
    let mut c = Capa { pix, s, ox: w as f32 / 2.0, oy: h as f32 - 6.0 * s, m: if g.espejo { -1.0 } else { 1.0 } };
    let base = pelo_base(g.pelaje);
    let contorno = if matches!(g.pelaje, Pelaje::Smoking | Pelaje::CalicoNegra | Pelaje::GrisRayado) { rgb(25, 24, 30) } else { oscurecer(base, 0.45) };
    silueta(&mut c, g, contorno, 1.8);
    silueta(&mut c, g, base, 0.0);
    manchas(&mut c, g);
    cara(&mut c, g);
    let pp = PixmapPaint { quality: FilterQuality::Nearest, ..Default::default() };
    l.pix.draw_pixmap((x - c.ox).round() as i32, (y - c.oy).round() as i32, c.pix.as_ref(), &pp, Transform::identity(), None);
}

/// Lugares en el pasto donde se puede sentar un gato: (x, y de las patas, escala).
/// Zona derecha, para no tapar los íconos ni el reloj. Más arriba = más lejos = más chico.
const LUGARES: [(f32, f32, f32); 12] = [
    (492.0, 546.0, 0.60),
    (566.0, 516.0, 0.64),
    (636.0, 550.0, 0.60),
    (710.0, 505.0, 0.56),
    (780.0, 548.0, 0.62),
    (852.0, 500.0, 0.58),
    (918.0, 548.0, 0.60),
    (982.0, 512.0, 0.56),
    (412.0, 546.0, 0.60),
    (676.0, 468.0, 0.50),
    (818.0, 466.0, 0.50),
    (930.0, 480.0, 0.50),
];

/// Los gatitos de la casa: cada uno ocupa un lugar y de vez en cuando se cambia a uno libre.
pub struct Manada {
    gatos: [Gato; 8],
    lugar: [usize; 8],
}

impl Manada {
    pub fn nueva() -> Manada {
        let verde = rgb(80, 170, 70);
        let azul = rgb(90, 160, 230);
        let ambar = rgb(225, 170, 50);
        let g = |pelaje, peludo, espejo, ojos, ancho| Gato { pelaje, peludo, espejo, ojos, ancho };
        Manada {
            gatos: [
                g(Pelaje::Smoking, false, false, verde, 1.0),
                g(Pelaje::Naranjo, true, true, ambar, 1.0),
                g(Pelaje::CalicoNaranja, false, false, ambar, 1.0),
                g(Pelaje::Plomo, false, true, verde, 1.3),
                g(Pelaje::Blanco, false, false, azul, 1.0),
                g(Pelaje::CalicoNegra, true, true, verde, 1.0),
                g(Pelaje::NaranjoPalido, false, false, ambar, 0.8),
                g(Pelaje::GrisRayado, false, true, verde, 1.0),
            ],
            lugar: [0, 1, 2, 3, 4, 5, 6, 7],
        }
    }

    /// Un gato (elegido con `a`) se cambia a un lugar libre (elegido con `b`). a, b en [0, 1).
    pub fn mover(&mut self, a: f32, b: f32) {
        let k = ((a * 8.0) as usize).min(7);
        let libres: Vec<usize> = (0..LUGARES.len()).filter(|i| !self.lugar.contains(i)).collect();
        if libres.is_empty() {
            return;
        }
        let nuevo = libres[((b * libres.len() as f32) as usize).min(libres.len() - 1)];
        // mira hacia donde se fue
        self.gatos[k].espejo = LUGARES[nuevo].0 < LUGARES[self.lugar[k]].0;
        self.lugar[k] = nuevo;
    }

    pub fn dibujar(&self, l: &mut Lienzo) {
        // de atrás (más arriba) hacia adelante
        let mut orden: Vec<usize> = (0..8).collect();
        orden.sort_by(|&a, &b| LUGARES[self.lugar[a]].1.total_cmp(&LUGARES[self.lugar[b]].1));
        for k in orden {
            let (x, y, s) = LUGARES[self.lugar[k]];
            dibujar(l, &self.gatos[k], x, y, s);
        }
    }
}
