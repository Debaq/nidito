//! Escritorio: fondo con íconos, ventanas reales (mover, minimizar, maximizar, cerrar),
//! barra de tareas con Inicio, idioma y hora, menú Inicio y diálogo de apagar.

use crate::apps::{self, App, Cursor, Ev, Frases, AH, AW, T};
use crate::cx::{Cx, Idioma, Son};
use crate::dibujos::{self, Obj};
use crate::gfx::*;
use crate::config::Config;
use crate::padres::{Padres, RP};
use crate::tiempo::{llamar, Suceso, Tiempo, RT};
use crate::{H, W};
use chrono::{Datelike, Timelike};
use std::f32::consts::{FRAC_PI_2, PI, TAU as TAU_F};
use tiny_skia::{FilterQuality, Pixmap, PixmapPaint, Transform};

pub enum Accion {
    Salir(i32),
    Apagar,
    Reiniciar,
}

pub const BARRA_H: f32 = 44.0;
const BARRA_Y: f32 = H as f32 - BARRA_H;
const TIT_MAX: f32 = 38.0;
const TIT_NOR: f32 = 34.0;
const ESC_NOR: f32 = 0.6;

pub struct Info {
    pub titulo: &'static str,
    pub en: &'static str,
    pub color: C,
    pub ic: usize,
    pub juego: bool,
    pub crear: fn(&mut Cx) -> Box<dyn App>,
}

pub fn lista() -> Vec<Info> {
    vec![
        Info { titulo: "Pintar", en: "Paint", color: rgb(232, 74, 95), ic: 0, juego: false, crear: apps::pintar::Pintar::nuevo },
        Info { titulo: "Letras", en: "Letters", color: rgb(66, 135, 245), ic: 1, juego: false, crear: apps::letras::Letras::nuevo },
        Info { titulo: "Escribir", en: "Write", color: rgb(40, 110, 200), ic: 10, juego: false, crear: apps::escribir::Escribir::nuevo },
        Info { titulo: "Contar", en: "Count", color: rgb(255, 140, 50), ic: 2, juego: true, crear: apps::contar::Contar::nuevo },
        Info { titulo: "Mi clase", en: "My class", color: rgb(46, 160, 110), ic: 3, juego: false, crear: apps::clase::Clase::nuevo },
        Info { titulo: "Cocina", en: "Kitchen", color: rgb(230, 110, 160), ic: 4, juego: true, crear: apps::cocina::Cocina::nuevo },
        Info { titulo: "English", en: "English", color: rgb(70, 90, 200), ic: 5, juego: true, crear: apps::english::English::nuevo },
        Info { titulo: "Memorice", en: "Memory", color: rgb(240, 180, 30), ic: 6, juego: true, crear: apps::memorice::Memorice::nuevo },
        Info { titulo: "Globos", en: "Balloons", color: rgb(20, 180, 190), ic: 7, juego: true, crear: apps::globos::Globos::nuevo },
        Info { titulo: "Teclas", en: "Keys", color: rgb(150, 90, 220), ic: 8, juego: true, crear: apps::teclas::Teclas::nuevo },
        Info { titulo: "Mis dibujos", en: "My drawings", color: rgb(110, 120, 140), ic: 9, juego: false, crear: apps::galeria::Galeria::nuevo },
    ]
}

