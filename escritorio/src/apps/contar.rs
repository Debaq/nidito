//! Contar: "¿Cuántas manzanas hay?" y "Toca el número 7". Números del 1 al 10.

use super::*;
use crate::cx::Son;
use crate::dibujos::{self, Obj, CONTABLES};
use crate::efectos::Confeti;

enum Modo {
    Cuantos { obj: Obj, pos: Vec<(f32, f32)>, contados: Vec<usize> },
    Toca { burbujas: Vec<(usize, f32, f32)> },
}

pub struct Contar {
    modo: Modo,
    n: usize,
    max: usize,
    racha: usize,
    estrellas: usize,
    ronda: usize,
    celebrando: f32,
    sacudir: Option<(usize, f32)>,
    confeti: Confeti,
}

const S: f32 = 92.0;
const BY: f32 = 412.0;
const BW: f32 = 84.0;
const BG: f32 = 10.0;

fn boton(i: usize, max: usize) -> R {
    let total = max as f32 * BW + (max - 1) as f32 * BG;
    let x0 = AW / 2.0 - total / 2.0;
    R::new(x0 + (i - 1) as f32 * (BW + BG), BY, BW, 86.0)
}

impl Contar {
    pub fn nuevo(cx: &mut Cx) -> Box<dyn App> {
        let mut c = Contar {
            modo: Modo::Toca { burbujas: vec![] },
            n: 1,
            max: 5,
            racha: 0,
            estrellas: 0,
            ronda: 0,
            celebrando: 0.0,
            sacudir: None,
            confeti: Confeti::default(),
        };
        c.nueva_ronda(cx);
        Box::new(c)
    }

    fn nueva_ronda(&mut self, cx: &mut Cx) {
        self.ronda += 1;
        self.sacudir = None;
        let mut n = 1 + cx.rng.n(self.max);
        if n == self.n {
            n = 1 + cx.rng.n(self.max);
        }
        self.n = n;
        if self.ronda % 3 == 0 {
            let mut nums: Vec<usize> = (1..=self.max).collect();
            cx.rng.barajar(&mut nums);
            let cols = 5usize;
            let filas = self.max.div_ceil(cols);
            let mut burbujas = Vec::new();
            for (k, v) in nums.into_iter().enumerate() {
                let (col, fila) = (k % cols, k / cols);
                let en_fila = if fila + 1 == filas { self.max - fila * cols } else { cols };
                let x0 = AW / 2.0 - (en_fila as f32 - 1.0) * 95.0;
                let x = x0 + col as f32 * 190.0 + cx.rng.rango(-18.0, 18.0);
                let yc = if filas == 1 { 290.0 } else { 195.0 + fila as f32 * 185.0 };
                burbujas.push((v, x, yc + cx.rng.rango(-14.0, 14.0)));
            }
            self.modo = Modo::Toca { burbujas };
        } else {
            let obj = *cx.rng.elegir(&CONTABLES);
            let mut pos = Vec::new();
            let filas: Vec<usize> = if n <= 5 { vec![n] } else { vec![5, n - 5] };
            let ys: Vec<f32> = if filas.len() == 1 { vec![240.0] } else { vec![170.0, 305.0] };
            for (f, &cant) in filas.iter().enumerate() {
                let x0 = AW / 2.0 - (cant as f32 - 1.0) * 62.5;
                for i in 0..cant {
                    pos.push((x0 + i as f32 * 125.0, ys[f]));
                }
            }
            self.modo = Modo::Cuantos { obj, contados: vec![0; pos.len()], pos };
        }
        self.preguntar(cx);
    }

    fn pregunta(&self) -> String {
        match &self.modo {
            Modo::Cuantos { obj, .. } => {
                format!("¿{} {} hay?", if obj.fem() { "Cuántas" } else { "Cuántos" }, obj.plural())
            }
            Modo::Toca { .. } => format!("Toca el número {}", self.n),
        }
    }

    fn preguntar(&self, cx: &mut Cx) {
        match &self.modo {
            Modo::Cuantos { obj, .. } => {
                let (es, en) = frase_cuantos(*obj);
                cx.decir_bil(&es, &en);
            }
            Modo::Toca { .. } => {
                let (es, en) = frase_toca(self.n);
                cx.decir_bil(&es, &en);
            }
        }
    }

