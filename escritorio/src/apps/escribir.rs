//! Escribir: procesador de texto tipo WordPad. Tamaños, colores, centrado, stickers en el texto,
//! cursor con flechas o clic, Nuevo / Abrir / Guardar en ~/Documentos.

use super::*;
use crate::cx::Son;
use crate::dibujos::{self, Obj, TODOS};

#[derive(Clone, Copy, PartialEq)]
enum Cont {
    Letra(char),
    Sticker(Obj),
    Salto,
}

#[derive(Clone, Copy)]
struct Pieza {
    c: Cont,
    tam: usize,
    color: C,
    centro: bool,
}

const TAMS: [f32; 3] = [30.0, 46.0, 70.0];
const TAM_NOMBRES: [(&str, &str); 3] = [("chico", "small"), ("mediano", "medium"), ("grande", "big")];
const COLORES: [(&str, &str, C); 6] =
    [("negro", "black", TINTA), ("rojo", "red", ROJO), ("azul", "blue", AZUL), ("verde", "green", VERDE), ("morado", "purple", MORADO), ("naranjo", "orange", NARANJO)];
const STICKERS: [Obj; 10] = [Obj::Corazon, Obj::Estrella, Obj::Sol, Obj::Flor, Obj::Gato, Obj::Perro, Obj::Casa, Obj::Arbol, Obj::Manzana, Obj::Galleta];

const PAGINA: R = R::new(150.0, 76.0, 700.0, 442.0);
const MARGEN: f32 = 28.0;

#[derive(Clone, Copy, PartialEq)]
enum Bot {
    Nuevo,
    Abrir,
    Guardar,
    Tam(usize),
    Color(usize),
    Izq,
    Centro,
    Leer,
}

fn botones() -> Vec<(R, Bot)> {
    let mut v = vec![
        (R::new(10.0, 8.0, 64.0, 58.0), Bot::Nuevo),
        (R::new(78.0, 8.0, 64.0, 58.0), Bot::Abrir),
        (R::new(146.0, 8.0, 64.0, 58.0), Bot::Guardar),
    ];
    for i in 0..3 {
        v.push((R::new(230.0 + i as f32 * 52.0, 12.0, 48.0, 50.0), Bot::Tam(i)));
    }
    for i in 0..6 {
        v.push((R::new(400.0 + i as f32 * 44.0, 16.0, 40.0, 42.0), Bot::Color(i)));
    }
    v.push((R::new(678.0, 12.0, 50.0, 50.0), Bot::Izq));
    v.push((R::new(732.0, 12.0, 50.0, 50.0), Bot::Centro));
    v.push((R::new(880.0, 10.0, 110.0, 54.0), Bot::Leer));
    v
}

fn sticker_r(i: usize) -> R {
    R::new(870.0 + (i % 2) as f32 * 62.0, 84.0 + (i / 2) as f32 * 84.0, 56.0, 76.0)
}

const DIR_DOCS: &str = "Documentos";

pub fn frases(f: &mut Frases) {
    f.bil("Escribe lo que quieras.", "Write anything you want.");
    f.bil("Documento nuevo.", "New document.");
    f.bil("Guardado.", "Saved.");
    f.bil("Abrir", "Open");
    f.bil("Centrado", "Centered");
    f.bil("A la izquierda", "Left");
    f.bil("Todavía no hay documentos.", "No documents yet.");
    for (a, b) in TAM_NOMBRES {
        f.bil(a, b);
    }
    for (a, b, _) in COLORES {
        f.bil(a, b);
    }
}

struct Lugar {
    x: f32,
    y: f32,
    w: f32,
    alto: f32,
}

pub struct Escribir {
    doc: Vec<Pieza>,
    cursor: usize,
    tam: usize,
    color: usize,
    centro: bool,
    archivo: Option<String>,
    cambiado: bool,
    scroll: f32,
    lugares: Vec<Lugar>,
    fin: Lugar,
    abrir: Option<Vec<(String, String)>>,
    aviso: f32,
}