/// Ícono de cada app (dentro de un cuadrado de lado s centrado en x, y).
pub fn icono(l: &mut Lienzo, i: usize, x: f32, y: f32, s: f32, color: C) {
    let r = R::new(x - s / 2.0, y - s / 2.0, s, s);
    if i == 9 {
        // carpeta
        let c = rgb(245, 190, 60);
        l.rrect(r.x + s * 0.05, r.y + s * 0.12, s * 0.42, s * 0.2, s * 0.06, oscurecer(c, 0.1));
        l.rrect(r.x + s * 0.05, r.y + s * 0.2, s * 0.9, s * 0.68, s * 0.08, oscurecer(c, 0.1));
        l.rrect(r.x + s * 0.05, r.y + s * 0.3, s * 0.9, s * 0.58, s * 0.08, c);
        l.rect(r.x + s * 0.25, r.y + s * 0.22, s * 0.5, s * 0.14, BLANCO);
        return;
    }
    l.rrect_r(r.mover(0.0, s * 0.04), s * 0.24, oscurecer(color, 0.3));
    l.rrect_r(r, s * 0.24, color);
    l.rrect(r.x + s * 0.06, r.y + s * 0.04, s * 0.88, s * 0.4, s * 0.2, alfa(BLANCO, 0.15));
    let u = s / 100.0;
    match i {
        0 => {
            l.ovalo(x, y + 4.0 * u, 36.0 * u, 28.0 * u, rgb(250, 230, 190));
            for (k, c) in [ROJO, AMARILLO, AZUL, VERDE].iter().enumerate() {
                let a = -2.6 + k as f32 * 0.75;
                l.circulo(x + a.cos() * 22.0 * u, y + 4.0 * u + a.sin() * 15.0 * u, 7.0 * u, *c);
            }
            l.linea(x + 8.0 * u, y + 30.0 * u, x + 34.0 * u, y - 18.0 * u, 6.0 * u, CAFE);
        }
        1 => l.texto_c_borde("Aa", x, y, 56.0 * u, BLANCO, oscurecer(color, 0.4), 2.0 * u),
        2 => l.texto_c_borde("123", x, y, 46.0 * u, BLANCO, oscurecer(color, 0.4), 2.0 * u),
        3 => {
            l.rect(x - 34.0 * u, y - 26.0 * u, 68.0 * u, 44.0 * u, BLANCO);
            l.rect(x - 34.0 * u, y - 26.0 * u, 68.0 * u, 10.0 * u, AMARILLO);
            l.linea(x - 24.0 * u, y - 4.0 * u, x + 10.0 * u, y - 4.0 * u, 4.0 * u, TINTA);
            l.linea(x - 24.0 * u, y + 8.0 * u, x + 20.0 * u, y + 8.0 * u, 4.0 * u, TINTA);
            l.linea(x - 20.0 * u, y + 18.0 * u, x - 28.0 * u, y + 36.0 * u, 4.0 * u, CAFE);
            l.linea(x + 20.0 * u, y + 18.0 * u, x + 28.0 * u, y + 36.0 * u, 4.0 * u, CAFE);
        }
        4 => apps::cocina::dibujar_plato(l, apps::cocina::Plato::Panqueques, x, y - 4.0 * u, 80.0 * u),
        5 => {
            l.rrect(x - 34.0 * u, y - 26.0 * u, 68.0 * u, 42.0 * u, 12.0 * u, BLANCO);
            l.poligono(&[(x - 18.0 * u, y + 14.0 * u), (x - 24.0 * u, y + 30.0 * u), (x - 4.0 * u, y + 14.0 * u)], BLANCO);
            l.texto_c("Hi!", x, y - 5.0 * u, 34.0 * u, color);
        }
        6 => {
            l.rrect(x - 30.0 * u, y - 24.0 * u, 34.0 * u, 46.0 * u, 6.0 * u, MORADO);
            l.rrect(x - 4.0 * u, y - 20.0 * u, 34.0 * u, 46.0 * u, 6.0 * u, BLANCO);
            dibujos::dibujar(l, Obj::Corazon, x + 13.0 * u, y + 3.0 * u, 26.0 * u);
        }
        7 => {
            for (k, c) in [(-1.0, ROJO), (1.0, AMARILLO)] {
                let bx = x + k * 14.0 * u;
                l.linea(bx, y + 10.0 * u, x, y + 36.0 * u, 2.0 * u, BLANCO);
                l.ovalo(bx, y - 8.0 * u, 16.0 * u, 20.0 * u, c);
            }
        }
        10 => {
            l.rrect(x - 26.0 * u, y - 32.0 * u, 46.0 * u, 62.0 * u, 4.0 * u, BLANCO);
            for k in 0..4 {
                l.rect(x - 18.0 * u, y - 20.0 * u + k as f32 * 12.0 * u, 30.0 * u, 3.0 * u, rgb(150, 170, 210));
            }
            l.linea(x + 8.0 * u, y + 26.0 * u, x + 32.0 * u, y - 8.0 * u, 8.0 * u, AMARILLO);
            l.poligono(&[(x + 4.0 * u, y + 32.0 * u), (x + 5.0 * u, y + 22.0 * u), (x + 12.0 * u, y + 27.0 * u)], TINTA);
        }
        8 => {
            l.rrect(x - 36.0 * u, y - 18.0 * u, 72.0 * u, 38.0 * u, 6.0 * u, BLANCO);
            for f in 0..3 {
                for c in 0..5 {
                    l.rect(x - 31.0 * u + c as f32 * 13.0 * u, y - 13.0 * u + f as f32 * 11.0 * u, 10.0 * u, 8.0 * u, rgb(190, 180, 210));
                }
            }
        }
        _ => {}
    }
}

struct Ventana {
    app: Box<dyn App>,
    info: usize,
    pix: Pixmap,
    max: bool,
    min: bool,
    pos: (f32, f32),
    anim: f32,
    ultimo_render: f32,
    sucio: bool,
}

impl Ventana {
    fn marco(&self) -> R {
        if self.max {
            R::new(0.0, 0.0, W as f32, BARRA_Y)
        } else {
            R::new(self.pos.0, self.pos.1, AW * ESC_NOR + 12.0, AH * ESC_NOR + TIT_NOR + 6.0)
        }
    }
    fn titulo_r(&self) -> R {
        let m = self.marco();
        R::new(m.x, m.y, m.w, if self.max { TIT_MAX } else { TIT_NOR })
    }
    fn contenido(&self) -> (f32, f32, f32) {
        if self.max {
            ((W as f32 - AW) / 2.0, TIT_MAX, 1.0)
        } else {
            (self.pos.0 + 6.0, self.pos.1 + TIT_NOR, ESC_NOR)
        }
    }
    fn contenido_r(&self) -> R {
        let (x, y, e) = self.contenido();
        R::new(x, y, AW * e, AH * e)
    }
    fn local(&self, p: (f32, f32)) -> (f32, f32) {
        let (x, y, e) = self.contenido();
        ((p.0 - x) / e, (p.1 - y) / e)
    }
    /// Botones: 0 minimizar, 1 maximizar, 2 cerrar
    fn boton(&self, k: usize) -> R {
        let t = self.titulo_r();
        let (w, h) = (46.0, t.h - 8.0);
        R::new(t.der() - 6.0 - (3 - k) as f32 * (w + 4.0) + 4.0, t.y + 4.0, w, h)
    }
}

enum Dialogo {
    Apagar,
}

pub struct Escritorio {
    apps: Vec<Info>,
    fondo: Pixmap,
    ventanas: Vec<Ventana>,
    arrastre: Option<(usize, f32, f32)>,
    captura: Option<usize>,
    menu: bool,
    dialogo: Option<Dialogo>,
    padres: Option<Padres>,
    saludo: bool,
    ultimo_titulo: (f32, usize),
    reloj: (String, String),
    reloj_t: f32,
    sucio: bool,
    refresco: f32,
    pub tiempo: Tiempo,
}

pub fn icono_r(i: usize) -> R {
    let (c, f) = (i / 4, i % 4);
    R::new(14.0 + c as f32 * 112.0, 12.0 + f as f32 * 132.0, 108.0, 124.0)
}

const INICIO: R = R::new(6.0, BARRA_Y + 5.0, 118.0, BARRA_H - 10.0);
const IDIOMA: R = R::new(W as f32 - 236.0, BARRA_Y + 5.0, 140.0, BARRA_H - 10.0);
const MENU: R = R::new(6.0, BARRA_Y - 496.0, 360.0, 490.0);

