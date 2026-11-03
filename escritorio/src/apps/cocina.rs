//! Cocina: recetas paso a paso. Contar ingredientes, revolver, horno con cuenta regresiva y decorar.

use super::*;
use crate::cx::Son;
use crate::dibujos::{self, Obj};
use crate::efectos::Confeti;

#[derive(Clone, Copy, PartialEq)]
enum Paso {
    Poner(Obj, usize),
    Revolver,
    Horno,
    Sarten,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Plato {
    Panqueques,
    Galletas,
    Ensalada,
    Pizza,
    Queque,
}

struct Receta {
    es: &'static str,
    en: &'static str,
    plato: Plato,
    pasos: &'static [Paso],
    deco: [Obj; 3],
    bandeja: bool,
    /// (decora es, decora en, listo es, listo en)
    textos: [&'static str; 4],
}

use Obj::*;
use Paso::*;

const RECETAS: [Receta; 5] = [
    Receta {
        es: "Panqueques",
        en: "Pancakes",
        plato: Plato::Panqueques,
        pasos: &[Poner(Harina, 2), Poner(Huevo, 2), Poner(Leche, 1), Revolver, Sarten],
        deco: [Frutilla, Platano, Chocolate],
        bandeja: false,
        textos: ["Decora tus panqueques.", "Decorate your pancakes.", "Tus panqueques están listos.", "Your pancakes are ready."],
    },
    Receta {
        es: "Galletas",
        en: "Cookies",
        plato: Plato::Galletas,
        pasos: &[Poner(Harina, 3), Poner(Mantequilla, 1), Poner(Azucar, 2), Revolver, Horno],
        deco: [Chocolate, Frutilla, Uva],
        bandeja: false,
        textos: ["Decora tus galletas.", "Decorate your cookies.", "Tus galletas están listas.", "Your cookies are ready."],
    },
    Receta {
        es: "Ensalada de frutas",
        en: "Fruit salad",
        plato: Plato::Ensalada,
        pasos: &[Poner(Platano, 2), Poner(Frutilla, 4), Poner(Uva, 3), Poner(Naranja, 1), Revolver],
        deco: [Frutilla, Uva, Manzana],
        bandeja: false,
        textos: ["Decora tu ensalada.", "Decorate your salad.", "Tu ensalada está lista.", "Your salad is ready."],
    },
    Receta {
        es: "Pizza",
        en: "Pizza",
        plato: Plato::Pizza,
        pasos: &[Poner(Tomate, 3), Poner(Queso, 4), Horno],
        deco: [Tomate, Queso, Huevo],
        bandeja: true,
        textos: ["Decora tu pizza.", "Decorate your pizza.", "Tu pizza está lista.", "Your pizza is ready."],
    },
    Receta {
        es: "Queque",
        en: "Cake",
        plato: Plato::Queque,
        pasos: &[Poner(Huevo, 3), Poner(Harina, 2), Poner(Azucar, 2), Poner(Leche, 1), Revolver, Horno],
        deco: [Frutilla, Chocolate, Estrella],
        bandeja: false,
        textos: ["Decora tu queque.", "Decorate your cake.", "Tu queque está listo.", "Your cake is ready."],
    },
];

const INGREDIENTES: [Obj; 12] = [Huevo, Harina, Leche, Azucar, Mantequilla, Platano, Frutilla, Uva, Naranja, Tomate, Queso, Chocolate];

fn articulo_en(s: &str) -> &'static str {
    if s.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    }
}

fn frase_pon(o: Obj, n: usize) -> (String, String) {
    (format!("Pon {}.", o.con_cantidad(n)), format!("Put {}.", o.en_cantidad(n)))
}
fn frase_eso_es(o: Obj) -> (String, String) {
    (format!("Eso es {}.", o.con_cantidad(1)), format!("That is {} {}.", articulo_en(o.ingles_unidad()), o.ingles_unidad()))
}
fn frase_necesitamos(o: Obj) -> (String, String) {
    (format!("Necesitamos {}.", o.plural()), format!("We need {}.", o.ingles_plural()))
}

