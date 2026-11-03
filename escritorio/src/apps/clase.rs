//! Mi clase: armar láminas (texto, lápiz, stickers) y presentarlas a un curso de animalitos.
//! Se guarda en ~/Clases/mi-clase/.

use super::*;
use crate::cx::Son;
use crate::dibujos::{self, Obj, TODOS};
use crate::efectos::Confeti;
use std::f32::consts::FRAC_PI_2;
use tiny_skia::{FilterQuality, Pixmap, PixmapPaint, Transform};

const SW: f32 = 740.0;
const SH: f32 = 416.0;
const SLIDE: R = R::new(126.0, 86.0, SW, SH);
const MAX_DIAPOS: usize = 8;
const STICKERS: [Obj; 10] = [Obj::Estrella, Obj::Corazon, Obj::Sol, Obj::Flor, Obj::Gato, Obj::Perro, Obj::Pajaro, Obj::Casa, Obj::Arbol, Obj::Manzana];
const COLORES_LAPIZ: [C; 5] = [TINTA, ROJO, AZUL, VERDE, NARANJO];
const COLORES_TITULO: [C; 5] = [AZUL, MORADO, VERDE, NARANJO, ROSADO];

#[derive(Clone, Copy, PartialEq)]
enum Herr {
    Texto,
    Lapiz(usize),
    Sticker(Obj),
}

struct Diapo {
    color: usize,
    texto: String,
    stickers: Vec<(Obj, f32, f32)>,
    dibujo: Pixmap,
}

impl Diapo {
    fn nueva(color: usize) -> Diapo {
        Diapo { color, texto: String::new(), stickers: vec![], dibujo: Pixmap::new(SW as u32, SH as u32).unwrap() }
    }
}

enum Modo {
    Editar,
    Presentar { fin: bool, t: f32 },
}

pub struct Clase {
    diapos: Vec<Diapo>,
    actual: usize,
    modo: Modo,
    herr: Herr,
    arrastre: Option<(usize, f32, f32)>,
    trazando: Option<(f32, f32)>,
    borrar_t: f32,
    confeti: Confeti,
    render: Pixmap,
    miniaturas: Vec<Pixmap>,
    mini_sucia: bool,
}

const B_TEXTO: R = R::new(126.0, 14.0, 120.0, 56.0);
const B_LAPIZ: R = R::new(254.0, 14.0, 120.0, 56.0);
const B_PRESENTAR: R = R::new(736.0, 14.0, 180.0, 56.0);
const PAPELERA: (f32, f32) = (952.0, 42.0);
const MW: f32 = 100.0;
const MH: f32 = 56.0;

fn mini_r(i: usize) -> R {
    R::new(12.0, 86.0 + i as f32 * 66.0, MW, MH)
}
fn color_r(i: usize) -> (f32, f32) {
    (400.0 + i as f32 * 46.0, 42.0)
}
fn sticker_r(i: usize) -> R {
    R::new(878.0 + (i % 2) as f32 * 58.0, 86.0 + (i / 2) as f32 * 84.0, 54.0, 76.0)
}

fn dir_clase() -> std::path::PathBuf {
    ruta_datos("Clases").join("mi-clase")
}

pub fn frase_fin(nombre: &str) -> String {
    if nombre.trim().is_empty() {
        "¡Muy buena clase, profesora!".into()
    } else {
        format!("¡Muy buena clase, profesora {}!", nombre.trim())
    }
}

pub fn frases(f: &mut Frases, nombre: &str) {
    f.bil("Nueva lámina.", "New slide.");
    f.bil("¡A presentar!", "Let's present!");
    f.es(frase_fin(nombre));
    f.en("Great class, teacher!");
    f.bil("¿Borrar la lámina? Toca otra vez.", "Erase the slide? Touch again.");
    f.bil("Escribe con el teclado.", "Type with the keyboard.");
    f.bil("lápiz", "pencil");
    f.bil("Fin de la clase.", "The end.");
    for (a, b) in [("negro", "black"), ("rojo", "red"), ("azul", "blue"), ("verde", "green"), ("naranjo", "orange")] {
        f.bil(a, b);
    }
}