pub fn menu_item(i: usize) -> R {
    R::new(MENU.x + 12.0 + (i % 2) as f32 * 170.0, MENU.y + 80.0 + (i / 2) as f32 * 56.0, 162.0, 52.0)
}
pub const MENU_APAGAR: R = R::new(MENU.x + 12.0, MENU.y + MENU.h - 52.0, MENU.w - 24.0, 42.0);
const DIALOGO: R = R::new(272.0, 170.0, 480.0, 230.0);
const DLG_SI: R = R::new(DIALOGO.x + 40.0, DIALOGO.y + 150.0, 190.0, 56.0);
pub const DLG_NO: R = R::new(DIALOGO.x + 250.0, DIALOGO.y + 150.0, 190.0, 56.0);

fn tarea_r(k: usize) -> R {
    R::new(132.0 + k as f32 * 150.0, BARRA_Y + 5.0, 144.0, BARRA_H - 10.0)
}

fn saludo(nombre: &str) -> (String, String) {
    if nombre.trim().is_empty() {
        ("¡Hola! ¿Qué quieres hacer hoy?".into(), "Hello! What do you want to do today?".into())
    } else {
        (format!("¡Hola, {}! ¿Qué quieres hacer hoy?", nombre.trim()), "Hello! What do you want to do today?".into())
    }
}

pub fn frases(f: &mut Frases, llamado: &str) {
    let (a, b) = saludo(llamado);
    f.bil(a, b);
    for i in lista() {
        f.bil(i.titulo, i.en);
    }
    f.es("Español");
    f.en("English");
    f.es("Español e inglés");
    f.en("Spanish and English");
    f.bil("¿Quieres apagar el computador?", "Do you want to turn off the computer?");
    f.bil("¡Chao! Hasta pronto.", "Bye bye!");
    f.bil("Inicio", "Start");
}

const DIAS: [&str; 7] = ["lunes", "martes", "miércoles", "jueves", "viernes", "sábado", "domingo"];
const MESES: [&str; 12] =
    ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"];

fn hora() -> (String, String) {
    let n = chrono::Local::now();
    (
        format!("{:02}:{:02}", n.hour(), n.minute()),
        format!("{} {} de {}", DIAS[n.weekday().num_days_from_monday() as usize], n.day(), MESES[n.month0() as usize]),
    )
}

impl Escritorio {
    pub fn nuevo(l: &mut Lienzo, cfg: &Config) -> Escritorio {
        let apps = lista();
        let mut fondo = Pixmap::new(W, H).unwrap();
        l.con(&mut fondo, |l| {
            let (w, h) = (W as f32, H as f32);
            l.degradado_v(0.0, 0.0, w, h, rgb(110, 190, 240), rgb(200, 235, 255));
            // sol
            for k in 0..12 {
                let a = k as f32 * PI / 6.0;
                l.linea(860.0 + a.cos() * 62.0, 250.0 + a.sin() * 62.0, 860.0 + a.cos() * 82.0, 250.0 + a.sin() * 82.0, 8.0, rgb(255, 220, 90));
            }
            l.circulo(860.0, 250.0, 50.0, rgb(255, 225, 90));
            // nubes
            for &(x, y, s) in &[(420.0, 70.0, 1.0), (640.0, 150.0, 0.8), (520.0, 260.0, 0.6)] {
                for &(dx, dy, r) in &[(0.0, 0.0, 30.0), (34.0, -14.0, 38.0), (70.0, 0.0, 30.0), (34.0, 8.0, 28.0)] {
                    l.circulo(x + dx * s, y + dy * s, r * s, alfa(BLANCO, 0.9));
                }
            }
            // cerros
            l.ovalo(300.0, h + 40.0, 520.0, 200.0, rgb(120, 200, 110));
            l.ovalo(850.0, h + 60.0, 420.0, 230.0, rgb(90, 180, 95));
            for &(x, y) in &[(560.0, 470.0), (640.0, 500.0), (720.0, 455.0), (800.0, 420.0), (470.0, 500.0), (900.0, 440.0)] {
                dibujos::dibujar(l, Obj::Flor, x, y, 34.0);
            }
            dibujos::dibujar(l, Obj::Arbol, 960.0, 400.0, 150.0);
            dibujos::dibujar(l, Obj::Casa, 660.0, 380.0, 120.0);
            // íconos fijos (solo el que está bajo el ratón se redibuja)
            for (i, info) in apps.iter().enumerate() {
                let r = icono_r(i);
                icono(l, info.ic, r.cx(), r.y + 48.0, 76.0, info.color);
                l.texto_c_borde(info.titulo, r.cx(), r.y + 106.0, 19.0, BLANCO, alfa(TINTA, 0.8), 1.5);
            }
        });
        Escritorio {
            apps,
            fondo,
            ventanas: vec![],
            arrastre: None,
            captura: None,
            menu: false,
            dialogo: None,
            padres: None,
            saludo: false,
            ultimo_titulo: (-10.0, usize::MAX),
            reloj: hora(),
            reloj_t: 0.0,
            sucio: true,
            refresco: 0.0,
            tiempo: Tiempo::nuevo(cfg),
        }
    }

    fn foco(&self) -> Option<usize> {
        self.ventanas.iter().rposition(|v| !v.min)
    }

    fn al_frente(&mut self, i: usize) -> usize {
        let v = self.ventanas.remove(i);
        self.ventanas.push(v);
        self.ventanas.len() - 1
    }

