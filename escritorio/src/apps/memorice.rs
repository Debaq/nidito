//! Memorice: encontrar parejas. 3, 4 y luego 6 pares.

use super::*;
use crate::cx::Son;
use crate::dibujos::{self, Obj, VOCABULARIO};
use crate::efectos::Confeti;
use std::f32::consts::PI;

struct Carta {
    obj: Obj,
    r: R,
    cara: bool,
    giro: f32,
    hecha: bool,
}

pub struct Memorice {
    cartas: Vec<Carta>,
    abiertas: Vec<usize>,
    espera: f32,
    nivel: usize,
    celebrando: f32,
    confeti: Confeti,
}

pub fn frases(f: &mut Frases) {
    f.bil("¡Pareja!", "A pair!");
    f.bil("¡Ganaste!", "You win!");
    f.bil("Busca las parejas.", "Find the pairs.");
}

impl Memorice {
    pub fn nuevo(cx: &mut Cx) -> Box<dyn App> {
        let mut m = Memorice { cartas: vec![], abiertas: vec![], espera: 0.0, nivel: 0, celebrando: 0.0, confeti: Confeti::default() };
        m.repartir(cx);
        cx.decir_bil("Busca las parejas.", "Find the pairs.");
        Box::new(m)
    }

    fn repartir(&mut self, cx: &mut Cx) {
        let pares = [3, 4, 6][self.nivel.min(2)];
        let (cols, filas) = match pares {
            3 => (3, 2),
            4 => (4, 2),
            _ => (4, 3),
        };
        let mut objs: Vec<Obj> = VOCABULARIO.to_vec();
        cx.rng.barajar(&mut objs);
        let mut lista: Vec<Obj> = objs[..pares].iter().flat_map(|o| [*o, *o]).collect();
        cx.rng.barajar(&mut lista);
        let gap = 18.0;
        let h = ((AH - 40.0 - gap * (filas - 1) as f32) / filas as f32).min(200.0);
        let w = (h * 0.82).min((AW - 60.0 - gap * (cols - 1) as f32) / cols as f32);
        let tw = cols as f32 * w + (cols - 1) as f32 * gap;
        let th = filas as f32 * h + (filas - 1) as f32 * gap;
        let (x0, y0) = (AW / 2.0 - tw / 2.0, AH / 2.0 - th / 2.0);
        self.cartas = lista
            .into_iter()
            .enumerate()
            .map(|(i, obj)| {
                let (c, f) = (i % cols, i / cols);
                Carta { obj, r: R::new(x0 + c as f32 * (w + gap), y0 + f as f32 * (h + gap), w, h), cara: false, giro: 0.0, hecha: false }
            })
            .collect();
        self.abiertas.clear();
    }
}

impl App for Memorice {
    fn animando(&self) -> bool {
        self.espera > 0.0 || self.celebrando > 0.0 || self.confeti.activo() || self.cartas.iter().any(|c| c.giro > 0.0 && c.giro < 1.0)
    }

    fn evento(&mut self, ev: &Ev, cx: &mut Cx) {
        let Ev::Clic(x, y) = ev else { return };
        if self.espera > 0.0 || self.celebrando > 0.0 || self.abiertas.len() >= 2 {
            return;
        }
        let Some(i) = self.cartas.iter().position(|c| c.r.dentro((*x, *y))) else { return };
        if self.cartas[i].cara || self.cartas[i].hecha {
            return;
        }
        self.cartas[i].cara = true;
        self.abiertas.push(i);
        cx.sonar(Son::Clic);
        let obj = self.cartas[i].obj;
        if self.abiertas.len() == 2 {
            let (a, b) = (self.abiertas[0], self.abiertas[1]);
            if self.cartas[a].obj == self.cartas[b].obj {
                self.cartas[a].hecha = true;
                self.cartas[b].hecha = true;
                self.abiertas.clear();
                cx.sonar(Son::Ding);
                let r = self.cartas[b].r;
                self.confeti.chispas(r.cx(), r.cy(), 25, &mut cx.rng, None);
                if self.cartas.iter().all(|c| c.hecha) {
                    self.celebrando = 3.2;
                    self.nivel += 1;
                    cx.sonar(Son::Fanfarria);
                    self.confeti.lanzar(AW / 2.0, AH / 2.0, 100, &mut cx.rng);
                    cx.decir_bil("¡Ganaste!", "You win!");
                } else {
                    cx.decir_v(&[obj.singular(), "¡Pareja!"], &[obj.ingles(), "A pair!"]);
                }
            } else {
                self.espera = 1.3;
                cx.decir_obj(obj);
            }
        } else {
            cx.decir_obj(obj);
        }
    }

    fn actualizar(&mut self, dt: f32, cx: &mut Cx) {
        for c in &mut self.cartas {
            let meta = if c.cara || c.hecha { 1.0 } else { 0.0 };
            if c.giro < meta {
                c.giro = (c.giro + dt * 3.5).min(1.0);
            } else if c.giro > meta {
                c.giro = (c.giro - dt * 3.5).max(0.0);
            }
        }
        if self.espera > 0.0 {
            self.espera -= dt;
            if self.espera <= 0.0 {
                for &i in &self.abiertas {
                    self.cartas[i].cara = false;
                }
                self.abiertas.clear();
            }
        }
        if self.celebrando > 0.0 {
            self.celebrando -= dt;
            if self.celebrando <= 0.0 {
                self.repartir(cx);
            }
        }
        self.confeti.actualizar(dt);
    }

    fn dibujar(&mut self, l: &mut Lienzo, cx: &Cx) {
        l.degradado_v(0.0, 0.0, AW, AH, rgb(225, 247, 235), rgb(200, 235, 215));
        for c in &self.cartas {
            let sx = (c.giro * PI).cos().abs();
            let frente = c.giro > 0.5;
            let sobre = c.r.dentro(cx.rl) && !frente;
            let r0 = if sobre { c.r.crecer(4.0) } else { c.r };
            let w = (r0.w * sx).max(2.0);
            let r = R::new(r0.cx() - w / 2.0, r0.y, w, r0.h);
            l.rrect_r(r.mover(0.0, 6.0), 18.0, alfa(TINTA, 0.25));
            if frente {
                l.rrect_r(r, 18.0, BLANCO);
                if c.hecha {
                    l.rrect_borde(r, 18.0, 6.0, VERDE);
                }
                if sx > 0.15 {
                    dibujos::dibujar(l, c.obj, r.cx(), r.cy(), r0.w.min(r0.h) * 0.62 * sx);
                }
            } else {
                l.rrect_r(r, 18.0, MORADO);
                if sx > 0.3 {
                    let ri = R::new(r.x + 10.0 * sx, r.y + 10.0, r.w - 20.0 * sx, r.h - 20.0);
                    l.rrect_r(ri, 12.0, rgb(180, 130, 240));
                    l.estrella(r.cx(), r.cy(), r0.w.min(r0.h) * 0.22 * sx, -PI / 2.0, AMARILLO);
                }
            }
        }
        if self.celebrando > 0.0 {
            felicitar(l, "¡Ganaste!", AW / 2.0, AH / 2.0, cx.t);
        }
        self.confeti.dibujar(l);
    }
}