    fn responder(&mut self, k: usize, cx: &mut Cx) {
        if self.celebrando > 0.0 {
            return;
        }
        if k == self.n {
            self.estrellas += 1;
            self.racha += 1;
            if self.racha >= 3 && self.max < 10 {
                self.max += 1;
                self.racha = 0;
            }
            self.celebrando = 2.6;
            cx.sonar(Son::Fanfarria);
            let (x, y) = match &self.modo {
                Modo::Cuantos { .. } => {
                    let r = boton(k, self.max.max(k));
                    (r.cx(), r.y)
                }
                Modo::Toca { burbujas } => burbujas.iter().find(|b| b.0 == k).map(|b| (b.1, b.2)).unwrap_or((500.0, 260.0)),
            };
            self.confeti.lanzar(x, y, 70, &mut cx.rng);
            let (a_es, a_en) = alabanza(cx);
            let (es, en) = match &self.modo {
                Modo::Cuantos { obj, .. } => frase_hay(*obj, k),
                Modo::Toca { .. } => frase_es_el(k),
            };
            cx.decir_v(&[a_es, es.as_str()], &[a_en, en.as_str()]);
        } else {
            self.racha = 0;
            self.sacudir = Some((k, 0.6));
            cx.sonar(Son::Boop);
            let (e1, n1) = frase_ese_es(k);
            match &mut self.modo {
                Modo::Cuantos { contados, obj, .. } => {
                    contados.iter_mut().for_each(|c| *c = 0);
                    let (e2, n2) = frase_cuenta_otra(*obj);
                    cx.decir_v(&[e1, e2], &[n1, n2]);
                }
                Modo::Toca { .. } => {
                    let (e2, n2) = frase_busca(self.n);
                    cx.decir_v(&[e1, e2], &[n1, n2]);
                }
            }
        }
    }
}

fn frase_cuantos(o: Obj) -> (String, String) {
    (
        format!("¿{} {} hay?", if o.fem() { "Cuántas" } else { "Cuántos" }, o.plural()),
        format!("How many {}?", o.ingles_plural()),
    )
}
fn frase_toca(n: usize) -> (String, String) {
    (format!("Toca el número {}.", num(n)), format!("Touch number {}.", num_en(n)))
}
fn frase_hay(o: Obj, n: usize) -> (String, String) {
    (format!("Hay {}.", o.con_cantidad(n)), format!("{}!", o.en_cantidad(n)))
}
fn frase_es_el(n: usize) -> (String, String) {
    (format!("Es el {}.", num(n)), format!("Number {}!", num_en(n)))
}
fn frase_ese_es(n: usize) -> (String, String) {
    (format!("Ese es el {}.", num(n)), format!("That is {}.", num_en(n)))
}
fn frase_cuenta_otra(o: Obj) -> (String, String) {
    (format!("¡Cuénta{} otra vez!", if o.fem() { "las" } else { "los" }), "Count again!".to_string())
}
fn frase_busca(n: usize) -> (String, String) {
    (format!("Busca el {}.", num(n)), format!("Find {}.", num_en(n)))
}

pub fn frases(f: &mut Frases) {
    for o in CONTABLES {
        let (a, b) = frase_cuantos(o);
        f.bil(a, b);
        let (a, b) = frase_cuenta_otra(o);
        f.bil(a, b);
        for n in 1..=10 {
            let (a, b) = frase_hay(o, n);
            f.bil(a, b);
        }
    }
    for n in 1..=10 {
        for g in [frase_toca, frase_es_el, frase_ese_es, frase_busca] {
            let (a, b) = g(n);
            f.bil(a, b);
        }
    }
}

impl App for Contar {
    fn animando(&self) -> bool {
        self.celebrando > 0.0 || self.sacudir.is_some() || self.confeti.activo()
    }

    fn evento(&mut self, ev: &Ev, cx: &mut Cx) {
        match ev {
            Ev::Clic(x, y) => {
                let p = (*x, *y);
                if dist(p, (950.0, 42.0)) < 32.0 {
                    self.preguntar(cx);
                    return;
                }
                if self.celebrando > 0.0 {
                    return;
                }
                let max = self.max;
                let mut resp = None;
                match &mut self.modo {
                    Modo::Cuantos { pos, contados, .. } => {
                        for (j, q) in pos.iter().enumerate() {
                            if dist(p, *q) < S * 0.55 {
                                if contados[j] == 0 {
                                    let k = contados.iter().max().copied().unwrap_or(0) + 1;
                                    contados[j] = k;
                                }
                                cx.sonar(Son::Clic);
                                cx.decir_num(contados[j]);
                                return;
                            }
                        }
                        for k in 1..=max {
                            if boton(k, max).dentro(p) {
                                resp = Some(k);
                            }
                        }
                    }
                    Modo::Toca { burbujas } => {
                        for b in burbujas.iter() {
                            if dist(p, (b.1, b.2)) < 62.0 {
                                resp = Some(b.0);
                            }
                        }
                    }
                }
                if let Some(k) = resp {
                    self.responder(k, cx);
                }
            }
            Ev::Texto(t) => {
                if let Some(d) = t.chars().next().and_then(|c| c.to_digit(10)) {
                    let k = if d == 0 { 10 } else { d as usize };
                    if k <= self.max {
                        self.responder(k, cx);
                    }
                }
            }
            _ => {}
        }
    }