    pub fn abrir(&mut self, i: usize, cx: &mut Cx) {
        if self.apps[i].juego && self.tiempo.juegos_agotados() {
            cx.sonar(Son::Boop);
            cx.decir_bil("Los juegos están durmiendo. Mañana vuelven.", "The games are sleeping. They come back tomorrow.");
            return;
        }
        cx.sonar(Son::Whoosh);
        if let Some(k) = self.ventanas.iter().position(|v| v.info == i) {
            let k = self.al_frente(k);
            self.ventanas[k].min = false;
            return;
        }
        let info = &self.apps[i];
        cx.decir_bil(info.titulo, info.en);
        let app = (info.crear)(cx);
        let n = self.ventanas.len() as f32;
        self.ventanas.push(Ventana {
            app,
            info: i,
            pix: Pixmap::new(AW as u32, AH as u32).unwrap(),
            max: true,
            min: false,
            pos: (40.0 + n * 34.0, 24.0 + n * 26.0),
            anim: 0.0,
            ultimo_render: -1.0,
            sucio: true,
        });
    }

    fn cerrar(&mut self, k: usize, cx: &mut Cx) {
        let mut v = self.ventanas.remove(k);
        v.app.cerrar(cx);
        cx.audio.callar();
        cx.sonar(Son::Whoosh);
        self.captura = None;
        self.arrastre = None;
    }

    /// Cierra todo guardando (antes de apagar o salir).
    pub fn cerrar_todo(&mut self, cx: &mut Cx) {
        while !self.ventanas.is_empty() {
            let mut v = self.ventanas.remove(0);
            v.app.cerrar(cx);
        }
    }

    fn evento_app(&mut self, k: usize, ev: Ev, cx: &mut Cx) {
        let v = &mut self.ventanas[k];
        let ev = match ev {
            Ev::Clic(x, y) => {
                let p = v.local((x, y));
                Ev::Clic(p.0, p.1)
            }
            Ev::Mueve(x, y) => {
                let p = v.local((x, y));
                Ev::Mueve(p.0, p.1)
            }
            Ev::Suelta(x, y) => {
                let p = v.local((x, y));
                Ev::Suelta(p.0, p.1)
            }
            e => e,
        };
        v.app.evento(&ev, cx);
        v.sucio = true;
        if let Ev::Mueve(x, y) = ev {
            cx.rl = (x, y);
        }
    }

    /// ¿Hay que dibujar este cuadro? (si nada cambió, el equipo descansa)
    pub fn necesita_cuadro(&mut self, dt: f32) -> bool {
        self.refresco += dt;
        let foco_anim = self.foco().is_some_and(|k| self.ventanas[k].app.animando());
        let anim = self.ventanas.iter().any(|v| v.anim < 1.0) || self.padres.is_some();
        if self.sucio || foco_anim || anim || self.refresco >= 0.25 {
            self.sucio = false;
            self.refresco = 0.0;
            true
        } else {
            false
        }
    }

    pub fn evento(&mut self, ev: Ev, cx: &mut Cx) -> Option<Accion> {
        self.sucio = true;
        self.tiempo.registrar_input(cx.t);
        if let Some(p) = &mut self.padres {
            match p.evento(&ev, cx) {
                RP::Seguir => {}
                RP::Cerrar => self.padres = None,
                RP::MasTiempo => {
                    self.tiempo.agregar(10.0);
                    self.padres = None;
                }
                RP::Bloquear => {
                    self.cerrar_todo(cx);
                    self.menu = false;
                    self.dialogo = None;
                    self.tiempo.bloquear();
                    self.tiempo.saludar(cx);
                    self.padres = None;
                }
                RP::Accion(a) => {
                    if !matches!(a, Accion::Salir(_)) {
                        self.cerrar_todo(cx);
                    }
                    return Some(a);
                }
            }
            return None;
        }
        if self.tiempo.bloqueado {
            match self.tiempo.evento(&ev, cx) {
                Some(RT::Desbloqueado) => {
                    let (a, b) = saludo(&llamar(cx));
                    cx.decir_bil(&a, &b);
                }
                Some(RT::Apagar) => {
                    cx.decir_bil("¡Chao! Hasta pronto.", "Bye bye!");
                    return Some(Accion::Apagar);
                }
                None => {}
            }
            return None;
        }
        if let Some(Dialogo::Apagar) = self.dialogo {
            if let Ev::Clic(x, y) = ev {
                if DLG_SI.dentro((x, y)) {
                    self.cerrar_todo(cx);
                    cx.decir_bil("¡Chao! Hasta pronto.", "Bye bye!");
                    return Some(Accion::Apagar);
                }
                if DLG_NO.dentro((x, y)) {
                    self.dialogo = None;
                    cx.sonar(Son::Clic);
                }
            }
            return None;
        }
        match ev {
            Ev::Clic(x, y) => self.clic((x, y), cx),
            Ev::Mueve(x, y) => {
                if let Some((k, dx, dy)) = self.arrastre {
                    let v = &mut self.ventanas[k];
                    v.pos.0 = (x - dx).clamp(-400.0, W as f32 - 120.0);
                    v.pos.1 = (y - dy).clamp(0.0, BARRA_Y - TIT_NOR);
                } else if let Some(k) = self.captura.or(self.foco()) {
                    if k < self.ventanas.len() {
                        self.evento_app(k, Ev::Mueve(x, y), cx);
                    }
                }
            }
            Ev::Suelta(x, y) => {
                self.arrastre = None;
                if let Some(k) = self.captura.take() {
                    if k < self.ventanas.len() {
                        self.evento_app(k, Ev::Suelta(x, y), cx);
                    }
                }
            }
            e @ (Ev::Texto(_) | Ev::Tecla(_)) => {
                if matches!(e, Ev::Tecla(T::Escape)) && self.menu {
                    self.menu = false;
                    return None;
                }
                if let Some(k) = self.foco() {
                    self.evento_app(k, e, cx);
                }
            }
        }
        None
    }

