//! English: tarjetas con dibujos (español + inglés) y juego "Where is the cat?".

use super::*;
use crate::cx::Son;
use crate::dibujos::{self, Obj, VOCABULARIO};
use crate::efectos::Confeti;

#[derive(PartialEq, Clone, Copy)]
enum Modo {
    Mirar,
    Jugar,
}

pub struct English {
    modo: Modo,
    pagina: usize,
    sel: Option<(usize, f32)>,
    opciones: Vec<Obj>,
    meta: Obj,
    estrellas: usize,
    celebrando: f32,
    sacudir: Option<(usize, f32)>,
    confeti: Confeti,
}

const B_MIRAR: R = R::new(20.0, 14.0, 190.0, 54.0);
const B_JUGAR: R = R::new(222.0, 14.0, 190.0, 54.0);
const POR_PAG: usize = 6;

fn tarjeta(i: usize) -> R {
    let (c, f) = (i % 3, i / 3);
    R::new(150.0 + c as f32 * 240.0, 88.0 + f as f32 * 212.0, 220.0, 196.0)
}
fn opcion(i: usize) -> R {
    R::new(80.0 + i as f32 * 290.0, 150.0, 260.0, 300.0)
}

fn donde(o: Obj) -> (String, String) {
    (format!("¿Dónde está {}?", o.con_articulo()), format!("Where is the {}?", o.ingles()))
}
fn si_es(o: Obj) -> String {
    format!("Yes! The {}!", o.ingles())
}
fn ese_es(o: Obj) -> String {
    format!("That is the {}.", o.ingles())
}

pub fn frases(f: &mut Frases) {
    f.bil("Mirar", "Look");
    f.bil("Jugar", "Play");
    for o in VOCABULARIO {
        let (a, b) = donde(o);
        f.bil(a, b);
        f.en(si_es(o));
        f.en(ese_es(o));
    }
}

impl English {
    pub fn nuevo(cx: &mut Cx) -> Box<dyn App> {
        let mut e = English {
            modo: Modo::Mirar,
            pagina: 0,
            sel: None,
            opciones: vec![],
            meta: Obj::Gato,
            estrellas: 0,
            celebrando: 0.0,
            sacudir: None,
            confeti: Confeti::default(),
        };
        e.nueva_pregunta(cx, false);
        cx.decir_es_en("Mirar", "Look");
        Box::new(e)
    }

    fn paginas() -> usize {
        VOCABULARIO.len().div_ceil(POR_PAG)
    }

    fn nueva_pregunta(&mut self, cx: &mut Cx, hablar: bool) {
        let mut v = VOCABULARIO.to_vec();
        cx.rng.barajar(&mut v);
        if v[0] == self.meta {
            v.swap(0, 3);
        }
        self.opciones = v[..3].to_vec();
        self.meta = self.opciones[cx.rng.n(3)];
        if hablar {
            self.preguntar(cx);
        }
    }

    fn preguntar(&self, cx: &mut Cx) {
        let (es, en) = donde(self.meta);
        if cx.idioma == crate::cx::Idioma::Es {
            cx.decir_es_en(&es, &en);
        } else {
            cx.decir_en(&en);
        }
    }
}

impl App for English {
    fn animando(&self) -> bool {
        self.celebrando > 0.0 || self.sacudir.is_some() || self.confeti.activo() || self.sel.is_some_and(|s| s.1 < 0.4)
    }

