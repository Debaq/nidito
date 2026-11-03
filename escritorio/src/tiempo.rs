//! Tiempo de uso: pantalla de bloqueo con clave de adultos, sesión con límite elegido,
//! tope diario para juegos y avisos suaves. Solo cuenta el tiempo activo.

use crate::apps::{Ev, Frases, T};
use crate::config::Config;
use crate::cx::{Cx, Son};
use crate::dibujos::{self, Obj};
use crate::gfx::*;
use crate::{H, W};
use sha2::{Digest, Sha256};
use std::f32::consts::{FRAC_PI_2, TAU};

/// Segundos sin tocar nada tras los cuales el reloj se detiene.
const INACTIVO: f32 = 60.0;
const OPCIONES: [(u32, &str); 5] = [(20, "20 min"), (30, "30 min"), (45, "45 min"), (60, "1 hora"), (0, "Sin límite")];

pub fn hash(sal: &str, clave: &str) -> String {
    Sha256::digest(format!("{sal}:{clave}").as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

pub enum RT {
    Desbloqueado,
    Apagar,
}

pub enum Suceso {
    JuegosAgotados,
    SesionTerminada,
}

#[derive(PartialEq)]
enum Fase {
    Clave,
    Elegir,
    Fin(f32),
}

pub struct Tiempo {
    pub activo: bool,
    pub bloqueado: bool,
    fase: Fase,
    entrada: String,
    error: f32,
    sal: String,
    hash: String,
    limite: Option<f32>,
    uso: f32,
    juegos_lim: Option<f32>,
    juegos_uso: f32,
    ultimo_input: f32,
    aviso5: bool,
    aviso1: bool,
    aviso_juegos: bool,
    guardar_t: f32,
}

fn opcion_r(i: usize) -> R {
    R::new(122.0 + i as f32 * 158.0, 330.0, 146.0, 76.0)
}
const APAGAR: R = R::new(W as f32 - 190.0, H as f32 - 70.0, 170.0, 52.0);
const CAJA: R = R::new(W as f32 / 2.0 - 160.0, 340.0, 320.0, 70.0);

fn archivo_juegos() -> std::path::PathBuf {
    let hoy = chrono::Local::now().format("%Y-%m-%d").to_string();
    crate::apps::ruta_datos(".local/share/nidito").join(format!("juegos-{hoy}.txt"))
}

pub fn llamar(cx: &Cx) -> String {
    if !cx.apodo.trim().is_empty() {
        cx.apodo.trim().to_string()
    } else {
        cx.nombre.trim().to_string()
    }
}

fn frase_bloqueo(n: &str) -> String {
    if n.is_empty() {
        "¡Hola! Pídele a papá o mamá que pongan la clave.".into()
    } else {
        format!("¡Hola, {n}! Pídele a papá o mamá que pongan la clave.")
    }
}
fn frase_fin(n: &str) -> String {
    if n.is_empty() {
        "¡Se acabó el tiempo! Hora de jugar afuera.".into()
    } else {
        format!("¡Se acabó el tiempo, {n}! Hora de jugar afuera.")
    }
}

pub fn frases(f: &mut Frases, llamado: &str) {
    f.es(frase_bloqueo(llamado));
    f.en("Hello! Ask mom or dad for the password.");
    f.es(frase_fin(llamado));
    f.en("Time is up! Let's go play outside.");
    f.bil("Quedan cinco minutos.", "Five minutes left.");
    f.bil("Queda un minuto.", "One minute left.");
    f.bil("Los juegos se fueron a dormir. Puedes pintar, escribir o armar una clase.", "The games went to sleep. You can paint, write or make a class.");
    f.bil("Los juegos están durmiendo. Mañana vuelven.", "The games are sleeping. They come back tomorrow.");
}

impl Tiempo {
    pub fn nuevo(cfg: &Config) -> Tiempo {
        let activo = !cfg.clave.is_empty();
        let juegos_uso = std::fs::read_to_string(archivo_juegos()).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0.0);
        Tiempo {
            activo,
            bloqueado: activo,
            fase: Fase::Clave,
            entrada: String::new(),
            error: 0.0,
            sal: cfg.sal.clone(),
            hash: cfg.clave.clone(),
            limite: None,
            uso: 0.0,
            juegos_lim: if activo && cfg.juegos_min > 0 { Some(cfg.juegos_min as f32 * 60.0) } else { None },
            juegos_uso,
            ultimo_input: 0.0,
            aviso5: false,
            aviso1: false,
            aviso_juegos: false,
            guardar_t: 0.0,
        }
    }

    pub fn clave_ok(&self, intento: &str) -> bool {
        self.activo && hash(&self.sal, intento) == self.hash
    }

    pub fn verificador(&self) -> Option<(String, String)> {
        if self.activo {
            Some((self.sal.clone(), self.hash.clone()))
        } else {
            None
        }
    }

    pub fn registrar_input(&mut self, t: f32) {
        self.ultimo_input = t;
    }

    pub fn juegos_agotados(&self) -> bool {
        self.juegos_lim.is_some_and(|l| self.juegos_uso >= l)
    }

    pub fn agregar(&mut self, min: f32) {
        if let Some(l) = &mut self.limite {
            *l += min * 60.0;
        }
        if let Some(j) = &mut self.juegos_lim {
            if self.juegos_uso >= *j {
                *j = self.juegos_uso + min * 60.0;
            }
        }
        self.aviso5 = false;
        self.aviso1 = false;
        self.aviso_juegos = false;
    }

    pub fn bloquear(&mut self) {
        if self.activo {
            self.bloqueado = true;
            self.fase = Fase::Clave;
            self.entrada.clear();
        }
    }

    pub fn saludar(&self, cx: &mut Cx) {
        let n = llamar(cx);
        cx.decir_v(&[frase_bloqueo(&n)], &["Hello! Ask mom or dad for the password."]);
    }

    fn guardar_juegos(&self) {
        let p = archivo_juegos();
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let _ = std::fs::write(p, format!("{}", self.juegos_uso as u32));
    }

    pub fn evento(&mut self, ev: &Ev, cx: &mut Cx) -> Option<RT> {
        if let Ev::Clic(x, y) = ev {
            if APAGAR.dentro((*x, *y)) {
                return Some(RT::Apagar);
            }
        }
        match self.fase {
            Fase::Clave => match ev {
                Ev::Texto(t) => {
                    for c in t.chars().filter(|c| !c.is_control()) {
                        if self.entrada.chars().count() < 32 {
                            self.entrada.push(c);
                        }
                    }
                }
                Ev::Tecla(T::Espacio) => self.entrada.push(' '),
                Ev::Tecla(T::Borrar) => {
                    self.entrada.pop();
                }
                Ev::Tecla(T::Enter) => {
                    if self.clave_ok(&self.entrada) {
                        self.fase = Fase::Elegir;
                        cx.sonar(Son::Ding);
                    } else {
                        self.error = 0.6;
                        cx.sonar(Son::Boop);
                    }
                    self.entrada.clear();
                }
                _ => {}
            },
            Fase::Elegir => {
                if let Ev::Clic(x, y) = ev {
                    for (i, (min, _)) in OPCIONES.iter().enumerate() {
                        if opcion_r(i).dentro((*x, *y)) {
                            self.limite = if *min == 0 { None } else { Some(*min as f32 * 60.0) };
                            self.uso = 0.0;
                            self.aviso5 = false;
                            self.aviso1 = false;
                            self.bloqueado = false;
                            self.fase = Fase::Clave;
                            cx.sonar(Son::Fanfarria);
                            return Some(RT::Desbloqueado);
                        }
                    }
                }
                if let Ev::Tecla(T::Escape) = ev {
                    self.fase = Fase::Clave;
                }
            }
            Fase::Fin(_) => {}
        }
        None
    }

    pub fn tick(&mut self, dt: f32, cx: &mut Cx, en_juego: bool) -> Option<Suceso> {
        self.error = (self.error - dt).max(0.0);
        if let Fase::Fin(t) = &mut self.fase {
            *t += dt;
            if *t > 8.0 {
                self.fase = Fase::Clave;
                self.saludar(cx);
            }
            return None;
        }
        if !self.activo || self.bloqueado {
            return None;
        }
        if cx.t - self.ultimo_input > INACTIVO {
            return None;
        }
        self.uso += dt;
        if en_juego {
            self.juegos_uso += dt;
            self.guardar_t += dt;
            if self.guardar_t > 20.0 {
                self.guardar_t = 0.0;
                self.guardar_juegos();
            }
            if self.juegos_agotados() && !self.aviso_juegos {
                self.aviso_juegos = true;
                self.guardar_juegos();
                cx.decir_bil(
                    "Los juegos se fueron a dormir. Puedes pintar, escribir o armar una clase.",
                    "The games went to sleep. You can paint, write or make a class.",
                );
                return Some(Suceso::JuegosAgotados);
            }
        }
        if let Some(l) = self.limite {
            let queda = l - self.uso;
            if queda <= 300.0 && !self.aviso5 && l > 400.0 {
                self.aviso5 = true;
                cx.decir_bil("Quedan cinco minutos.", "Five minutes left.");
            }
            if queda <= 60.0 && !self.aviso1 {
                self.aviso1 = true;
                cx.decir_bil("Queda un minuto.", "One minute left.");
            }
            if queda <= 0.0 {
                self.guardar_juegos();
                self.bloqueado = true;
                self.fase = Fase::Fin(0.0);
                cx.sonar(Son::Fanfarria);
                let n = llamar(cx);
                cx.decir_v(&[frase_fin(&n)], &["Time is up! Let's go play outside."]);
                return Some(Suceso::SesionTerminada);
            }
        }
        None
    }

    /// Fracción restante (sesión o juegos si está jugando), para el círculo de la barra.
    pub fn restante(&self, en_juego: bool) -> Option<(f32, bool)> {
        if !self.activo {
            return None;
        }
        let ses = self.limite.map(|l| ((l - self.uso) / l).clamp(0.0, 1.0));
        let jue = self.juegos_lim.map(|l| ((l - self.juegos_uso) / l).clamp(0.0, 1.0));
        match (en_juego, jue, ses) {
            (true, Some(j), Some(s)) => Some(if j < s { (j, true) } else { (s, false) }),
            (true, Some(j), None) => Some((j, true)),
            (_, _, Some(s)) => Some((s, false)),
            _ => None,
        }
    }

    pub fn dibujar(&self, l: &mut Lienzo, cx: &Cx) {
        let (w, h) = (W as f32, H as f32);
        l.degradado_v(0.0, 0.0, w, h, rgb(30, 36, 80), rgb(80, 60, 130));
        for k in 0..40 {
            let x = (k as f32 * 197.0) % w;
            let y = (k as f32 * 83.0) % (h * 0.6);
            let b = 0.5 + 0.5 * (cx.t * 1.5 + k as f32).sin();
            l.circulo(x, y, 1.5 + (k % 3) as f32 * 0.7, alfa(BLANCO, 0.4 + 0.5 * b));
        }
        // luna
        l.circulo(860.0, 110.0, 50.0, rgb(255, 240, 180));
        l.circulo(882.0, 96.0, 46.0, rgb(40, 44, 92));
        // nido con pajarito durmiendo
        let (nx, ny) = (w / 2.0, 130.0);
        l.ovalo(nx, ny + 40.0, 120.0, 46.0, rgb(150, 100, 55));
        dibujos::dibujar(l, Obj::Pajaro, nx, ny, 120.0);
        l.ovalo(nx, ny + 50.0, 130.0, 36.0, rgb(170, 115, 65));
        for k in 0..9 {
            let a = k as f32 * 0.7;
            l.linea(nx - 120.0 + k as f32 * 30.0, ny + 40.0 + a.sin() * 8.0, nx - 100.0 + k as f32 * 30.0, ny + 64.0, 4.0, rgb(120, 80, 40));
        }
        let z = (cx.t * 0.8).fract();
        l.texto("z", nx + 60.0 + z * 30.0, ny - 60.0 - z * 40.0, 24.0 + z * 14.0, alfa(BLANCO, 1.0 - z));
        let n = llamar(cx);
        let hola = if n.is_empty() { "¡Hola!".to_string() } else { format!("¡Hola, {n}!") };
        match self.fase {
            Fase::Clave => {
                l.texto_c(&hola, w / 2.0, 250.0, 44.0, AMARILLO);
                l.texto_c("Pídele a papá o mamá la clave", w / 2.0, 300.0, 28.0, BLANCO);
                let dx = if self.error > 0.0 { (self.error * 40.0).sin() * 12.0 } else { 0.0 };
                let caja = CAJA.mover(dx, 0.0);
                l.rrect_r(caja, 16.0, alfa(BLANCO, 0.92));
                if self.error > 0.0 {
                    l.rrect_borde(caja, 16.0, 3.0, ROJO);
                }
                let n = self.entrada.chars().count();
                for k in 0..n {
                    l.circulo(caja.x + 30.0 + k as f32 * 22.0, caja.cy(), 7.0, TINTA);
                }
                if (cx.t * TAU).sin() > 0.0 {
                    l.rect(caja.x + 22.0 + n as f32 * 22.0, caja.y + 18.0, 3.0, 34.0, TINTA);
                }
                l.texto_c("Clave de adultos y Enter", w / 2.0, 440.0, 20.0, alfa(BLANCO, 0.7));
            }
            Fase::Elegir => {
                l.texto_c("¿Cuánto tiempo?", w / 2.0, 285.0, 36.0, BLANCO);
                for (i, (_, t)) in OPCIONES.iter().enumerate() {
                    let r = opcion_r(i);
                    let r2 = if r.dentro(cx.raton) { r.crecer(4.0) } else { r };
                    l.rrect_r(r2, 18.0, if i == 4 { rgb(120, 110, 170) } else { VERDE });
                    l.texto_c(t, r2.cx(), r2.cy(), 28.0, BLANCO);
                }
                if let Some(j) = self.juegos_lim {
                    let queda = ((j - self.juegos_uso) / 60.0).max(0.0).ceil();
                    l.texto_c(&format!("Juegos hoy: quedan {queda} min"), w / 2.0, 440.0, 22.0, alfa(BLANCO, 0.8));
                }
            }
            Fase::Fin(_) => {
                l.texto_c("¡Se acabó el tiempo!", w / 2.0, 300.0, 50.0, AMARILLO);
                l.texto_c("Hora de jugar afuera", w / 2.0, 365.0, 34.0, BLANCO);
                dibujos::dibujar(l, Obj::Sol, w / 2.0 - 220.0, 360.0, 70.0);
                dibujos::dibujar(l, Obj::Arbol, w / 2.0 + 230.0, 360.0, 90.0);
            }
        }
        let r = APAGAR;
        l.rrect_r(r, 14.0, if r.dentro(cx.raton) { rgb(220, 70, 70) } else { rgb(170, 60, 70) });
        l.texto_c("Apagar", r.cx() + 14.0, r.cy(), 26.0, BLANCO);
        let (ix, iy, ir) = (r.x + 28.0, r.cy(), 11.0);
        let pts: Vec<(f32, f32)> = (0..=16).map(|i| {
            let a = -FRAC_PI_2 + 0.6 + (TAU - 1.2) * i as f32 / 16.0;
            (ix + a.cos() * ir, iy + a.sin() * ir)
        }).collect();
        l.polilinea(&pts, 3.0, BLANCO);
        l.linea(ix, iy - ir - 3.0, ix, iy - 2.0, 3.0, BLANCO);
    }
}