    fn clic(&mut self, p: (f32, f32), cx: &mut Cx) {
        // menú Inicio
        if self.menu {
            self.menu = false;
            if MENU.dentro(p) {
                for i in 0..self.apps.len() {
                    if menu_item(i).dentro(p) {
                        self.abrir(i, cx);
                        return;
                    }
                }
                if MENU_APAGAR.dentro(p) {
                    self.dialogo = Some(Dialogo::Apagar);
                    cx.sonar(Son::Clic);
                    cx.decir_bil("¿Quieres apagar el computador?", "Do you want to turn off the computer?");
                }
                return;
            }
            if INICIO.dentro(p) {
                cx.sonar(Son::Clic);
                return;
            }
        }
        // barra de tareas
        if p.1 >= BARRA_Y {
            if INICIO.dentro(p) {
                self.menu = true;
                cx.sonar(Son::Clic);
                cx.decir_bil("Inicio", "Start");
                return;
            }
            if IDIOMA.dentro(p) {
                cx.idioma = match cx.idioma {
                    Idioma::Es => Idioma::En,
                    Idioma::En => Idioma::Ambos,
                    Idioma::Ambos => Idioma::Es,
                };
                cx.sonar(Son::Clic);
                match cx.idioma {
                    Idioma::Es => cx.decir("Español"),
                    Idioma::En => cx.decir_en("English"),
                    Idioma::Ambos => cx.decir_es_en("Español e inglés", "Spanish and English"),
                }
                return;
            }
            let orden: Vec<usize> = self.orden_tareas();
            for (k, &vi) in orden.iter().enumerate() {
                if tarea_r(k).dentro(p) {
                    cx.sonar(Son::Clic);
                    if Some(vi) == self.foco() {
                        self.ventanas[vi].min = true;
                    } else {
                        let j = self.al_frente(vi);
                        self.ventanas[j].min = false;
                    }
                    return;
                }
            }
            return;
        }
        // ventanas, de arriba hacia abajo
        for k in (0..self.ventanas.len()).rev() {
            let v = &self.ventanas[k];
            if v.min || !v.marco().dentro(p) {
                continue;
            }
            let k = if k != self.ventanas.len() - 1 { self.al_frente(k) } else { k };
            let v = &mut self.ventanas[k];
            if v.titulo_r().dentro(p) {
                if v.boton(2).dentro(p) {
                    self.cerrar(k, cx);
                } else if v.boton(1).dentro(p) {
                    v.max = !v.max;
                    cx.sonar(Son::Clic);
                } else if v.boton(0).dentro(p) {
                    v.min = true;
                    cx.sonar(Son::Clic);
                } else {
                    // doble clic en el título: maximizar/restaurar
                    if self.ultimo_titulo.1 == v.info && cx.t - self.ultimo_titulo.0 < 0.4 {
                        v.max = !v.max;
                        self.ultimo_titulo = (-10.0, usize::MAX);
                    } else {
                        self.ultimo_titulo = (cx.t, v.info);
                        if !v.max {
                            self.arrastre = Some((k, p.0 - v.pos.0, p.1 - v.pos.1));
                        }
                    }
                }
                return;
            }
            if v.contenido_r().dentro(p) {
                self.captura = Some(k);
                self.evento_app(k, Ev::Clic(p.0, p.1), cx);
            }
            return;
        }
        // íconos del escritorio
        for i in 0..self.apps.len() {
            if icono_r(i).dentro(p) {
                self.abrir(i, cx);
                return;
            }
        }
    }

    /// Ventanas en el orden en que fueron abiertas (para la barra).
    fn orden_tareas(&self) -> Vec<usize> {
        let mut v: Vec<usize> = (0..self.ventanas.len()).collect();
        v.sort_by_key(|&k| self.ventanas[k].info);
        v
    }

    pub fn abrir_padres(&mut self, cx: &mut Cx) {
        self.padres = Some(Padres::nuevo(cx, self.tiempo.verificador()));
        self.menu = false;
    }

    pub fn actualizar(&mut self, dt: f32, cx: &mut Cx) {
        if !self.saludo {
            self.saludo = true;
            if self.tiempo.bloqueado {
                self.tiempo.saludar(cx);
            } else {
                let (a, b) = saludo(&llamar(cx));
                cx.decir_bil(&a, &b);
            }
        }
        let en_juego = self.foco().is_some_and(|k| self.apps[self.ventanas[k].info].juego);
        match self.tiempo.tick(dt, cx, en_juego) {
            Some(Suceso::JuegosAgotados) => {
                let mut k = 0;
                while k < self.ventanas.len() {
                    if self.apps[self.ventanas[k].info].juego {
                        let mut v = self.ventanas.remove(k);
                        v.app.cerrar(cx);
                    } else {
                        k += 1;
                    }
                }
                self.captura = None;
                self.arrastre = None;
            }
            Some(Suceso::SesionTerminada) => {
                self.cerrar_todo(cx);
                self.menu = false;
                self.dialogo = None;
                self.captura = None;
                self.arrastre = None;
            }
            None => {}
        }
        self.reloj_t -= dt;
        if self.reloj_t <= 0.0 {
            self.reloj = hora();
            self.reloj_t = 5.0;
        }
        let foco = self.foco();
        if let Some(k) = foco {
            let p = self.ventanas[k].local(cx.raton);
            cx.rl = p;
        } else {
            cx.rl = (-100.0, -100.0);
        }
        for (k, v) in self.ventanas.iter_mut().enumerate() {
            if v.anim < 1.0 {
                v.anim = (v.anim + dt / 0.22).min(1.0);
            }
            if !v.min || Some(k) == foco {
                v.app.actualizar(dt, cx);
            }
        }
        if let Some(p) = &mut self.padres {
            p.actualizar(dt);
        }
    }

