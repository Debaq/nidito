//! Música: un xilófono de 8 barras (Do a Do). Tocar libre (clic, arrastrar o teclas 1-8 / A-K)
//! o seguir una canción: la barra que toca brilla; con el altavoz se escucha la canción entera.

use super::*;
use crate::cx::Son;
use crate::efectos::Confeti;

const NOTAS: [&str; 8] = ["Do", "Re", "Mi", "Fa", "Sol", "La", "Si", "Do"];
/// Teclas de la fila del medio, una por barra.
const TECLAS: &str = "asdfghjk";

struct Cancion {
    titulo: &'static str,
    en: &'static str,
    /// (barra 0-7, duración en negras)
    notas: &'static [(usize, f32)],
}

/// Melodías tradicionales (dominio público).
const CANCIONES: [Cancion; 2] = [
    Cancion {
        titulo: "Estrellita",
        en: "Twinkle Twinkle",
        notas: &[
            (0, 1.0), (0, 1.0), (4, 1.0), (4, 1.0), (5, 1.0), (5, 1.0), (4, 2.0),
            (3, 1.0), (3, 1.0), (2, 1.0), (2, 1.0), (1, 1.0), (1, 1.0), (0, 2.0),
            (4, 1.0), (4, 1.0), (3, 1.0), (3, 1.0), (2, 1.0), (2, 1.0), (1, 2.0),
            (4, 1.0), (4, 1.0), (3, 1.0), (3, 1.0), (2, 1.0), (2, 1.0), (1, 2.0),
            (0, 1.0), (0, 1.0), (4, 1.0), (4, 1.0), (5, 1.0), (5, 1.0), (4, 2.0),
            (3, 1.0), (3, 1.0), (2, 1.0), (2, 1.0), (1, 1.0), (1, 1.0), (0, 2.0),
        ],
    },
    Cancion {
        titulo: "Los pollitos",
        en: "Little Chicks",
        notas: &[
            (0, 1.0), (1, 1.0), (2, 1.0), (3, 1.0), (4, 1.0), (4, 2.0),
            (5, 1.0), (5, 1.0), (5, 1.0), (5, 1.0), (4, 2.0),
            (5, 1.0), (5, 1.0), (5, 1.0), (5, 1.0), (4, 2.0),
            (3, 1.0), (3, 1.0), (3, 1.0), (3, 1.0), (2, 1.0), (2, 1.0),
            (1, 1.0), (1, 1.0), (1, 1.0), (1, 1.0), (0, 2.0),
        ],
    },
];

const NEGRA: f32 = 0.42;

const B_LIBRE: R = R::new(20.0, 14.0, 190.0, 54.0);
fn b_cancion(i: usize) -> R {
    R::new(222.0 + i as f32 * 212.0, 14.0, 200.0, 54.0)
}
const ALTAVOZ: (f32, f32) = (940.0, 41.0);

fn barra(i: usize) -> R {
    let h = 340.0 - i as f32 * 22.0;
    R::new(58.0 + i as f32 * 112.0, 300.0 - h / 2.0, 92.0, h)
}

fn a_tocar() -> (&'static str, &'static str) {
    ("¡Toca el xilófono!", "Play the xylophone!")
}
fn sigue() -> (&'static str, &'static str) {
    ("Toca la barra que brilla.", "Touch the bar that shines.")
}
fn tocaste(c: &Cancion) -> (String, String) {
    (format!("¡Tocaste {}!", c.titulo), format!("You played {}!", c.en))
}

pub fn frases(f: &mut Frases) {
    let (a, b) = a_tocar();
    f.bil(a, b);
    let (a, b) = sigue();
    f.bil(a, b);
    for c in &CANCIONES {
        f.bil(c.titulo, c.en);
        let (a, b) = tocaste(c);
        f.bil(a, b);
    }
}

pub struct Musica {
    /// None = tocar libre; Some(i) = seguir la canción i
    cancion: Option<usize>,
    paso: usize,
    /// golpe de cada barra (segundos que le quedan a la animación)
    golpe: [f32; 8],
    /// barra bajo el dedo mientras se arrastra
    arrastre: Option<usize>,
    /// reproducción de la canción: (tiempo hasta la próxima nota, índice)
    escuchar: Option<(f32, usize)>,
    celebrando: f32,
    chispas: Confeti,
}