impl Escribir {
    pub fn nuevo(cx: &mut Cx) -> Box<dyn App> {
        cx.decir_bil("Escribe lo que quieras.", "Write anything you want.");
        Box::new(Escribir {
            doc: vec![],
            cursor: 0,
            tam: 1,
            color: 0,
            centro: false,
            archivo: None,
            cambiado: false,
            scroll: 0.0,
            lugares: vec![],
            fin: Lugar { x: 0.0, y: 0.0, w: 0.0, alto: 40.0 },
            abrir: None,
            aviso: 0.0,
        })
    }

    fn insertar(&mut self, c: Cont) {
        let p = Pieza { c, tam: self.tam, color: COLORES[self.color].2, centro: self.centro };
        self.doc.insert(self.cursor, p);
        self.cursor += 1;
        self.cambiado = true;
    }

    fn texto_plano(&self) -> String {
        self.doc
            .iter()
            .map(|p| match p.c {
                Cont::Letra(c) => c.to_string(),
                Cont::Sticker(o) => format!(" {} ", o.singular()),
                Cont::Salto => "\n".into(),
            })
            .collect()
    }

    fn guardar(&mut self) -> bool {
        if self.doc.is_empty() || !self.cambiado {
            return false;
        }
        let ruta = self.archivo.clone().unwrap_or_else(|| format!("{DIR_DOCS}/documento-{}.nid", crate::almacen::segundos()));
        let mut s = String::from("NIDITO-DOC 1\n");
        for p in &self.doc {
            let (k, v) = match p.c {
                Cont::Letra(c) => ("L", (c as u32).to_string()),
                Cont::Sticker(o) => ("S", TODOS.iter().position(|t| *t == o).unwrap_or(0).to_string()),
                Cont::Salto => ("N", "0".into()),
            };
            s.push_str(&format!("{k} {v} {} {} {} {} {}\n", p.tam, p.color[0], p.color[1], p.color[2], p.centro as u8));
        }
        if crate::almacen::escribir(&ruta, s.as_bytes()) {
            self.archivo = Some(ruta);
            self.cambiado = false;
            true
        } else {
            false
        }
    }

    fn cargar(&mut self, ruta: &str) {
        let Some(txt) = crate::almacen::leer_texto(ruta) else { return };
        let mut doc = vec![];
        for linea in txt.lines().skip(1) {
            let p: Vec<&str> = linea.split_whitespace().collect();
            if p.len() != 7 {
                continue;
            }
            let n = |i: usize| p[i].parse::<u32>().unwrap_or(0);
            let c = match p[0] {
                "L" => char::from_u32(n(1)).map(Cont::Letra),
                "S" => TODOS.get(n(1) as usize).map(|o| Cont::Sticker(*o)),
                "N" => Some(Cont::Salto),
                _ => None,
            };
            if let Some(c) = c {
                doc.push(Pieza { c, tam: (n(2) as usize).min(2), color: rgb(n(3) as u8, n(4) as u8, n(5) as u8), centro: n(6) == 1 });
            }
        }
        self.doc = doc;
        self.cursor = self.doc.len();
        self.archivo = Some(ruta.to_string());
        self.cambiado = false;
        self.scroll = 0.0;
    }

    fn lista_docs() -> Vec<(String, String)> {
        let mut v: Vec<String> = crate::almacen::listar(DIR_DOCS, "nid").into_iter().map(|(r, _)| r).collect();
        v.sort();
        v.reverse();
        v.truncate(12);
        v.into_iter()
            .map(|p| {
                let mut e = Escribir {
                    doc: vec![],
                    cursor: 0,
                    tam: 1,
                    color: 0,
                    centro: false,
                    archivo: None,
                    cambiado: false,
                    scroll: 0.0,
                    lugares: vec![],
                    fin: Lugar { x: 0.0, y: 0.0, w: 0.0, alto: 0.0 },
                    abrir: None,
                    aviso: 0.0,
                };
                e.cargar(&p);
                let t: String = e.texto_plano().replace('\n', " ").chars().take(22).collect();
                (p, if t.trim().is_empty() { "(dibujos)".into() } else { t })
            })
            .collect()
    }