pub fn frases(f: &mut Frases) {
    f.bil("¿Qué cocinamos hoy?", "What shall we cook today?");
    f.bil("¡Revuelve, revuelve!", "Stir, stir!");
    f.bil("Al horno. Toca el botón.", "Into the oven. Touch the button.");
    f.bil("A la sartén. Toca el botón.", "Into the pan. Touch the button.");
    f.bil("¡Listo!", "Ready!");
    f.bil("¡Qué rico!", "Yummy!");
    for r in &RECETAS {
        f.bil(r.es, r.en);
        f.bil(r.textos[0], r.textos[1]);
        f.bil(r.textos[2], r.textos[3]);
        for p in r.pasos {
            if let Poner(o, n) = p {
                let (a, b) = frase_pon(*o, *n);
                f.bil(a, b);
            }
        }
    }
    for o in INGREDIENTES {
        let (a, b) = frase_eso_es(o);
        f.bil(a, b);
        let (a, b) = frase_necesitamos(o);
        f.bil(a, b);
    }
}

enum Fase {
    Elegir,
    Pasos,
    Decorar,
    Final,
}

struct Vuelo {
    obj: Obj,
    desde: (f32, f32),
    hacia: (f32, f32),
    t: f32,
}

pub struct Cocina {
    fase: Fase,
    receta: usize,
    paso: usize,
    puestos: usize,
    estante: Vec<Obj>,
    contenido: Vec<(Obj, f32, f32)>,
    vuelos: Vec<Vuelo>,
    mezclado: bool,
    revuelto: f32,
    giro: f32,
    cuenta: Option<f32>,
    listo_t: f32,
    deco: Vec<(Obj, f32, f32)>,
    deco_sel: usize,
    sacudir: Option<(usize, f32)>,
    confeti: Confeti,
}

const BOL: (f32, f32) = (600.0, 300.0);
const PLATO: (f32, f32) = (430.0, 290.0);
const VOLVER: (f32, f32) = (40.0, 40.0);

fn estante_r(i: usize) -> R {
    R::new(24.0, 96.0 + i as f32 * 140.0, 170.0, 126.0)
}
fn tarjeta_r(i: usize) -> R {
    R::new(22.0 + i as f32 * 194.0, 120.0, 180.0, 290.0)
}
fn deco_r(i: usize) -> R {
    R::new(830.0, 110.0 + i as f32 * 120.0, 140.0, 104.0)
}
const B_LISTO: R = R::new(800.0, 440.0, 180.0, 60.0);
const B_HORNO: R = R::new(520.0, 420.0, 200.0, 70.0);

impl Cocina {
    pub fn nuevo(cx: &mut Cx) -> Box<dyn App> {
        cx.decir_bil("¿Qué cocinamos hoy?", "What shall we cook today?");
        Box::new(Cocina {
            fase: Fase::Elegir,
            receta: 0,
            paso: 0,
            puestos: 0,
            estante: vec![],
            contenido: vec![],
            vuelos: vec![],
            mezclado: false,
            revuelto: 0.0,
            giro: 0.0,
            cuenta: None,
            listo_t: 0.0,
            deco: vec![],
            deco_sel: 0,
            sacudir: None,
            confeti: Confeti::default(),
        })
    }