    fn evento(&mut self, ev: &Ev, cx: &mut Cx) {
        let Ev::Clic(x, y) = ev else { return };
        let p = (*x, *y);
        if B_MIRAR.dentro(p) && self.modo != Modo::Mirar {
            self.modo = Modo::Mirar;
            cx.sonar(Son::Clic);
            cx.decir_es_en("Mirar", "Look");
            return;
        }
        if B_JUGAR.dentro(p) && self.modo != Modo::Jugar {
            self.modo = Modo::Jugar;
            cx.sonar(Son::Clic);
            self.nueva_pregunta(cx, true);
            return;
        }
        if dist(p, (940.0, 41.0)) < 30.0 {
            match self.modo {
                Modo::Jugar => self.preguntar(cx),
                Modo::Mirar => {
                    if let Some((i, _)) = self.sel {
                        let o = VOCABULARIO[i];
                        cx.decir_es_en(o.singular(), o.ingles());
                    }
                }
            }
            return;
        }
        match self.modo {
            Modo::Mirar => {
                let n = English::paginas();
                if dist(p, (60.0, 300.0)) < 36.0 {
                    self.pagina = (self.pagina + n - 1) % n;
                    cx.sonar(Son::Whoosh);
                    return;
                }
                if dist(p, (940.0, 300.0)) < 36.0 {
                    self.pagina = (self.pagina + 1) % n;
                    cx.sonar(Son::Whoosh);
                    return;
                }
                for k in 0..POR_PAG {
                    let i = self.pagina * POR_PAG + k;
                    if i < VOCABULARIO.len() && tarjeta(k).dentro(p) {
                        self.sel = Some((i, 0.0));
                        let o = VOCABULARIO[i];
                        cx.sonar(Son::Clic);
                        cx.decir_es_en(o.singular(), o.ingles());
                    }
                }
            }
            Modo::Jugar => {
                if self.celebrando > 0.0 {
                    return;
                }
                for i in 0..3 {
                    if opcion(i).dentro(p) {
                        let o = self.opciones[i];
                        if o == self.meta {
                            self.estrellas += 1;
                            self.celebrando = 2.4;
                            cx.sonar(Son::Fanfarria);
                            let r = opcion(i);
                            self.confeti.lanzar(r.cx(), r.cy(), 70, &mut cx.rng);
                            cx.decir_en(&si_es(o));
                        } else {
                            cx.sonar(Son::Boop);
                            self.sacudir = Some((i, 0.6));
                            cx.decir_en_v(&[ese_es(o), donde(self.meta).1]);
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
        l.degradado_v(0.0, 0.0, AW, AH, rgb(235, 245, 255), rgb(215, 230, 255));
        l.rect(0.0, 0.0, AW, 82.0, rgb(222, 234, 255));
        boton_texto(l, B_MIRAR, "Mirar · Look", AZUL, self.modo == Modo::Mirar, cx);
        boton_texto(l, B_JUGAR, "Jugar · Play", ROJO, self.modo == Modo::Jugar, cx);
        boton_altavoz(l, 940.0, 41.0, 28.0, AZUL, cx);
        match self.modo {
            Modo::Mirar => {
                for k in 0..POR_PAG {
                    let i = self.pagina * POR_PAG + k;
                    if i >= VOCABULARIO.len() {
                        continue;
                    }
                    let o = VOCABULARIO[i];
                    let mut r = tarjeta(k);
                    let sel = self.sel.map(|s| s.0) == Some(i);
                    if sel {
                        let t = self.sel.unwrap().1;
                        r = r.crecer(6.0 * ease_out_back((t / 0.3).min(1.0)));
                    } else if r.dentro(cx.rl) {
                        r = r.crecer(3.0);
                    }
                    l.rrect_r(r.mover(0.0, 6.0), 20.0, alfa(TINTA, 0.18));
                    l.rrect_r(r, 20.0, BLANCO);
                    if sel {
                        l.rrect_borde(r, 20.0, 5.0, NARANJO);
                    }
                    dibujos::dibujar(l, o, r.cx(), r.y + 76.0, 110.0);
                    l.texto_c(o.ingles(), r.cx(), r.y + 150.0, 34.0, AZUL);
                    l.texto_c(o.singular(), r.cx(), r.y + 180.0, 22.0, GRIS);
                }
                boton_flecha(l, 60.0, 300.0, 34.0, -1.0, AZUL, cx);
                boton_flecha(l, 940.0, 300.0, 34.0, 1.0, AZUL, cx);
                let n = English::paginas();
                for i in 0..n {
                    l.circulo(AW / 2.0 - (n as f32 - 1.0) * 14.0 + i as f32 * 28.0, AH - 14.0, 7.0, if i == self.pagina { AZUL } else { rgb(190, 205, 230) });
                }
            }
            Modo::Jugar => {
                l.estrella(470.0, 42.0, 24.0, -std::f32::consts::FRAC_PI_2, AMARILLO);
                l.texto(&self.estrellas.to_string(), 500.0, 18.0, 40.0, rgb(200, 120, 0));
                let (_, en) = donde(self.meta);
                l.texto_c(&en, AW / 2.0, 115.0, 44.0, TINTA);
                for i in 0..3 {
                    let mut r = opcion(i);
                    if let Some((k, t)) = self.sacudir {
                        if k == i {
                            r = r.mover((t * 40.0).sin() * 10.0 * t, 0.0);
                        }
                    }
                    let gana = self.celebrando > 0.0 && self.opciones[i] == self.meta;
                    if r.dentro(cx.rl) || gana {
                        r = r.crecer(5.0);
                    }
                    l.rrect_r(r.mover(0.0, 6.0), 24.0, alfa(TINTA, 0.18));
                    l.rrect_r(r, 24.0, BLANCO);
                    if gana {
                        l.rrect_borde(r, 24.0, 6.0, VERDE);
                    }
                    dibujos::dibujar(l, self.opciones[i], r.cx(), r.cy(), 180.0);
                }
                if self.celebrando > 0.0 {
                    felicitar(l, "Yes!", AW / 2.0, 470.0, cx.t);
                }
            }
        }
        self.confeti.dibujar(l);
    }
}