    /// Párrafo actual: aplica el centrado a todas sus piezas.
    fn alinear(&mut self, centro: bool) {
        self.centro = centro;
        let mut a = self.cursor.min(self.doc.len());
        while a > 0 && self.doc[a - 1].c != Cont::Salto {
            a -= 1;
        }
        let mut b = self.cursor.min(self.doc.len());
        while b < self.doc.len() && self.doc[b].c != Cont::Salto {
            b += 1;
        }
        for p in &mut self.doc[a..b] {
            p.centro = centro;
        }
        if b < self.doc.len() {
            self.doc[b].centro = centro;
        }
        self.cambiado = true;
    }

    /// Calcula la posición de cada pieza (coordenadas de página, sin scroll).
    fn maquetar(&mut self, l: &mut Lienzo) {
        let ancho = PAGINA.w - 2.0 * MARGEN;
        let mut lugares: Vec<Lugar> = Vec::with_capacity(self.doc.len());
        let mut lineas: Vec<(usize, usize)> = vec![];
        let mut ini = 0;
        let mut x = 0.0;
        let mut ultimo_espacio: Option<usize> = None;
        let anchos: Vec<f32> = self
            .doc
            .iter()
            .map(|p| match p.c {
                Cont::Letra(c) => l.medir(&c.to_string(), TAMS[p.tam]),
                Cont::Sticker(_) => TAMS[p.tam] * 1.1,
                Cont::Salto => 0.0,
            })
            .collect();
        let mut i = 0;
        while i < self.doc.len() {
            let p = self.doc[i];
            if p.c == Cont::Salto {
                lineas.push((ini, i + 1));
                ini = i + 1;
                x = 0.0;
                ultimo_espacio = None;
                i += 1;
                continue;
            }
            if x + anchos[i] > ancho && i > ini {
                // cortar en el último espacio si hay
                let corte = match ultimo_espacio {
                    Some(e) if e > ini => e + 1,
                    _ => i,
                };
                lineas.push((ini, corte));
                ini = corte;
                x = anchos[ini..i].iter().sum();
                ultimo_espacio = None;
            }
            if p.c == Cont::Letra(' ') {
                ultimo_espacio = Some(i);
            }
            x += anchos[i];
            i += 1;
        }
        lineas.push((ini, self.doc.len()));
        lugares.resize_with(self.doc.len(), || Lugar { x: 0.0, y: 0.0, w: 0.0, alto: 0.0 });
        let mut y = MARGEN;
        let mut fin = Lugar { x: MARGEN, y: MARGEN, w: 0.0, alto: TAMS[self.tam] * 1.3 };
        for &(a, b) in &lineas {
            let alto = self.doc[a..b].iter().map(|p| TAMS[p.tam] * 1.3).fold(TAMS[self.tam] * 1.3 * 0.0 + 30.0 * 1.3, f32::max);
            let w: f32 = anchos[a..b].iter().sum();
            let centro = self.doc.get(a).map(|p| p.centro).unwrap_or(self.centro);
            let mut x = if centro { MARGEN + (ancho - w) / 2.0 } else { MARGEN };
            for k in a..b {
                lugares[k] = Lugar { x, y, w: anchos[k], alto };
                x += anchos[k];
            }
            fin = Lugar { x, y, w: 0.0, alto };
            y += alto;
        }
        if self.doc.last().is_some_and(|p| p.c == Cont::Salto) {
            let centro = self.centro;
            fin = Lugar { x: if centro { PAGINA.w / 2.0 } else { MARGEN }, y, w: 0.0, alto: TAMS[self.tam] * 1.3 };
        }
        self.lugares = lugares;
        self.fin = fin;
    }

    fn lugar_cursor(&self) -> (f32, f32, f32) {
        if self.cursor < self.lugares.len() {
            let l = &self.lugares[self.cursor];
            (l.x, l.y, l.alto)
        } else {
            (self.fin.x, self.fin.y, self.fin.alto)
        }
    }
}

impl App for Escribir {
    fn animando(&self) -> bool {
        self.aviso > 0.0
    }