    fn rec(&self) -> &'static Receta {
        &RECETAS[self.receta]
    }

    fn paso_actual(&self) -> Option<Paso> {
        self.rec().pasos.get(self.paso).copied()
    }

    fn empezar_paso(&mut self, cx: &mut Cx) {
        self.puestos = 0;
        self.revuelto = 0.0;
        self.cuenta = None;
        self.listo_t = 0.0;
        match self.paso_actual() {
            Some(Poner(o, n)) => {
                let mut otros: Vec<Obj> = INGREDIENTES.iter().copied().filter(|x| *x != o).collect();
                cx.rng.barajar(&mut otros);
                let mut est = vec![o, otros[0], otros[1]];
                cx.rng.barajar(&mut est);
                self.estante = est;
                let (a, b) = frase_pon(o, n);
                cx.decir_bil(&a, &b);
            }
            Some(Revolver) => cx.decir_bil("¡Revuelve, revuelve!", "Stir, stir!"),
            Some(Horno) => cx.decir_bil("Al horno. Toca el botón.", "Into the oven. Touch the button."),
            Some(Sarten) => cx.decir_bil("A la sartén. Toca el botón.", "Into the pan. Touch the button."),
            None => {
                self.fase = Fase::Decorar;
                self.deco.clear();
                let r = self.rec();
                cx.decir_bil(r.textos[0], r.textos[1]);
            }
        }
    }

    fn paso_hecho(&mut self, cx: &mut Cx) {
        cx.sonar(Son::Ding);
        self.listo_t = 1.4;
    }

    fn dibujar_bol(&self, l: &mut Lienzo, cx: &Cx) {
        let (bx, by) = BOL;
        if self.rec().bandeja {
            l.ovalo(bx, by + 40.0, 230.0, 90.0, rgb(170, 175, 185));
            l.ovalo(bx, by + 32.0, 215.0, 80.0, rgb(200, 205, 215));
            l.ovalo(bx, by + 30.0, 190.0, 68.0, rgb(235, 200, 140));
            for (o, dx, dy) in &self.contenido {
                dibujos::dibujar(l, *o, bx + dx, by + 30.0 + dy * 0.9, 56.0);
            }
            return;
        }
        let azul = rgb(110, 180, 235);
        l.ovalo(bx, by, 200.0, 56.0, oscurecer(azul, 0.35));
        if self.mezclado {
            let c = match self.rec().plato {
                Plato::Ensalada => rgb(250, 200, 120),
                Plato::Galletas => rgb(225, 180, 120),
                _ => rgb(250, 225, 160),
            };
            l.ovalo(bx, by + 4.0, 185.0, 46.0, c);
            for k in 0..3 {
                let a = self.giro + k as f32 * 2.1;
                l.ovalo(bx + a.cos() * 90.0, by + 4.0 + a.sin() * 22.0, 30.0, 8.0, aclarar(c, 0.35));
            }
        } else {
            for (o, dx, dy) in &self.contenido {
                let a = self.giro;
                let (rx, ry) = (dx * a.cos() - dy * 3.0 * a.sin(), (dx * a.sin() / 3.0 + dy * a.cos()));
                dibujos::dibujar(l, *o, bx + rx, by + ry - 8.0, 52.0);
            }
        }
        // frente del bol
        let mut pts = vec![];
        for i in 0..=24 {
            let a = std::f32::consts::PI * i as f32 / 24.0;
            pts.push((bx + a.cos() * 205.0, by + a.sin() * 150.0));
        }
        l.poligono(&pts, azul);
        l.ovalo(bx - 90.0, by + 60.0, 22.0, 40.0, alfa(BLANCO, 0.35));
        let mut borde = vec![];
        for i in 0..=24 {
            let a = std::f32::consts::PI * i as f32 / 24.0;
            borde.push((bx + a.cos() * 203.0, by + a.sin() * 57.0));
        }
        l.polilinea(&borde, 8.0, aclarar(azul, 0.4));
        if matches!(self.paso_actual(), Some(Revolver)) && !self.mezclado {
            // barra de avance
            let p = (self.revuelto / 2600.0).min(1.0);
            l.rrect(bx - 150.0, by + 170.0, 300.0, 22.0, 11.0, rgb(230, 230, 240));
            l.rrect(bx - 150.0, by + 170.0, 300.0 * p, 22.0, 11.0, VERDE);
            let _ = cx;
        }
    }

    fn dibujar_horno(&self, l: &mut Lienzo, cx: &Cx, sarten: bool) {
        let (hx, hy) = (620.0, 250.0);
        if sarten {
            l.rrect(hx - 190.0, hy + 60.0, 380.0, 120.0, 16.0, rgb(230, 230, 236));
            let fuego = self.cuenta.is_some();
            for k in 0..5 {
                let x = hx - 100.0 + k as f32 * 50.0;
                if fuego {
                    let h = 26.0 + (cx.t * 12.0 + k as f32).sin() * 6.0;
                    l.poligono(&[(x - 12.0, hy + 62.0), (x, hy + 62.0 - h), (x + 12.0, hy + 62.0)], NARANJO);
                }
            }
            l.linea(hx + 150.0, hy + 10.0, hx + 290.0, hy - 20.0, 22.0, TINTA);
            l.ovalo(hx, hy + 20.0, 170.0, 50.0, rgb(60, 60, 70));
            l.ovalo(hx, hy + 12.0, 150.0, 38.0, rgb(90, 90, 100));
            let c = if fuego { mezclar(rgb(250, 225, 160), rgb(220, 160, 70), 1.0 - self.cuenta.unwrap_or(0.0) / 5.0) } else { rgb(250, 225, 160) };
            l.ovalo(hx, hy + 12.0, 105.0, 26.0, c);
        } else {
            l.rrect(hx - 190.0, hy - 150.0, 380.0, 300.0, 24.0, rgb(200, 205, 215));
            l.rrect(hx - 170.0, hy - 130.0, 340.0, 40.0, 10.0, rgb(160, 165, 180));
            for k in 0..3 {
                l.circulo(hx - 120.0 + k as f32 * 50.0, hy - 110.0, 12.0, TINTA);
            }
            let encendido = self.cuenta.is_some();
            let vidrio = if encendido { mezclar(rgb(255, 170, 60), rgb(255, 120, 30), 0.5 + 0.5 * (cx.t * 4.0).sin()) } else { rgb(60, 60, 75) };
            l.rrect(hx - 150.0, hy - 70.0, 300.0, 190.0, 16.0, vidrio);
            l.rrect(hx - 110.0, hy - 90.0, 220.0, 12.0, 6.0, rgb(130, 135, 150));
        }
        if let Some(c) = self.cuenta {
            let n = c.ceil() as usize;
            if n > 0 {
                l.circulo(hx, hy - 10.0, 70.0, alfa(BLANCO, 0.9));
                l.texto_c(&n.to_string(), hx, hy - 10.0, 100.0, ROJO);
            }
        } else if self.listo_t <= 0.0 {
            let sobre = B_HORNO.dentro(cx.rl);
            let r = if sobre { B_HORNO.crecer(4.0) } else { B_HORNO };
            l.rrect_r(r.mover(0.0, 5.0), 35.0, oscurecer(ROJO, 0.3));
            l.rrect_r(r, 35.0, ROJO);
            l.poligono(&[(r.x + 40.0, r.cy() + 18.0), (r.x + 52.0, r.cy() - 22.0), (r.x + 64.0, r.cy() + 18.0)], AMARILLO);
            l.texto_c("Encender", r.cx() + 22.0, r.cy(), 32.0, BLANCO);
        }
    }
}