impl Clase {
    pub fn nuevo(cx: &mut Cx) -> Box<dyn App> {
        let mut c = Clase {
            diapos: vec![],
            actual: 0,
            modo: Modo::Editar,
            herr: Herr::Texto,
            arrastre: None,
            trazando: None,
            borrar_t: 0.0,
            confeti: Confeti::default(),
            render: Pixmap::new(SW as u32, SH as u32).unwrap(),
            miniaturas: vec![],
            mini_sucia: true,
        };
        c.cargar();
        if c.diapos.is_empty() {
            c.diapos.push(Diapo::nueva(0));
        }
        cx.decir_bil("Escribe con el teclado.", "Type with the keyboard.");
        Box::new(c)
    }

    fn guardar(&self) {
        let d = dir_clase();
        if std::fs::create_dir_all(&d).is_err() {
            return;
        }
        let mut s = String::new();
        for (i, di) in self.diapos.iter().enumerate() {
            s.push_str(&format!("DIAPO {}\n", di.color));
            s.push_str(&format!("TEXTO {}\n", di.texto.replace('\\', "\\\\").replace('\n', "\\n")));
            for (o, x, y) in &di.stickers {
                if let Some(k) = TODOS.iter().position(|t| t == o) {
                    s.push_str(&format!("STICKER {k} {x} {y}\n"));
                }
            }
            let _ = di.dibujo.save_png(d.join(format!("dibujo-{i}.png")));
        }
        let _ = std::fs::write(d.join("clase.txt"), s);
    }