    pub fn dibujar(&mut self, l: &mut Lienzo, cx: &Cx) {
        if self.tiempo.bloqueado {
            self.tiempo.dibujar(l, cx);
            if let Some(p) = &self.padres {
                p.dibujar(l, cx);
            }
            return;
        }
        let foco = self.foco();
        // desde la ventana maximizada más alta (lo de abajo no se ve)
        let desde = self.ventanas.iter().rposition(|v| !v.min && v.max && v.anim >= 1.0);
        if desde.is_none() {
            copiar_opaco(&mut l.pix, &self.fondo, 0, 0);
            self.dibujar_widgets(l, cx);
        }
        let inicio = desde.unwrap_or(0);
        for k in inicio..self.ventanas.len() {
            if self.ventanas[k].min {
                continue;
            }
            let activa = Some(k) == foco;
            let v = &mut self.ventanas[k];
            // contenido: la ventana activa a tiempo completo, las demás de vez en cuando
            let toca = if activa { v.sucio || v.app.animando() || cx.t - v.ultimo_render > 0.24 } else { cx.t - v.ultimo_render > 0.3 };
            if toca {
                v.sucio = false;
                let mut pix = std::mem::replace(&mut v.pix, Pixmap::new(1, 1).unwrap());
                l.con(&mut pix, |l| v.app.dibujar(l, cx));
                v.pix = pix;
                v.ultimo_render = cx.t;
            }
            dibujar_ventana(l, v, &self.apps[v.info], activa, cx);
        }
        self.dibujar_barra(l, cx);
        if self.menu {
            self.dibujar_menu(l, cx);
        }
        if let Some(Dialogo::Apagar) = self.dialogo {
            l.rect(0.0, 0.0, W as f32, H as f32, alfa(NEGRO, 0.45));
            let d = DIALOGO;
            l.rrect_r(d.mover(0.0, 8.0), 18.0, alfa(NEGRO, 0.3));
            l.rrect_r(d, 18.0, rgb(245, 246, 250));
            l.rrect(d.x, d.y, d.w, 46.0, 18.0, rgb(60, 70, 95));
            l.rect(d.x, d.y + 28.0, d.w, 18.0, rgb(60, 70, 95));
            l.texto("Apagar", d.x + 20.0, d.y + 8.0, 26.0, BLANCO);
            icono_apagar(l, d.x + 60.0, d.y + 98.0, 26.0, ROJO);
            l.texto("¿Quieres apagar", d.x + 105.0, d.y + 62.0, 30.0, TINTA);
            l.texto("el computador?", d.x + 105.0, d.y + 96.0, 30.0, TINTA);
            boton_r(l, DLG_SI, "Sí, apagar", ROJO, cx);
            boton_r(l, DLG_NO, "No", VERDE, cx);
        }
        if let Some(p) = &self.padres {
            p.dibujar(l, cx);
        }
    }

    fn dibujar_widgets(&mut self, l: &mut Lienzo, cx: &Cx) {
        // reloj grande y saludo, como un widget de escritorio
        let (h, f) = self.reloj.clone();
        l.rrect(W as f32 - 330.0, 18.0, 312.0, 150.0, 24.0, alfa(BLANCO, 0.55));
        l.texto_c(&h, W as f32 - 174.0, 64.0, 64.0, TINTA);
        l.texto_c(&f, W as f32 - 174.0, 112.0, 24.0, rgb(70, 80, 100));
        let n = llamar(cx);
        let hola = if n.is_empty() { "¡Hola!".to_string() } else { format!("¡Hola, {n}!") };
        l.texto_c(&hola, W as f32 - 174.0, 146.0, 26.0, rgb(230, 80, 120));
        let dormidos = self.tiempo.juegos_agotados();
        for (i, info) in self.apps.iter().enumerate() {
            let r = icono_r(i);
            if dormidos && info.juego {
                l.rrect(r.cx() - 40.0, r.y + 8.0, 80.0, 80.0, 18.0, alfa(rgb(40, 44, 92), 0.6));
                l.circulo(r.cx() + 18.0, r.y + 26.0, 14.0, rgb(255, 240, 180));
                l.circulo(r.cx() + 25.0, r.y + 21.0, 12.0, rgb(70, 74, 120));
                continue;
            }
            if r.dentro(cx.raton) && !self.menu {
                l.rrect_r(r, 14.0, alfa(BLANCO, 0.35));
                l.rrect_borde(r, 14.0, 2.0, alfa(BLANCO, 0.8));
                icono(l, info.ic, r.cx(), r.y + 48.0, 82.0, info.color);
                l.texto_c_borde(info.titulo, r.cx(), r.y + 106.0, 19.0, BLANCO, alfa(TINTA, 0.8), 1.5);
            }
        }
    }