    fn actualizar(&mut self, dt: f32, cx: &mut Cx) {
        self.confeti.actualizar(dt);
        if let Some((_, t)) = &mut self.sacudir {
            *t -= dt;
            if *t <= 0.0 {
                self.sacudir = None;
            }
        }
        if self.celebrando > 0.0 {
            self.celebrando -= dt;
            if self.celebrando <= 0.0 {
                self.nueva_ronda(cx);
            }
        }
    }

    fn dibujar(&mut self, l: &mut Lienzo, cx: &Cx) {
        l.degradado_v(0.0, 0.0, AW, AH, rgb(255, 248, 220), rgb(255, 232, 190));
        // marcador de estrellas
        l.estrella(46.0, 42.0, 26.0, -std::f32::consts::FRAC_PI_2, AMARILLO);
        l.texto(&format!("{}", self.estrellas), 80.0, 18.0, 40.0, rgb(200, 120, 0));
        let preg = self.pregunta();
        l.texto_c(&preg, AW / 2.0, 42.0, 42.0, TINTA);
        boton_altavoz(l, 950.0, 42.0, 30.0, NARANJO, cx);

        let sacude = |k: usize| -> f32 {
            match self.sacudir {
                Some((s, t)) if s == k => (t * 40.0).sin() * 10.0 * t,
                _ => 0.0,
            }
        };

        match &self.modo {
            Modo::Cuantos { obj, pos, contados } => {
                for (j, &(x, y)) in pos.iter().enumerate() {
                    let sobre = dist(cx.rl, (x, y)) < S * 0.55;
                    let k = contados[j];
                    if k > 0 {
                        l.circulo(x, y, S * 0.62, rgb(200, 240, 200));
                    } else if sobre {
                        l.circulo(x, y, S * 0.62, rgb(255, 255, 255));
                    }
                    let s = if sobre { S * 1.08 } else { S };
                    dibujos::dibujar(l, *obj, x, y, s);
                    if k > 0 {
                        let (bx, by) = (x + S * 0.42, y - S * 0.42);
                        l.circulo(bx, by, 22.0, VERDE);
                        l.texto_c(&k.to_string(), bx, by, 30.0, BLANCO);
                    }
                }
                for k in 1..=self.max {
                    let r = boton(k, self.max).mover(sacude(k), 0.0);
                    let c = ARCOIRIS[(k - 1) % ARCOIRIS.len()];
                    let sobre = r.dentro(cx.rl);
                    let r = if sobre { r.crecer(4.0) } else { r };
                    l.rrect_r(r.mover(0.0, 5.0), 18.0, oscurecer(c, 0.3));
                    l.rrect_r(r, 18.0, c);
                    l.texto_c_borde(&k.to_string(), r.cx(), r.cy(), 54.0, BLANCO, oscurecer(c, 0.4), 2.0);
                }
            }
            Modo::Toca { burbujas } => {
                for &(v, x, y) in burbujas {
                    let x = x + sacude(v);
                    let c = ARCOIRIS[(v - 1) % ARCOIRIS.len()];
                    let sobre = dist(cx.rl, (x, y)) < 62.0;
                    let r = if sobre { 66.0 } else { 60.0 };
                    l.circulo(x, y + 5.0, r, oscurecer(c, 0.3));
                    l.circulo(x, y, r, c);
                    l.circulo(x - r * 0.35, y - r * 0.35, r * 0.18, alfa(BLANCO, 0.5));
                    l.texto_c_borde(&v.to_string(), x, y, 70.0, BLANCO, oscurecer(c, 0.4), 3.0);
                }
            }
        }
        if self.celebrando > 0.0 {
            let alab = "¡Muy bien!";
            felicitar(l, alab, AW / 2.0, 360.0, cx.t);
        }
        self.confeti.dibujar(l);
    }
}