impl Musica {
    pub fn nuevo(cx: &mut Cx) -> Box<dyn App> {
        let (a, b) = a_tocar();
        cx.decir_bil(a, b);
        Box::new(Musica { cancion: None, paso: 0, golpe: [0.0; 8], arrastre: None, escuchar: None, celebrando: 0.0, chispas: Confeti::default() })
    }

    fn sonar_barra(&mut self, i: usize, cx: &mut Cx) {
        cx.sonar(Son::Xilo(i));
        self.golpe[i] = 0.25;
        let r = barra(i);
        self.chispas.chispas(r.cx(), r.y + 20.0, 6, &mut cx.rng, Some(ARCOIRIS[i]));
    }

    fn tocar(&mut self, i: usize, cx: &mut Cx) {
        self.sonar_barra(i, cx);
        let Some(c) = self.cancion else { return };
        if self.celebrando > 0.0 || self.escuchar.is_some() {
            return;
        }
        let notas = CANCIONES[c].notas;
        if notas[self.paso].0 == i {
            self.paso += 1;
            if self.paso >= notas.len() {
                self.paso = 0;
                self.celebrando = 3.0;
                cx.sonar(Son::Aplauso);
                self.chispas.lanzar(AW / 2.0, 200.0, 80, &mut cx.rng);
                let (a, b) = tocaste(&CANCIONES[c]);
                cx.decir_bil(&a, &b);
            }
        }
    }

    fn barra_en(p: (f32, f32)) -> Option<usize> {
        (0..8).find(|&i| barra(i).crecer(4.0).dentro(p))
    }

    fn elegir(&mut self, c: Option<usize>, cx: &mut Cx) {
        self.cancion = c;
        self.paso = 0;
        self.escuchar = None;
        self.celebrando = 0.0;
        cx.sonar(Son::Clic);
        match c {
            None => {
                let (a, b) = a_tocar();
                cx.decir_bil(a, b);
            }
            Some(i) => {
                let (a, b) = sigue();
                cx.decir_v(&[CANCIONES[i].titulo, a], &[CANCIONES[i].en, b]);
            }
        }
    }
}

impl App for Musica {
    fn animando(&self) -> bool {
        self.golpe.iter().any(|&g| g > 0.0) || self.chispas.activo() || self.escuchar.is_some() || self.celebrando > 0.0 || self.cancion.is_some()
    }

    fn evento(&mut self, ev: &Ev, cx: &mut Cx) {
        match ev {
            Ev::Clic(x, y) => {
                let p = (*x, *y);
                if B_LIBRE.dentro(p) {
                    self.elegir(None, cx);
                    return;
                }
                for i in 0..CANCIONES.len() {
                    if b_cancion(i).dentro(p) {
                        self.elegir(Some(i), cx);
                        return;
                    }
                }
                if self.cancion.is_some() && dist(p, ALTAVOZ) < 30.0 {
                    // escuchar la canción (o pararla)
                    self.escuchar = if self.escuchar.is_some() { None } else { Some((0.3, 0)) };
                    self.paso = 0;
                    return;
                }
                if let Some(i) = Musica::barra_en(p) {
                    self.arrastre = Some(i);
                    self.tocar(i, cx);
                }
            }
            Ev::Mueve(x, y) => {
                if cx.apretado && self.arrastre.is_some() {
                    let b = Musica::barra_en((*x, *y));
                    if let Some(i) = b {
                        if Some(i) != self.arrastre {
                            self.tocar(i, cx);
                        }
                    }
                    if b.is_some() {
                        self.arrastre = b;
                    }
                }
            }
            Ev::Suelta(..) => self.arrastre = None,
            Ev::Texto(t) => {
                for ch in t.chars() {
                    let ch = ch.to_ascii_lowercase();
                    let i = ch.to_digit(10).filter(|d| (1..=8).contains(d)).map(|d| d as usize - 1).or_else(|| TECLAS.find(ch));
                    if let Some(i) = i {
                        self.tocar(i, cx);
                    }
                }
            }
            _ => {}
        }
    }