    fn dibujar_barra(&mut self, l: &mut Lienzo, cx: &Cx) {
        let y = BARRA_Y;
        l.rect(0.0, y, W as f32, BARRA_H, rgb(32, 38, 56));
        l.rect(0.0, y, W as f32, 2.0, rgb(70, 80, 110));
        // Inicio
        let sobre = INICIO.dentro(cx.raton);
        let c = if self.menu { rgb(60, 190, 110) } else if sobre { rgb(70, 200, 120) } else { rgb(46, 170, 96) };
        l.rrect_r(INICIO, 10.0, c);
        let (ix, iy) = (INICIO.x + 22.0, INICIO.cy());
        l.poligono(&[(ix - 13.0, iy - 1.0), (ix, iy - 13.0), (ix + 13.0, iy - 1.0)], BLANCO);
        l.rect(ix - 9.0, iy - 2.0, 18.0, 13.0, BLANCO);
        l.rect(ix - 3.0, iy + 3.0, 6.0, 8.0, c);
        l.texto("Inicio", INICIO.x + 42.0, INICIO.y + 2.0, 24.0, BLANCO);
        // ventanas abiertas
        let foco = self.foco();
        for (k, &vi) in self.orden_tareas().iter().enumerate() {
            let r = tarea_r(k);
            let v = &self.ventanas[vi];
            let activa = Some(vi) == foco;
            let fondo = if activa { rgb(80, 95, 135) } else if r.dentro(cx.raton) { rgb(60, 70, 100) } else { rgb(45, 52, 75) };
            l.rrect_r(r, 8.0, fondo);
            if activa {
                l.rect(r.x + 10.0, r.abajo() - 3.0, r.w - 20.0, 3.0, rgb(120, 200, 255));
            }
            let info = &self.apps[v.info];
            icono(l, info.ic, r.x + 20.0, r.cy(), 24.0, info.color);
            let t = if info.titulo.len() > 10 { 16.0 } else { 19.0 };
            l.texto(info.titulo, r.x + 40.0, r.y + 6.0, t, if v.min { rgb(170, 175, 195) } else { BLANCO });
        }
        // idioma
        let fondo = if IDIOMA.dentro(cx.raton) { rgb(60, 70, 100) } else { rgb(45, 52, 75) };
        l.rrect_r(IDIOMA, 8.0, fondo);
        let (fx, fy) = (IDIOMA.x + 10.0, IDIOMA.y + 8.0);
        match cx.idioma {
            Idioma::Es => {
                bandera_cl(l, fx, fy, 30.0);
                l.texto("Español", fx + 38.0, IDIOMA.y + 5.0, 20.0, BLANCO);
            }
            Idioma::En => {
                bandera_us(l, fx, fy, 30.0);
                l.texto("English", fx + 38.0, IDIOMA.y + 5.0, 20.0, BLANCO);
            }
            Idioma::Ambos => {
                bandera_cl(l, fx, fy, 30.0);
                bandera_us(l, fx + 36.0, fy, 30.0);
                l.texto("Los dos", fx + 74.0, IDIOMA.y + 5.0, 18.0, BLANCO);
            }
        }
        // tiempo restante (círculo que se vacía)
        let en_juego = foco.is_some_and(|k| self.apps[self.ventanas[k].info].juego);
        if let Some((f, juego)) = self.tiempo.restante(en_juego) {
            let (px, py, pr) = (IDIOMA.x - 30.0, y + BARRA_H / 2.0, 15.0);
            l.circulo(px, py, pr + 2.0, rgb(70, 80, 110));
            let c = if f < 0.15 { ROJO } else if juego { NARANJO } else { VERDE };
            let mut pts = vec![(px, py)];
            let n = (f * 32.0).ceil() as usize;
            for k in 0..=n {
                let a = -FRAC_PI_2 + TAU_F * f * k as f32 / n.max(1) as f32;
                pts.push((px + a.cos() * pr, py + a.sin() * pr));
            }
            if f > 0.0 {
                l.poligono(&pts, c);
            }
        }
        // hora
        let h = self.reloj.0.clone();
        l.texto_c(&h, W as f32 - 46.0, y + BARRA_H / 2.0, 24.0, BLANCO);
    }

    fn dibujar_menu(&mut self, l: &mut Lienzo, cx: &Cx) {
        let m = MENU;
        l.rrect_r(m.mover(0.0, 6.0), 16.0, alfa(NEGRO, 0.3));
        l.rrect_r(m, 16.0, rgb(245, 247, 252));
        l.rrect(m.x, m.y, m.w, 70.0, 16.0, rgb(46, 170, 96));
        l.rect(m.x, m.y + 50.0, m.w, 20.0, rgb(46, 170, 96));
        let nombre = if cx.nombre.trim().is_empty() { "Nidito" } else { cx.nombre.trim() };
        l.circulo(m.x + 40.0, m.y + 35.0, 24.0, BLANCO);
        let ini: String = nombre.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
        l.texto_c(&ini, m.x + 40.0, m.y + 35.0, 30.0, rgb(46, 170, 96));
        l.texto(nombre, m.x + 76.0, m.y + 14.0, 32.0, BLANCO);
        for (i, info) in self.apps.iter().enumerate() {
            let r = menu_item(i);
            if r.dentro(cx.raton) {
                l.rrect_r(r, 10.0, rgb(220, 232, 250));
            }
            icono(l, info.ic, r.x + 28.0, r.cy(), 40.0, info.color);
            let t = if info.titulo.len() > 9 { 19.0 } else { 23.0 };
            l.texto(info.titulo, r.x + 56.0, r.cy() - l.alto(t) / 2.0, t, TINTA);
        }
        let r = MENU_APAGAR;
        l.rrect_r(r, 10.0, if r.dentro(cx.raton) { rgb(255, 215, 215) } else { rgb(250, 232, 232) });
        icono_apagar(l, r.x + 26.0, r.cy(), 13.0, ROJO);
        l.texto("Apagar", r.x + 50.0, r.y + 6.0, 24.0, rgb(180, 40, 50));
    }

    /// Dibuja el cursor del ratón (encima de todo).
    pub fn dibujar_cursor(&self, l: &mut Lienzo, cx: &Cx) {
        let (x, y) = cx.raton;
        let mut tipo = Cursor::Normal;
        if self.padres.is_none() && self.dialogo.is_none() && !self.menu {
            if let Some(k) = self.foco() {
                let v = &self.ventanas[k];
                if v.contenido_r().dentro((x, y)) {
                    tipo = v.app.cursor();
                }
            }
        }
        match tipo {
            Cursor::Ninguno => {}
            Cursor::Laser => {
                l.circulo(x, y, 18.0, alfa(ROJO, 0.25));
                l.circulo(x, y, 10.0, alfa(ROJO, 0.5));
                l.circulo(x, y, 6.0, rgb(255, 60, 60));
                l.circulo(x - 2.0, y - 2.0, 2.0, BLANCO);
            }
            Cursor::Cuchara => {
                l.linea(x + 10.0, y + 14.0, x + 52.0, y + 56.0, 9.0, rgb(160, 110, 60));
                l.ovalo(x + 2.0, y + 4.0, 16.0, 20.0, rgb(180, 125, 70));
                l.ovalo(x + 2.0, y + 2.0, 10.0, 13.0, rgb(200, 150, 95));
            }
            Cursor::Normal => {
                let s = 1.5;
                let pts: Vec<(f32, f32)> =
                    [(0.0, 0.0), (0.0, 17.0), (4.2, 13.2), (7.2, 20.0), (10.0, 18.8), (7.2, 12.4), (12.6, 12.4)].iter().map(|(a, b)| (x + a * s, y + b * s)).collect();
                l.delinear(ruta_poligono(&pts), 3.5, TINTA);
                l.poligono(&pts, BLANCO);
            }
        }
    }
}