    fn evento(&mut self, ev: &Ev, cx: &mut Cx) {
        if let Some(lista) = &self.abrir {
            if let Ev::Clic(x, y) = ev {
                let lista = lista.clone();
                self.abrir = None;
                for (i, (p, _)) in lista.iter().enumerate() {
                    let r = R::new(170.0 + (i % 3) as f32 * 225.0, 110.0 + (i / 3) as f32 * 96.0, 210.0, 84.0);
                    if r.dentro((*x, *y)) {
                        self.guardar();
                        self.cargar(p);
                        cx.sonar(Son::Whoosh);
                    }
                }
            }
            if let Ev::Tecla(T::Escape) = ev {
                self.abrir = None;
            }
            return;
        }
        match ev {
            Ev::Clic(x, y) => {
                let p = (*x, *y);
                for (r, b) in botones() {
                    if !r.dentro(p) {
                        continue;
                    }
                    cx.sonar(Son::Clic);
                    match b {
                        Bot::Nuevo => {
                            self.guardar();
                            self.doc.clear();
                            self.cursor = 0;
                            self.archivo = None;
                            self.scroll = 0.0;
                            cx.decir_bil("Documento nuevo.", "New document.");
                        }
                        Bot::Abrir => {
                            self.guardar();
                            let l = Escribir::lista_docs();
                            if l.is_empty() {
                                cx.decir_bil("Todavía no hay documentos.", "No documents yet.");
                            } else {
                                cx.decir_bil("Abrir", "Open");
                                self.abrir = Some(l);
                            }
                        }
                        Bot::Guardar => {
                            self.cambiado = true;
                            if self.guardar() {
                                self.aviso = 1.5;
                                cx.sonar(Son::Ding);
                                cx.decir_bil("Guardado.", "Saved.");
                            }
                        }
                        Bot::Tam(t) => {
                            self.tam = t;
                            cx.decir_bil(TAM_NOMBRES[t].0, TAM_NOMBRES[t].1);
                        }
                        Bot::Color(c) => {
                            self.color = c;
                            cx.decir_bil(COLORES[c].0, COLORES[c].1);
                        }
                        Bot::Izq => {
                            self.alinear(false);
                            cx.decir_bil("A la izquierda", "Left");
                        }
                        Bot::Centro => {
                            self.alinear(true);
                            cx.decir_bil("Centrado", "Centered");
                        }
                        Bot::Leer => {
                            let t = self.texto_plano().to_lowercase();
                            if !t.trim().is_empty() {
                                crate::palabras::leer(cx, &t);
                            }
                        }
                    }
                    return;
                }
                for (i, o) in STICKERS.iter().enumerate() {
                    if sticker_r(i).dentro(p) {
                        self.insertar(Cont::Sticker(*o));
                        cx.sonar(Son::Sello);
                        cx.decir_obj(*o);
                        return;
                    }
                }
                if PAGINA.dentro(p) {
                    // poner el cursor donde tocó
                    let (lx, ly) = (p.0 - PAGINA.x, p.1 - PAGINA.y + self.scroll);
                    let mut mejor = self.doc.len();
                    let mut en_linea = false;
                    for (k, lg) in self.lugares.iter().enumerate() {
                        if ly >= lg.y && ly < lg.y + lg.alto {
                            en_linea = true;
                            if lx < lg.x + lg.w / 2.0 {
                                mejor = k;
                                break;
                            }
                            mejor = k + 1;
                        } else if en_linea {
                            break;
                        }
                    }
                    if en_linea && mejor > 0 && self.doc.get(mejor - 1).is_some_and(|p| p.c == Cont::Salto) && mejor <= self.doc.len() {
                        mejor -= 1;
                    }
                    self.cursor = mejor.min(self.doc.len());
                    if let Some(p) = self.cursor.checked_sub(1).and_then(|k| self.doc.get(k)) {
                        self.centro = p.centro;
                    }
                }
            }
            Ev::Texto(t) => {
                for c in t.chars().filter(|c| !c.is_control()) {
                    self.insertar(Cont::Letra(c));
                    if let Some(n) = nombre_letra(c) {
                        cx.decir_bil(&n, &mayus(base(c)).to_string());
                    }
                }
            }
            Ev::Tecla(T::Espacio) => self.insertar(Cont::Letra(' ')),
            Ev::Tecla(T::Enter) => self.insertar(Cont::Salto),
            Ev::Tecla(T::Borrar) => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.doc.remove(self.cursor);
                    self.cambiado = true;
                    cx.sonar(Son::Clic);
                }
            }
            Ev::Tecla(T::Izq) => self.cursor = self.cursor.saturating_sub(1),
            Ev::Tecla(T::Der) => self.cursor = (self.cursor + 1).min(self.doc.len()),
            _ => {}
        }
    }

    fn actualizar(&mut self, dt: f32, _cx: &mut Cx) {
        self.aviso = (self.aviso - dt).max(0.0);
    }

    fn cerrar(&mut self, _cx: &mut Cx) {
        self.guardar();
    }

    fn dibujar(&mut self, l: &mut Lienzo, cx: &Cx) {
        self.maquetar(l);
        l.rect(0.0, 0.0, AW, AH, rgb(200, 206, 218));
        // página con desplazamiento para que el cursor siempre se vea
        let (_, cy, ca) = self.lugar_cursor();
        let vis = PAGINA.h - 10.0;
        if cy + ca - self.scroll > vis {
            self.scroll = cy + ca - vis;
        }
        if cy < self.scroll {
            self.scroll = cy;
        }
        let px = PAGINA.x;
        let py = PAGINA.y - self.scroll;
        l.rect(px + 5.0, PAGINA.y + 6.0, PAGINA.w, PAGINA.h + 40.0, alfa(TINTA, 0.18));
        l.rect(px, PAGINA.y, PAGINA.w, PAGINA.h + 40.0, BLANCO);
        for (k, p) in self.doc.iter().enumerate() {
            let lg = &self.lugares[k];
            let (x, y) = (px + lg.x, py + lg.y);
            if y + lg.alto < PAGINA.y - 80.0 || y > AH {
                continue;
            }
            let tam = TAMS[p.tam];
            let base = y + lg.alto * 0.8;
            match p.c {
                Cont::Letra(c) => {
                    let (asc, _) = l.metricas(tam);
                    l.texto(&c.to_string(), x, base - asc, tam, p.color);
                }
                Cont::Sticker(o) => dibujos::dibujar(l, o, x + lg.w / 2.0, base - tam * 0.45, tam),
                Cont::Salto => {}
            }
        }
        // cursor parpadeante
        if self.abrir.is_none() && (cx.t * std::f32::consts::TAU).sin() > 0.0 {
            let (x, y, a) = self.lugar_cursor();
            let alto = TAMS[self.tam] * 1.05;
            let base = py + y + a * 0.8;
            l.rect(px + x, base - alto * 0.85, 4.0, alto, COLORES[self.color].2);
        }
        // barra de herramientas (encima de la página)
        l.rect(0.0, 0.0, AW, 72.0, rgb(240, 242, 247));
        l.rect(0.0, 72.0, AW, 2.0, rgb(190, 195, 210));
        for (r, b) in botones() {
            let sel = match b {
                Bot::Tam(t) => t == self.tam,
                Bot::Color(c) => c == self.color,
                Bot::Izq => !self.centro,
                Bot::Centro => self.centro,
                _ => false,
            };
            let sobre = r.dentro(cx.rl);
            if sel || sobre {
                l.rrect_r(r, 10.0, if sel { rgb(255, 236, 190) } else { rgb(222, 230, 248) });
            }
            if sel {
                l.rrect_borde(r, 10.0, 2.5, NARANJO);
            }
            let (x, y) = (r.cx(), r.cy());
            match b {
                Bot::Nuevo => {
                    l.rrect(x - 13.0, y - 22.0, 26.0, 32.0, 3.0, BLANCO);
                    l.rrect_borde(R::new(x - 13.0, y - 22.0, 26.0, 32.0), 3.0, 2.0, GRIS);
                    l.circulo(x + 10.0, y + 4.0, 9.0, VERDE);
                    l.rect(x + 5.0, y + 3.0, 10.0, 3.0, BLANCO);
                    l.rect(x + 8.5, y - 1.0, 3.0, 10.0, BLANCO);
                    l.texto_c("Nuevo", x, y + 20.0, 15.0, TINTA);
                }
                Bot::Abrir => {
                    l.rrect(x - 16.0, y - 18.0, 14.0, 8.0, 2.0, rgb(220, 160, 40));
                    l.rrect(x - 16.0, y - 14.0, 32.0, 24.0, 3.0, rgb(245, 190, 60));
                    l.texto_c("Abrir", x, y + 20.0, 15.0, TINTA);
                }
                Bot::Guardar => {
                    l.rrect(x - 15.0, y - 20.0, 30.0, 30.0, 4.0, AZUL);
                    l.rect(x - 9.0, y - 20.0, 18.0, 10.0, BLANCO);
                    l.rect(x - 9.0, y - 3.0, 18.0, 11.0, rgb(200, 220, 255));
                    l.texto_c("Guardar", x, y + 20.0, 15.0, TINTA);
                }
                Bot::Tam(t) => l.texto_c("A", x, y, [22.0, 32.0, 44.0][t], TINTA),
                Bot::Color(c) => l.circulo(x, y, 15.0, COLORES[c].2),
                Bot::Izq | Bot::Centro => {
                    for k in 0..4 {
                        let w = if k % 2 == 0 { 30.0 } else { 20.0 };
                        let lx = if b == Bot::Izq { x - 15.0 } else { x - w / 2.0 };
                        l.rect(lx, y - 13.0 + k as f32 * 8.0, w, 4.0, TINTA);
                    }
                }
                Bot::Leer => {
                    l.rrect_r(r, 27.0, AZUL);
                    l.altavoz(r.x + 28.0, y, 28.0, BLANCO);
                    l.texto("Leer", r.x + 50.0, y - 15.0, 24.0, BLANCO);
                }
            }
        }
        // stickers
        l.rect(856.0, 74.0, AW - 856.0, AH - 74.0, rgb(200, 206, 218));
        l.rrect(862.0, 78.0, 132.0, AH - 84.0, 14.0, rgb(232, 236, 245));
        for (i, o) in STICKERS.iter().enumerate() {
            let r = sticker_r(i);
            l.rrect_r(r, 10.0, if r.dentro(cx.rl) { rgb(255, 245, 210) } else { BLANCO });
            dibujos::dibujar(l, *o, r.cx(), r.cy(), 44.0);
        }
        if self.aviso > 0.0 {
            l.rrect(AW / 2.0 - 110.0, AH - 70.0, 220.0, 54.0, 27.0, VERDE);
            l.texto_c("Guardado", AW / 2.0, AH - 43.0, 28.0, BLANCO);
        }
        if let Some(lista) = &self.abrir {
            l.rect(0.0, 0.0, AW, AH, alfa(TINTA, 0.55));
            l.rrect(150.0, 40.0, 700.0, 440.0, 20.0, rgb(245, 246, 250));
            l.texto_c("Abrir documento", AW / 2.0, 76.0, 32.0, TINTA);
            for (i, (_, t)) in lista.iter().enumerate() {
                let r = R::new(170.0 + (i % 3) as f32 * 225.0, 110.0 + (i / 3) as f32 * 96.0, 210.0, 84.0);
                l.rrect_r(r, 12.0, if r.dentro(cx.rl) { rgb(222, 232, 255) } else { BLANCO });
                l.rrect_borde(r, 12.0, 2.0, rgb(200, 205, 220));
                l.rect(r.x + 12.0, r.y + 14.0, 30.0, 40.0, rgb(230, 235, 245));
                l.rect(r.x + 17.0, r.y + 22.0, 20.0, 3.0, GRIS);
                l.rect(r.x + 17.0, r.y + 30.0, 20.0, 3.0, GRIS);
                l.rect(r.x + 17.0, r.y + 38.0, 14.0, 3.0, GRIS);
                let t1: String = t.chars().take(11).collect();
                let t2: String = t.chars().skip(11).take(11).collect();
                l.texto(&t1, r.x + 52.0, r.y + 12.0, 20.0, TINTA);
                l.texto(&t2, r.x + 52.0, r.y + 40.0, 20.0, TINTA);
            }
        }
    }
}