    fn cargar(&mut self) {
        let d = dir_clase();
        let Ok(txt) = std::fs::read_to_string(d.join("clase.txt")) else { return };
        for linea in txt.lines() {
            let (k, v) = linea.split_once(' ').unwrap_or((linea, ""));
            match k {
                "DIAPO" => {
                    let i = self.diapos.len();
                    let mut di = Diapo::nueva(v.trim().parse().unwrap_or(0) % COLORES_TITULO.len());
                    if let Ok(p) = Pixmap::load_png(d.join(format!("dibujo-{i}.png"))) {
                        if p.width() == SW as u32 && p.height() == SH as u32 {
                            di.dibujo = p;
                        }
                    }
                    self.diapos.push(di);
                }
                "TEXTO" => {
                    if let Some(di) = self.diapos.last_mut() {
                        let mut out = String::new();
                        let mut it = v.chars();
                        while let Some(c) = it.next() {
                            if c == '\\' {
                                match it.next() {
                                    Some('n') => out.push('\n'),
                                    Some(o) => out.push(o),
                                    None => {}
                                }
                            } else {
                                out.push(c);
                            }
                        }
                        di.texto = out;
                    }
                }
                "STICKER" => {
                    let p: Vec<&str> = v.split_whitespace().collect();
                    if let (Some(di), [k, x, y]) = (self.diapos.last_mut(), p.as_slice()) {
                        if let (Ok(k), Ok(x), Ok(y)) = (k.parse::<usize>(), x.parse::<f32>(), y.parse::<f32>()) {
                            if let Some(o) = TODOS.get(k) {
                                di.stickers.push((*o, x, y));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        self.diapos.truncate(MAX_DIAPOS);
    }

    /// Dibuja la lámina `i` en `self.render` (tamaño SW x SH).
    fn renderizar(&mut self, l: &mut Lienzo, i: usize) {
        let di = &self.diapos[i];
        let mut pix = std::mem::replace(&mut self.render, Pixmap::new(1, 1).unwrap());
        l.con(&mut pix, |l| {
            l.rect(0.0, 0.0, SW, SH, BLANCO);
            let col = COLORES_TITULO[di.color];
            l.rect(0.0, 0.0, SW, 86.0, col);
            l.rect(0.0, 86.0, SW, 6.0, oscurecer(col, 0.2));
            let mut lineas = di.texto.split('\n');
            let titulo = lineas.next().unwrap_or("");
            l.texto(titulo, 28.0, 14.0, 52.0, BLANCO);
            let mut y = 110.0;
            for li in lineas {
                l.circulo(40.0, y + 26.0, 7.0, col);
                l.texto(li, 60.0, y, 44.0, TINTA);
                y += 58.0;
            }
            l.pix.draw_pixmap(0, 0, di.dibujo.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
            for (o, x, y) in &di.stickers {
                dibujos::dibujar(l, *o, *x, *y, 90.0);
            }
            l.texto(&format!("{}", i + 1), SW - 40.0, SH - 44.0, 30.0, rgb(190, 190, 200));
        });
        self.render = pix;
    }

    fn actualizar_miniaturas(&mut self, l: &mut Lienzo) {
        self.miniaturas.clear();
        for i in 0..self.diapos.len() {
            self.renderizar(l, i);
            let mut m = Pixmap::new(MW as u32, MH as u32).unwrap();
            let pp = PixmapPaint { quality: FilterQuality::Bilinear, ..Default::default() };
            m.draw_pixmap(0, 0, self.render.as_ref(), &pp, Transform::from_scale(MW / SW, MH / SH), None);
            self.miniaturas.push(m);
        }
        self.mini_sucia = false;
    }

    fn en_lamina(p: (f32, f32)) -> Option<(f32, f32)> {
        if SLIDE.dentro(p) {
            Some((p.0 - SLIDE.x, p.1 - SLIDE.y))
        } else {
            None
        }
    }

    fn evento_editar(&mut self, ev: &Ev, cx: &mut Cx) {
        match ev {
            Ev::Clic(x, y) => {
                let p = (*x, *y);
                if B_PRESENTAR.dentro(p) {
                    self.modo = Modo::Presentar { fin: false, t: 0.0 };
                    self.actual = 0;
                    cx.sonar(Son::Whoosh);
                    cx.decir_bil("¡A presentar!", "Let's present!");
                    return;
                }
                if B_TEXTO.dentro(p) {
                    self.herr = Herr::Texto;
                    cx.sonar(Son::Clic);
                    cx.decir_bil("Escribe con el teclado.", "Type with the keyboard.");
                    return;
                }
                if B_LAPIZ.dentro(p) {
                    self.herr = Herr::Lapiz(match self.herr {
                        Herr::Lapiz(c) => c,
                        _ => 0,
                    });
                    cx.sonar(Son::Clic);
                    cx.decir_bil("lápiz", "pencil");
                    return;
                }
                for i in 0..COLORES_LAPIZ.len() {
                    if dist(p, color_r(i)) < 20.0 {
                        self.herr = Herr::Lapiz(i);
                        cx.sonar(Son::Clic);
                        let n = [("negro", "black"), ("rojo", "red"), ("azul", "blue"), ("verde", "green"), ("naranjo", "orange")][i];
                        cx.decir_bil(n.0, n.1);
                        return;
                    }
                }
                if dist(p, PAPELERA) < 28.0 {
                    if self.borrar_t > 0.0 {
                        let c = self.diapos[self.actual].color;
                        self.diapos[self.actual] = Diapo::nueva(c);
                        self.borrar_t = 0.0;
                        self.mini_sucia = true;
                        cx.sonar(Son::Whoosh);
                    } else {
                        self.borrar_t = 2.5;
                        cx.decir_bil("¿Borrar la lámina? Toca otra vez.", "Erase the slide? Touch again.");
                    }
                    return;
                }
                for (i, o) in STICKERS.iter().enumerate() {
                    if sticker_r(i).dentro(p) {
                        self.herr = Herr::Sticker(*o);
                        cx.sonar(Son::Clic);
                        cx.decir_obj(*o);
                        return;
                    }
                }
                for i in 0..self.diapos.len() {
                    if mini_r(i).dentro(p) {
                        self.actual = i;
                        cx.sonar(Son::Clic);
                        cx.decir_num(i + 1);
                        return;
                    }
                }
                let n = self.diapos.len();
                if n < MAX_DIAPOS && mini_r(n).dentro(p) {
                    self.diapos.push(Diapo::nueva(n % COLORES_TITULO.len()));
                    self.actual = n;
                    self.mini_sucia = true;
                    cx.sonar(Son::Pop);
                    cx.decir_bil("Nueva lámina.", "New slide.");
                    return;
                }
                if let Some(lp) = Clase::en_lamina(p) {
                    // ¿tomó un sticker para moverlo?
                    let di = &self.diapos[self.actual];
                    if let Some(k) = di.stickers.iter().rposition(|(_, sx, sy)| dist(lp, (*sx, *sy)) < 45.0) {
                        let (_, sx, sy) = di.stickers[k];
                        self.arrastre = Some((k, sx - lp.0, sy - lp.1));
                        return;
                    }
                    match self.herr {
                        Herr::Sticker(o) => {
                            self.diapos[self.actual].stickers.push((o, lp.0, lp.1));
                            self.mini_sucia = true;
                            cx.sonar(Son::Sello);
                        }
                        Herr::Lapiz(c) => {
                            self.diapos[self.actual].dibujo.linea(lp.0, lp.1, lp.0, lp.1, 7.0, COLORES_LAPIZ[c]);
                            self.trazando = Some(lp);
                            self.mini_sucia = true;
                        }
                        Herr::Texto => {}
                    }
                }
            }
            Ev::Mueve(x, y) => {
                let lp = (x - SLIDE.x, y - SLIDE.y);
                if let Some((k, dx, dy)) = self.arrastre {
                    let s = &mut self.diapos[self.actual].stickers[k];
                    s.1 = (lp.0 + dx).clamp(0.0, SW);
                    s.2 = (lp.1 + dy).clamp(0.0, SH);
                    self.mini_sucia = true;
                } else if let (Some(a), Herr::Lapiz(c)) = (self.trazando, self.herr) {
                    self.diapos[self.actual].dibujo.linea(a.0, a.1, lp.0, lp.1, 7.0, COLORES_LAPIZ[c]);
                    self.trazando = Some(lp);
                }
            }
            Ev::Suelta(..) => {
                self.arrastre = None;
                if self.trazando.take().is_some() {
                    self.mini_sucia = true;
                }
            }
            Ev::Texto(t) => {
                for c in t.chars().filter(|c| !c.is_control()) {
                    let di = &mut self.diapos[self.actual];
                    if di.texto.chars().count() < 200 {
                        di.texto.push(c);
                        self.mini_sucia = true;
                        if let Some(n) = nombre_letra(c) {
                            cx.decir_bil(&n, &mayus(base(c)).to_string());
                        }
                    }
                }
            }
            Ev::Tecla(T::Espacio) => {
                self.diapos[self.actual].texto.push(' ');
            }
            Ev::Tecla(T::Enter) => {
                let di = &mut self.diapos[self.actual];
                if di.texto.split('\n').count() < 6 {
                    di.texto.push('\n');
                    self.mini_sucia = true;
                }
            }
            Ev::Tecla(T::Borrar) => {
                if self.diapos[self.actual].texto.pop().is_some() {
                    self.mini_sucia = true;
                    cx.sonar(Son::Clic);
                }
            }
            _ => {}
        }
    }

    fn avanzar(&mut self, d: i32, cx: &mut Cx) {
        let Modo::Presentar { fin, t } = &mut self.modo else { return };
        if *fin {
            if d < 0 {
                *fin = false;
            } else {
                self.modo = Modo::Editar;
            }
            return;
        }
        let n = self.diapos.len() as i32;
        let nuevo = self.actual as i32 + d;
        if nuevo >= n {
            *fin = true;
            *t = 0.0;
            cx.sonar(Son::Aplauso);
            self.confeti.lanzar(AW / 2.0, AH / 2.0, 120, &mut cx.rng);
            let es = frase_fin(&cx.nombre);
            cx.decir_v(&[es.as_str()], &["Great class, teacher!"]);
        } else if nuevo >= 0 {
            self.actual = nuevo as usize;
            *t = 0.0;
            cx.sonar(Son::Whoosh);
        }
    }
}

impl App for Clase {
    fn animando(&self) -> bool {
        self.borrar_t > 0.0 || self.confeti.activo() || matches!(self.modo, Modo::Presentar { fin: true, .. }) || matches!(self.modo, Modo::Presentar { t, .. } if t < 0.4)
    }

    fn evento(&mut self, ev: &Ev, cx: &mut Cx) {
        match self.modo {
            Modo::Editar => self.evento_editar(ev, cx),
            Modo::Presentar { .. } => match ev {
                Ev::Clic(x, _) => {
                    if *x < 120.0 {
                        self.avanzar(-1, cx)
                    } else {
                        self.avanzar(1, cx)
                    }
                }
                Ev::Tecla(T::Izq) | Ev::Tecla(T::Borrar) => self.avanzar(-1, cx),
                Ev::Tecla(T::Escape) => self.modo = Modo::Editar,
                Ev::Tecla(_) => self.avanzar(1, cx),
                _ => {}
            },
        }
    }

    fn actualizar(&mut self, dt: f32, _cx: &mut Cx) {
        if self.borrar_t > 0.0 {
            self.borrar_t -= dt;
        }
        if let Modo::Presentar { t, .. } = &mut self.modo {
            *t += dt;
        }
        self.confeti.actualizar(dt);
    }

    fn cerrar(&mut self, _cx: &mut Cx) {
        self.guardar();
    }

    fn cursor(&self) -> Cursor {
        if matches!(self.modo, Modo::Presentar { .. }) {
            Cursor::Laser
        } else {
            Cursor::Normal
        }
    }

    fn dibujar(&mut self, l: &mut Lienzo, cx: &Cx) {
        match self.modo {
            Modo::Editar => {
                if self.mini_sucia && !cx.apretado {
                    self.actualizar_miniaturas(l);
                }
                self.renderizar(l, self.actual);
                l.rect(0.0, 0.0, AW, AH, rgb(236, 240, 248));
                // barra de herramientas
                boton_texto(l, B_TEXTO, "Texto", AZUL, self.herr == Herr::Texto, cx);
                boton_texto(l, B_LAPIZ, "Lápiz", VERDE, matches!(self.herr, Herr::Lapiz(_)), cx);
                for (i, c) in COLORES_LAPIZ.iter().enumerate() {
                    let (x, y) = color_r(i);
                    if self.herr == Herr::Lapiz(i) {
                        l.circulo(x, y, 21.0, TINTA);
                    }
                    l.circulo(x, y, 17.0, *c);
                    if *c == TINTA {
                        l.anillo(x, y, 17.0, 2.0, BLANCO);
                    }
                }
                boton_texto(l, B_PRESENTAR, "Presentar", ROJO, true, cx);
                boton_papelera(l, PAPELERA.0, PAPELERA.1, 26.0, self.borrar_t, cx);
                // miniaturas
                for (i, m) in self.miniaturas.iter().enumerate() {
                    let r = mini_r(i);
                    if i == self.actual {
                        l.rrect_r(r.crecer(4.0), 6.0, NARANJO);
                    }
                    copiar_opaco(&mut l.pix, m, r.x as i32, r.y as i32);
                }
                let n = self.diapos.len();
                if n < MAX_DIAPOS {
                    let r = mini_r(n);
                    let sobre = r.dentro(cx.rl);
                    l.rrect_r(r, 8.0, if sobre { rgb(210, 240, 210) } else { BLANCO });
                    l.rrect_borde(r, 8.0, 2.0, VERDE);
                    l.texto_c("+", r.cx(), r.cy(), 48.0, VERDE);
                }
                // lámina
                l.rect(SLIDE.x + 4.0, SLIDE.y + 6.0, SW, SH, alfa(TINTA, 0.2));
                copiar_opaco(&mut l.pix, &self.render, SLIDE.x as i32, SLIDE.y as i32);
                if self.herr == Herr::Texto && (cx.t * std::f32::consts::TAU).sin() > 0.0 {
                    // indicador de que se puede escribir
                    let di = &self.diapos[self.actual];
                    let ultima = di.texto.split('\n').next_back().unwrap_or("");
                    let nl = di.texto.split('\n').count();
                    let (tam, x0, y0) = if nl <= 1 { (52.0, 28.0, 14.0) } else { (44.0, 60.0, 110.0 + (nl - 2) as f32 * 58.0) };
                    let w = l.medir(ultima, tam);
                    let col = if nl <= 1 { BLANCO } else { TINTA };
                    l.rect(SLIDE.x + x0 + w + 3.0, SLIDE.y + y0 + 8.0, 5.0, tam, col);
                }
                // stickers
                for (i, o) in STICKERS.iter().enumerate() {
                    let r = sticker_r(i);
                    let sel = self.herr == Herr::Sticker(*o);
                    l.rrect_r(r, 12.0, if sel { rgb(255, 240, 180) } else { BLANCO });
                    if sel {
                        l.rrect_borde(r, 12.0, 4.0, NARANJO);
                    }
                    dibujos::dibujar(l, *o, r.cx(), r.cy(), 46.0);
                }
            }
            Modo::Presentar { fin, t } => {
                l.rect(0.0, 0.0, AW, AH, rgb(40, 44, 60));
                if fin {
                    l.degradado_v(0.0, 0.0, AW, AH, rgb(60, 50, 110), rgb(30, 30, 60));
                    l.texto_c_borde("Fin de la clase", AW / 2.0, 150.0, 70.0, AMARILLO, TINTA, 4.0);
                } else {
                    self.renderizar(l, self.actual);
                    let esc = 1.12;
                    let w = SW * esc;
                    let x = (AW - w) / 2.0;
                    let entra = ease_out((t / 0.3).min(1.0));
                    let pp = PixmapPaint { quality: FilterQuality::Bilinear, ..Default::default() };
                    let tr = Transform::from_row(esc, 0.0, 0.0, esc, x + (1.0 - entra) * 60.0, 8.0);
                    l.pix.draw_pixmap(0, 0, self.render.as_ref(), &pp, tr, None);
                    l.texto(&format!("{} / {}", self.actual + 1, self.diapos.len()), 14.0, AH - 40.0, 26.0, rgb(160, 165, 190));
                }
                // el curso: animalitos mirando
                let curso = [Obj::Gato, Obj::Perro, Obj::Pajaro, Obj::Gato, Obj::Perro, Obj::Pajaro];
                for (i, o) in curso.iter().enumerate() {
                    let x = 170.0 + i as f32 * 132.0;
                    let salto = if fin { ((t * 9.0 + i as f32).sin().abs()) * 40.0 } else { (cx.t * 2.0 + i as f32).sin() * 3.0 };
                    let y = if fin { 340.0 } else { AH - 10.0 } - salto;
                    dibujos::dibujar(l, *o, x, y, if fin { 120.0 } else { 90.0 });
                    if fin && (t * 9.0 + i as f32).sin() > 0.0 {
                        l.estrella(x + 50.0, y - 50.0, 12.0, -FRAC_PI_2, AMARILLO);
                    }
                }
            }
        }
        self.confeti.dibujar(l);
    }
}