pub fn dibujar_plato(l: &mut Lienzo, p: Plato, cx: f32, cy: f32, s: f32) {
    let plato = |l: &mut Lienzo| {
        l.ovalo(cx, cy + 0.3 * s, 0.6 * s, 0.17 * s, rgb(200, 205, 215));
        l.ovalo(cx, cy + 0.28 * s, 0.52 * s, 0.13 * s, BLANCO);
    };
    match p {
        Plato::Panqueques => {
            plato(l);
            for k in 0..4 {
                let y = cy + 0.2 * s - k as f32 * 0.09 * s;
                l.ovalo(cx, y + 0.02 * s, 0.4 * s, 0.1 * s, rgb(200, 140, 60));
                l.ovalo(cx, y, 0.38 * s, 0.09 * s, rgb(240, 190, 100));
            }
            l.ovalo(cx, cy - 0.12 * s, 0.22 * s, 0.05 * s, rgb(150, 80, 30));
        }
        Plato::Galletas => {
            plato(l);
            dibujos::dibujar(l, Obj::Galleta, cx - 0.22 * s, cy + 0.12 * s, 0.4 * s);
            dibujos::dibujar(l, Obj::Galleta, cx + 0.22 * s, cy + 0.12 * s, 0.4 * s);
            dibujos::dibujar(l, Obj::Galleta, cx, cy - 0.02 * s, 0.4 * s);
        }
        Plato::Ensalada => {
            for (o, dx, dy) in [(Obj::Frutilla, -0.2, -0.08), (Obj::Uva, 0.0, -0.12), (Obj::Platano, 0.22, -0.05), (Obj::Naranja, 0.05, 0.0)] {
                dibujos::dibujar(l, o, cx + dx * s, cy + dy * s, 0.3 * s);
            }
            let mut pts = vec![];
            for i in 0..=20 {
                let a = std::f32::consts::PI * i as f32 / 20.0;
                pts.push((cx + a.cos() * 0.5 * s, cy + 0.05 * s + a.sin() * 0.35 * s));
            }
            l.poligono(&pts, rgb(255, 150, 120));
        }
        Plato::Pizza => {
            l.circulo(cx, cy, 0.5 * s, rgb(215, 160, 90));
            l.circulo(cx, cy, 0.43 * s, rgb(220, 60, 40));
            for (dx, dy) in [(-0.15, -0.1), (0.15, -0.15), (0.0, 0.15), (0.2, 0.12), (-0.2, 0.15), (0.0, -0.25)] {
                l.circulo(cx + dx * s, cy + dy * s, 0.1 * s, rgb(255, 220, 110));
            }
        }
        Plato::Queque => {
            plato(l);
            l.rrect(cx - 0.36 * s, cy - 0.18 * s, 0.72 * s, 0.42 * s, 0.06 * s, rgb(180, 110, 60));
            l.rect(cx - 0.36 * s, cy + 0.02 * s, 0.72 * s, 0.05 * s, rgb(255, 240, 220));
            l.rrect(cx - 0.38 * s, cy - 0.24 * s, 0.76 * s, 0.12 * s, 0.06 * s, rgb(255, 245, 235));
            for k in 0..6 {
                l.circulo(cx - 0.3 * s + k as f32 * 0.12 * s, cy - 0.12 * s, 0.05 * s, rgb(255, 245, 235));
            }
        }
    }
}