    fn actualizar(&mut self, dt: f32, cx: &mut Cx) {
        self.chispas.actualizar(dt);
        for g in &mut self.golpe {
            *g = (*g - dt).max(0.0);
        }
        if self.celebrando > 0.0 {
            self.celebrando -= dt;
        }
        if let (Some((t, k)), Some(c)) = (&mut self.escuchar, self.cancion) {
            *t -= dt;
            if *t <= 0.0 {
                let notas = CANCIONES[c].notas;
                let (i, d) = notas[*k];
                *t += d * NEGRA;
                *k += 1;
                let fin = *k >= notas.len();
                self.sonar_barra(i, cx);
                if fin {
                    self.escuchar = None;
                }
            }
        }
    }

    fn dibujar(&mut self, l: &mut Lienzo, cx: &Cx) {
        l.degradado_v(0.0, 0.0, AW, AH, rgb(240, 236, 255), rgb(222, 214, 250));
        l.rect(0.0, 0.0, AW, 82.0, rgb(230, 224, 252));
        boton_texto(l, B_LIBRE, "Tocar libre", MORADO, self.cancion.is_none(), cx);
        for (i, c) in CANCIONES.iter().enumerate() {
            boton_texto(l, b_cancion(i), c.titulo, if i == 0 { NARANJO } else { AMARILLO }, self.cancion == Some(i), cx);
        }
        if self.cancion.is_some() {
            boton_altavoz(l, ALTAVOZ.0, ALTAVOZ.1, 28.0, if self.escuchar.is_some() { ROJO } else { MORADO }, cx);
        }
        // rieles de madera
        for k in [0.2, 0.8] {
            let pts: Vec<(f32, f32)> = (0..8).map(|i| {
                let r = barra(i);
                (r.cx(), r.y + r.h * k)
            }).collect();
            let (a, b) = (pts[0], pts[7]);
            let m = (b.1 - a.1) / (b.0 - a.0);
            l.linea(a.0 - 60.0, a.1 - 60.0 * m, b.0 + 60.0, b.1 + 60.0 * m, 16.0, rgb(150, 100, 60));
        }
        // la barra que toca en la canción
        let objetivo = match self.cancion {
            Some(c) if self.escuchar.is_none() && self.celebrando <= 0.0 => Some(CANCIONES[c].notas[self.paso].0),
            _ => None,
        };
        for i in 0..8 {
            let g = self.golpe[i] / 0.25;
            let mut r = barra(i).mover(0.0, 5.0 * g);
            let c = ARCOIRIS[i];
            if objetivo == Some(i) {
                let a = 0.5 + 0.5 * (cx.t * 6.0).sin();
                l.rrect_r(r.crecer(8.0 + 3.0 * a), 18.0, alfa(AMARILLO, 0.45 + 0.4 * a));
                // flechita encima
                let (fx, fy) = (r.cx(), r.y - 22.0 - 6.0 * a);
                l.poligono(&[(fx - 16.0, fy - 14.0), (fx + 16.0, fy - 14.0), (fx, fy + 6.0)], NARANJO);
            } else if r.dentro(cx.rl) {
                r = r.crecer(2.0);
            }
            l.rrect_r(r.mover(0.0, 6.0), 12.0, oscurecer(c, 0.35));
            l.rrect_r(r, 12.0, if g > 0.0 { aclarar(c, 0.35 * g) } else { c });
            l.rrect(r.x + 8.0, r.y + 6.0, r.w - 16.0, 14.0, 7.0, alfa(BLANCO, 0.3));
            for k in [0.2, 0.8] {
                l.circulo(r.cx(), r.y + r.h * k, 7.0, rgb(200, 200, 210));
                l.circulo(r.cx(), r.y + r.h * k, 4.0, rgb(150, 150, 165));
            }
            l.texto_c_borde(NOTAS[i], r.cx(), r.cy(), 30.0, BLANCO, oscurecer(c, 0.45), 2.0);
        }
        // progreso de la canción
        if let Some(c) = self.cancion {
            let n = CANCIONES[c].notas.len();
            let ancho = (n as f32 - 1.0) * 18.0;
            for k in 0..n {
                let x = AW / 2.0 - ancho / 2.0 + k as f32 * 18.0;
                let hecho = k < self.paso || self.celebrando > 0.0;
                l.circulo(x, AH - 22.0, 6.0, if hecho { ARCOIRIS[CANCIONES[c].notas[k].0] } else { rgb(200, 192, 230) });
            }
        }
        if self.celebrando > 0.0 {
            felicitar(l, "¡Bravo!", AW / 2.0, 120.0, cx.t);
        }
        self.chispas.dibujar(l);
    }
}
