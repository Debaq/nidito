//! Teclas: cada tecla hace aparecer una figura o letra con sonido.

use super::*;
use crate::cx::Son;
use crate::dibujos::{self, Obj, CONTABLES};
use crate::efectos::Confeti;
use std::f32::consts::{FRAC_PI_2, TAU};
use tiny_skia::Pixmap;

#[derive(Clone, Copy)]
enum Tipo {
    Letra(char),
    Circulo,
    Estrella,
    Corazon,
    Cuadrado,
    Triangulo,
    Objeto(Obj),
}

struct Forma {
    x: f32,
    y: f32,
    c: C,
    tipo: Tipo,
    t: f32,
    rot: f32,
    tam: f32,
}

const VIDA: f32 = 5.0;

pub struct Teclas {
    fondo: Pixmap,
    formas: Vec<Forma>,
    chispas: Confeti,
    ultimo_rastro: f32,
}

pub fn frases(_f: &mut Frases) {}

impl Teclas {
    pub fn nuevo(cx: &mut Cx) -> Box<dyn App> {
        let mut fondo = Pixmap::new(AW as u32, AH as u32).unwrap();
        fondo.degradado_v(0.0, 0.0, AW, AH, rgb(27, 31, 75), rgb(70, 46, 110));
        for _ in 0..90 {
            let (x, y, r) = (cx.rng.rango(0.0, AW), cx.rng.rango(0.0, AH), cx.rng.rango(0.8, 2.2));
            fondo.circulo(x, y, r, alfa(BLANCO, cx.rng.rango(0.3, 0.9)));
        }
        Box::new(Teclas { fondo, formas: Vec::new(), chispas: Confeti::default(), ultimo_rastro: 0.0 })
    }

    fn crear(&mut self, tipo: Tipo, x: f32, y: f32, cx: &mut Cx) {
        let c = *cx.rng.elegir(&ARCOIRIS);
        let tam = match tipo {
            Tipo::Letra(_) => cx.rng.rango(130.0, 190.0),
            _ => cx.rng.rango(70.0, 140.0),
        };
        self.formas.push(Forma { x, y, c, tipo, t: 0.0, rot: cx.rng.rango(0.0, TAU), tam });
        if self.formas.len() > 24 {
            self.formas.remove(0);
        }
        self.chispas.chispas(x, y, 12, &mut cx.rng, Some(c));
    }

    fn al_azar(cx: &mut Cx) -> Tipo {
        match cx.rng.n(6) {
            0 => Tipo::Circulo,
            1 => Tipo::Estrella,
            2 => Tipo::Corazon,
            3 => Tipo::Cuadrado,
            4 => Tipo::Triangulo,
            _ => Tipo::Objeto(*cx.rng.elegir(&CONTABLES)),
        }
    }
}

impl App for Teclas {
    fn animando(&self) -> bool {
        !self.formas.is_empty() || self.chispas.activo()
    }

    fn evento(&mut self, ev: &Ev, cx: &mut Cx) {
        let pos = |cx: &mut Cx| (cx.rng.rango(110.0, AW - 110.0), cx.rng.rango(100.0, AH - 100.0));
        match ev {
            Ev::Texto(t) => {
                for c in t.chars().filter(|c| !c.is_control()) {
                    let (x, y) = pos(cx);
                    if c.is_alphanumeric() {
                        self.crear(Tipo::Letra(mayus(c)), x, y, cx);
                        cx.sonar(Son::Pop);
                        if let Some(n) = nombre_letra(c) {
                            cx.decir_bil(&n, &mayus(base(c)).to_string());
                        }
                    } else {
                        let tp = Teclas::al_azar(cx);
                        self.crear(tp, x, y, cx);
                        let n = cx.rng.n(8);
                cx.sonar(Son::Nota(n));
                    }
                }
            }
            Ev::Tecla(_) => {
                let (x, y) = pos(cx);
                let tp = Teclas::al_azar(cx);
                self.crear(tp, x, y, cx);
                let n = cx.rng.n(8);
                cx.sonar(Son::Nota(n));
            }
            Ev::Clic(x, y) => {
                let tp = Teclas::al_azar(cx);
                self.crear(tp, *x, *y, cx);
                let nota = ((*x / AW) * 8.0) as usize;
                cx.sonar(Son::Nota(nota.min(7)));
            }
            Ev::Mueve(x, y) => {
                if cx.t - self.ultimo_rastro > 0.05 {
                    self.ultimo_rastro = cx.t;
                    self.chispas.chispas(*x, *y, 2, &mut cx.rng, None);
                }
            }
            _ => {}
        }
    }

    fn actualizar(&mut self, dt: f32, _cx: &mut Cx) {
        for f in &mut self.formas {
            f.t += dt;
            f.rot += dt * 0.4;
        }
        self.formas.retain(|f| f.t < VIDA);
        self.chispas.actualizar(dt);
    }

    fn dibujar(&mut self, l: &mut Lienzo, _cx: &Cx) {
        copiar_opaco(&mut l.pix, &self.fondo, 0, 0);
        for f in &self.formas {
            let esc = ease_out_back(f.t / 0.35);
            let a = if f.t > VIDA - 1.0 { VIDA - f.t } else { 1.0 };
            let s = f.tam * esc;
            let c = alfa(f.c, a);
            match f.tipo {
                Tipo::Letra(ch) => {
                    let tam = (s / 8.0).round() * 8.0; // tamaños discretos para la caché de glifos
                    if tam >= 8.0 {
                        l.texto_c_borde(&ch.to_string(), f.x, f.y, tam, c, alfa(BLANCO, a), 4.0);
                    }
                }
                Tipo::Circulo => {
                    l.circulo(f.x, f.y, s * 0.45, c);
                    l.circulo(f.x - s * 0.15, f.y - s * 0.15, s * 0.1, alfa(BLANCO, 0.5 * a));
                }
                Tipo::Estrella => l.estrella(f.x, f.y, s * 0.55, f.rot - FRAC_PI_2, c),
                Tipo::Corazon => l.corazon(f.x, f.y, s, c),
                Tipo::Cuadrado => {
                    let (sn, cs) = f.rot.sin_cos();
                    let h = s * 0.4;
                    let p = |x: f32, y: f32| (f.x + x * cs - y * sn, f.y + x * sn + y * cs);
                    l.poligono(&[p(-h, -h), p(h, -h), p(h, h), p(-h, h)], c);
                }
                Tipo::Triangulo => {
                    let pts: Vec<(f32, f32)> =
                        (0..3).map(|i| f.rot + i as f32 * TAU / 3.0).map(|an| (f.x + an.cos() * s * 0.5, f.y + an.sin() * s * 0.5)).collect();
                    l.poligono(&pts, c);
                }
                Tipo::Objeto(o) => {
                    if a > 0.5 {
                        dibujos::dibujar(l, o, f.x, f.y, s);
                    }
                }
            }
        }
        self.chispas.dibujar(l);
    }
}