impl App for Cocina {
    fn animando(&self) -> bool {
        !self.vuelos.is_empty() || self.cuenta.is_some() || self.listo_t > 0.0 || self.sacudir.is_some() || self.confeti.activo()
    }

    fn evento(&mut self, ev: &Ev, cx: &mut Cx) {
        match ev {
            Ev::Clic(x, y) => {
                let p = (*x, *y);
                if !matches!(self.fase, Fase::Elegir) && dist(p, VOLVER) < 30.0 {
                    self.fase = Fase::Elegir;
                    cx.sonar(Son::Whoosh);
                    cx.decir_bil("¿Qué cocinamos hoy?", "What shall we cook today?");
                    return;
                }
                match self.fase {
                    Fase::Elegir => {
                        for i in 0..RECETAS.len() {
                            if tarjeta_r(i).dentro(p) {
                                self.receta = i;
                                self.paso = 0;
                                self.contenido.clear();
                                self.vuelos.clear();
                                self.mezclado = false;
                                self.fase = Fase::Pasos;
                                cx.sonar(Son::Whoosh);
                                self.empezar_paso(cx);
                                return;
                            }
                        }
                    }
                    Fase::Pasos => {
                        if self.listo_t > 0.0 {
                            return;
                        }
                        match self.paso_actual() {
                            Some(Poner(o, n)) => {
                                for (i, e) in self.estante.clone().iter().enumerate() {
                                    let r = estante_r(i);
                                    if !r.dentro(p) {
                                        continue;
                                    }
                                    if *e == o {
                                        if self.puestos + self.vuelos.len() >= n {
                                            return;
                                        }
                                        let destino = (cx.rng.rango(-120.0, 120.0), cx.rng.rango(-22.0, 18.0));
                                        self.vuelos.push(Vuelo { obj: o, desde: (r.cx(), r.cy()), hacia: destino, t: 0.0 });
                                        cx.sonar(Son::Whoosh);
                                    } else {
                                        cx.sonar(Son::Boop);
                                        self.sacudir = Some((i, 0.6));
                                        let (a1, b1) = frase_eso_es(*e);
                                        let (a2, b2) = frase_necesitamos(o);
                                        cx.decir_v(&[a1, a2], &[b1, b2]);
                                    }
                                    return;
                                }
                            }
                            Some(Horno) | Some(Sarten) => {
                                if self.cuenta.is_none() && B_HORNO.dentro(p) {
                                    self.cuenta = Some(5.0);
                                    cx.sonar(Son::Clic);
                                    cx.decir_num(5);
                                }
                            }
                            _ => {}
                        }
                    }
                    Fase::Decorar => {
                        for i in 0..3 {
                            if deco_r(i).dentro(p) {
                                self.deco_sel = i;
                                cx.sonar(Son::Clic);
                                cx.decir_obj(self.rec().deco[i]);
                                return;
                            }
                        }
                        if B_LISTO.dentro(p) {
                            self.fase = Fase::Final;
                            cx.sonar(Son::Fanfarria);
                            self.confeti.lanzar(PLATO.0, PLATO.1, 120, &mut cx.rng);
                            let r = self.rec();
                            cx.decir_v(&["¡Qué rico!", r.textos[2]], &["Yummy!", r.textos[3]]);
                            return;
                        }
                        if dist(p, PLATO) < 230.0 {
                            self.deco.push((self.rec().deco[self.deco_sel], p.0 - PLATO.0, p.1 - PLATO.1));
                            cx.sonar(Son::Sello);
                        }
                    }
                    Fase::Final => {
                        if B_LISTO.dentro(p) {
                            self.fase = Fase::Elegir;
                            cx.sonar(Son::Whoosh);
                            cx.decir_bil("¿Qué cocinamos hoy?", "What shall we cook today?");
                        }
                    }
                }
            }
            Ev::Mueve(x, y) => {
                if matches!(self.fase, Fase::Pasos) && matches!(self.paso_actual(), Some(Revolver)) && !self.mezclado {
                    let (dx, dy) = ((x - BOL.0) / 210.0, (y - BOL.1) / 120.0);
                    if dx * dx + dy * dy < 1.0 {
                        let d = dist(cx.rl, (*x, *y)).min(80.0);
                        self.revuelto += d;
                        self.giro += d * 0.01;
                        if self.revuelto >= 2600.0 {
                            self.mezclado = true;
                            self.paso_hecho(cx);
                            let (a, b) = alabanza(cx);
                            cx.decir_bil(a, b);
                        }
                    }
                }
            }
            Ev::Texto(t) => {
                // el número de la tecla también sirve para contar
                if let (Fase::Pasos, Some(Poner(o, n))) = (&self.fase, self.paso_actual()) {
                    if t.chars().next().and_then(|c| c.to_digit(10)).is_some() {
                        if let Some(i) = self.estante.iter().position(|e| *e == o) {
                            if self.puestos + self.vuelos.len() < n {
                                let r = estante_r(i);
                                let destino = (cx.rng.rango(-120.0, 120.0), cx.rng.rango(-22.0, 18.0));
                                self.vuelos.push(Vuelo { obj: o, desde: (r.cx(), r.cy()), hacia: destino, t: 0.0 });
                            }
                        }
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
        // vuelos al bol
        let mut llegados = vec![];
        for (i, v) in self.vuelos.iter_mut().enumerate() {
            v.t += dt / 0.45;
            if v.t >= 1.0 {
                llegados.push(i);
            }
        }
        for i in llegados.into_iter().rev() {
            let v = self.vuelos.remove(i);
            self.contenido.push((v.obj, v.hacia.0, v.hacia.1));
            self.puestos += 1;
            cx.sonar(Son::Pop);
            if let Some(Poner(_, n)) = self.paso_actual() {
                if self.puestos >= n {
                    self.paso_hecho(cx);
                    let (a, b) = alabanza(cx);
                    let (num_es, num_en) = (num(self.puestos), num_en(self.puestos));
                    cx.decir_v(&[num_es, a], &[num_en, b]);
                } else {
                    cx.decir_num(self.puestos);
                }
            }
        }
        if let Some(c) = &mut self.cuenta {
            let antes = c.ceil() as usize;
            *c -= dt;
            let ahora = c.max(0.0).ceil() as usize;
            if ahora != antes {
                if ahora == 0 {
                    self.cuenta = None;
                    self.mezclado = true;
                    self.paso_hecho(cx);
                    cx.decir_bil("¡Listo!", "Ready!");
                } else {
                    cx.sonar(Son::Tic);
                    cx.decir_num(ahora);
                }
            }
        }
        if self.listo_t > 0.0 {
            self.listo_t -= dt;
            if self.listo_t <= 0.0 {
                self.paso += 1;
                self.empezar_paso(cx);
            }
        }
    }

    fn cursor(&self) -> Cursor {
        if matches!(self.fase, Fase::Pasos) && matches!(self.paso_actual(), Some(Revolver)) && !self.mezclado {
            Cursor::Cuchara
        } else {
            Cursor::Normal
        }
    }

    fn dibujar(&mut self, l: &mut Lienzo, cx: &Cx) {
        l.degradado_v(0.0, 0.0, AW, AH, rgb(255, 244, 230), rgb(250, 225, 200));
        // azulejos
        for i in 0..14 {
            l.rect(i as f32 * 75.0, 0.0, 2.0, 80.0, rgb(240, 220, 200));
        }
        match self.fase {
            Fase::Elegir => {
                l.texto_c("¿Qué cocinamos hoy?", AW / 2.0, 60.0, 46.0, TINTA);
                for (i, r) in RECETAS.iter().enumerate() {
                    let t = tarjeta_r(i);
                    let sobre = t.dentro(cx.rl);
                    let t = if sobre { t.crecer(5.0) } else { t };
                    l.rrect_r(t.mover(0.0, 6.0), 22.0, alfa(TINTA, 0.2));
                    l.rrect_r(t, 22.0, BLANCO);
                    l.rrect(t.x, t.y + t.h - 70.0, t.w, 70.0, 22.0, ARCOIRIS[i * 2 % 8]);
                    dibujar_plato(l, r.plato, t.cx(), t.y + 105.0, 160.0);
                    let tam = if r.es.len() > 12 { 24.0 } else { 32.0 };
                    l.texto_c(r.es, t.cx(), t.y + t.h - 35.0, tam, BLANCO);
                }
            }
            Fase::Pasos => {
                // pasos arriba
                let rec = self.rec();
                let n = rec.pasos.len();
                for i in 0..n {
                    let x = AW / 2.0 - (n as f32 - 1.0) * 22.0 + i as f32 * 44.0;
                    let c = if i < self.paso { VERDE } else if i == self.paso { NARANJO } else { rgb(220, 210, 200) };
                    l.circulo(x, 30.0, 13.0, c);
                }
                let instr = match self.paso_actual() {
                    Some(Poner(o, n)) => frase_pon(o, n).0,
                    Some(Revolver) => "¡Revuelve, revuelve!".into(),
                    Some(Horno) => "Al horno".into(),
                    Some(Sarten) => "A la sartén".into(),
                    None => String::new(),
                };
                l.texto_c(&instr, AW / 2.0 + 60.0, 72.0, 36.0, TINTA);
                match self.paso_actual() {
                    Some(Poner(o, n)) => {
                        for (i, e) in self.estante.iter().enumerate() {
                            let mut r = estante_r(i);
                            if let Some((k, t)) = self.sacudir {
                                if k == i {
                                    r = r.mover((t * 40.0).sin() * 10.0 * t, 0.0);
                                }
                            }
                            let sobre = r.dentro(cx.rl);
                            let r = if sobre { r.crecer(4.0) } else { r };
                            l.rrect_r(r.mover(0.0, 5.0), 20.0, alfa(TINTA, 0.2));
                            l.rrect_r(r, 20.0, BLANCO);
                            dibujos::dibujar(l, *e, r.cx(), r.cy(), 96.0);
                        }
                        self.dibujar_bol(l, cx);
                        // casillas para contar
                        for k in 0..n {
                            let x = BOL.0 - (n as f32 - 1.0) * 30.0 + k as f32 * 60.0;
                            let y = 140.0;
                            l.circulo(x, y, 25.0, if k < self.puestos { rgb(200, 240, 200) } else { BLANCO });
                            l.anillo(x, y, 25.0, 3.0, if k < self.puestos { VERDE } else { rgb(210, 200, 190) });
                            if k < self.puestos {
                                dibujos::dibujar(l, o, x, y, 36.0);
                            } else {
                                l.texto_c(&(k + 1).to_string(), x, y, 26.0, rgb(190, 180, 170));
                            }
                        }
                        for v in &self.vuelos {
                            let t = ease_out(v.t);
                            let hasta = (BOL.0 + v.hacia.0, BOL.1 + v.hacia.1);
                            let x = v.desde.0 + (hasta.0 - v.desde.0) * t;
                            let y = v.desde.1 + (hasta.1 - v.desde.1) * t - (t * std::f32::consts::PI).sin() * 120.0;
                            dibujos::dibujar(l, v.obj, x, y, 70.0);
                        }
                    }
                    Some(Revolver) => {
                        self.dibujar_bol(l, cx);
                    }
                    Some(Horno) => self.dibujar_horno(l, cx, false),
                    Some(Sarten) => self.dibujar_horno(l, cx, true),
                    None => {}
                }
                if self.listo_t > 0.0 {
                    l.texto_c_borde("¡Bien!", 200.0, 300.0, 64.0, AMARILLO, TINTA, 3.0);
                }
            }
            Fase::Decorar | Fase::Final => {
                let rec = self.rec();
                l.texto_c(if matches!(self.fase, Fase::Final) { rec.textos[2] } else { rec.textos[0] }, AW / 2.0, 50.0, 40.0, TINTA);
                dibujar_plato(l, rec.plato, PLATO.0, PLATO.1, 420.0);
                for (o, dx, dy) in &self.deco {
                    dibujos::dibujar(l, *o, PLATO.0 + dx, PLATO.1 + dy, 52.0);
                }
                if matches!(self.fase, Fase::Decorar) {
                    for i in 0..3 {
                        let r = deco_r(i);
                        let sel = i == self.deco_sel;
                        l.rrect_r(r, 20.0, if sel { rgb(255, 240, 180) } else { BLANCO });
                        l.rrect_borde(r, 20.0, if sel { 5.0 } else { 2.0 }, if sel { NARANJO } else { rgb(220, 210, 200) });
                        dibujos::dibujar(l, rec.deco[i], r.cx(), r.cy(), 76.0);
                    }
                }
                let txt = if matches!(self.fase, Fase::Final) { "Otra receta" } else { "¡Terminé!" };
                boton_texto(l, B_LISTO, txt, VERDE, true, cx);
                if matches!(self.fase, Fase::Final) {
                    felicitar(l, "¡Qué rico!", PLATO.0, 90.0, cx.t);
                }
            }
        }
        if !matches!(self.fase, Fase::Elegir) {
            boton_flecha(l, VOLVER.0, VOLVER.1, 26.0, -1.0, NARANJO, cx);
        }
        self.confeti.dibujar(l);
    }
}