fn boton_r(l: &mut Lienzo, r: R, t: &str, c: C, cx: &Cx) {
    let r2 = if r.dentro(cx.raton) { r.crecer(3.0) } else { r };
    l.rrect_r(r2.mover(0.0, 4.0), 14.0, oscurecer(c, 0.3));
    l.rrect_r(r2, 14.0, c);
    l.texto_c(t, r2.cx(), r2.cy(), 28.0, BLANCO);
}

fn icono_apagar(l: &mut Lienzo, x: f32, y: f32, r: f32, c: C) {
    let pts: Vec<(f32, f32)> = (0..=20)
        .map(|i| {
            let a = -FRAC_PI_2 + 0.6 + (2.0 * PI - 1.2) * i as f32 / 20.0;
            (x + a.cos() * r, y + a.sin() * r)
        })
        .collect();
    l.polilinea(&pts, r * 0.28, c);
    l.linea(x, y - r * 1.15, x, y - r * 0.2, r * 0.28, c);
}

fn bandera_cl(l: &mut Lienzo, x: f32, y: f32, w: f32) {
    let h = w * 0.66;
    l.rect(x, y, w, h / 2.0, BLANCO);
    l.rect(x, y + h / 2.0, w, h / 2.0, rgb(215, 40, 50));
    l.rect(x, y, w / 3.0, h / 2.0, rgb(0, 57, 166));
    l.estrella(x + w / 6.0, y + h / 4.0, h * 0.16, -FRAC_PI_2, BLANCO);
}

fn bandera_us(l: &mut Lienzo, x: f32, y: f32, w: f32) {
    let h = w * 0.66;
    for k in 0..7 {
        l.rect(x, y + k as f32 * h / 7.0, w, h / 7.0, if k % 2 == 0 { rgb(190, 30, 45) } else { BLANCO });
    }
    l.rect(x, y, w * 0.45, h * 0.55, rgb(40, 50, 120));
    for f in 0..2 {
        for c in 0..3 {
            l.circulo(x + 3.0 + c as f32 * 4.5, y + 3.5 + f as f32 * 5.5, 1.1, BLANCO);
        }
    }
}

fn dibujar_ventana(l: &mut Lienzo, v: &Ventana, info: &Info, activa: bool, cx: &Cx) {
    let color = if activa { info.color } else { mezclar(info.color, rgb(150, 155, 170), 0.55) };
    if v.anim < 1.0 {
        let desde = R::new(W as f32 / 2.0 - 60.0, BARRA_Y / 2.0 - 40.0, 120.0, 80.0);
        let r = R::lerp(desde, v.marco(), ease_out(v.anim));
        l.rrect_r(r, 12.0, color);
        l.rrect_r(r.crecer(-8.0), 8.0, rgb(250, 250, 252));
        return;
    }
    let m = v.marco();
    let t = v.titulo_r();
    let radio = if v.max { 0.0 } else { 12.0 };
    if !v.max {
        l.rrect_r(m.mover(4.0, 8.0), radio, alfa(NEGRO, 0.25));
    }
    l.rrect_r(m, radio, color);
    // contenido
    let (x, y, e) = v.contenido();
    if e == 1.0 {
        copiar_opaco(&mut l.pix, &v.pix, x as i32, y as i32);
    } else {
        let pp = PixmapPaint { quality: FilterQuality::Bilinear, ..Default::default() };
        l.pix.draw_pixmap(0, 0, v.pix.as_ref(), &pp, Transform::from_row(e, 0.0, 0.0, e, x, y), None);
    }
    // título
    let isz = t.h - 10.0;
    icono(l, info.ic, t.x + 12.0 + isz / 2.0, t.cy(), isz, oscurecer(info.color, 0.15));
    l.texto(info.titulo, t.x + 22.0 + isz, t.y + (t.h - l.alto(t.h * 0.62)) / 2.0, t.h * 0.62, BLANCO);
    for k in 0..3 {
        let b = v.boton(k);
        let sobre = b.dentro(cx.raton);
        let fondo = match (k, sobre) {
            (2, true) => rgb(235, 60, 60),
            (2, false) => rgb(215, 75, 75),
            (_, true) => alfa(BLANCO, 0.45),
            _ => alfa(BLANCO, 0.25),
        };
        l.rrect_r(b, 8.0, fondo);
        let (bx, by) = (b.cx(), b.cy());
        let u = b.h * 0.22;
        match k {
            0 => l.linea(bx - u, by + u * 0.8, bx + u, by + u * 0.8, 3.0, BLANCO),
            1 => {
                if v.max {
                    l.rrect_borde(R::new(bx - u * 0.6, by - u * 1.1, u * 1.7, u * 1.5), 2.0, 2.5, BLANCO);
                    l.rrect_r(R::new(bx - u * 1.1, by - u * 0.5, u * 1.7, u * 1.5), 2.0, if sobre { mezclar(color, BLANCO, 0.45) } else { mezclar(color, BLANCO, 0.25) });
                    l.rrect_borde(R::new(bx - u * 1.1, by - u * 0.5, u * 1.7, u * 1.5), 2.0, 2.5, BLANCO);
                } else {
                    l.rrect_borde(R::new(bx - u, by - u, u * 2.0, u * 2.0), 2.0, 3.0, BLANCO);
                }
            }
            _ => {
                l.linea(bx - u, by - u, bx + u, by + u, 3.5, BLANCO);
                l.linea(bx + u, by - u, bx - u, by + u, 3.5, BLANCO);
            }
        }
    }
}
