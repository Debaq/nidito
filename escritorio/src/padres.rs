//! Menú para adultos (Ctrl+Alt+P): pide la clave de adultos y luego volumen, tiempo,
//! ajustes (juegos, pausa para los ojos, cumpleaños), terminal, reiniciar y apagar.

use crate::apps::{Ev, T};
use crate::config::Ajustes;
use crate::cx::{Cx, Son};
use crate::escritorio::Accion;
use crate::gfx::*;
use crate::{H, W};
use std::process::Command;

pub enum RP {
    Seguir,
    Cerrar,
    Accion(Accion),
    MasTiempo,
    Bloquear,
    /// se guardaron ajustes nuevos
    Ajustes(Ajustes),
}

enum Fase {
    Pregunta,
    Menu,
    Ajustes,
}

pub struct Padres {
    fase: Fase,
    entrada: String,
    error: f32,
    volumen: String,
    verificador: Option<(String, String)>,
    ajustes: Ajustes,
}

const PANEL: R = R::new(212.0, 40.0, 600.0, 500.0);

fn boton(i: usize) -> R {
    if i == BOTONES.len() - 1 {
        // "Volver" ocupa toda la última fila
        return R::new(PANEL.x + 30.0, PANEL.y + 100.0 + (i / 2) as f32 * 72.0, 540.0, 60.0);
    }
    R::new(PANEL.x + 30.0 + (i % 2) as f32 * 280.0, PANEL.y + 100.0 + (i / 2) as f32 * 72.0, 260.0, 60.0)
}

const BOTONES: [&str; 9] = ["Volumen −", "Volumen +", "+10 minutos", "Bloquear ahora", "Ajustes", "Terminal", "Reiniciar", "Apagar", "Volver"];

const PAUSAS: [u32; 6] = [0, 10, 15, 20, 30, 45];
const MESES: [&str; 12] = ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"];

/// Botones − / + de los ajustes: (fila, 0 = menos | 1 = más). La fila 3 es el mes del cumpleaños.
fn ajuste_btn(fila: usize, mas: usize) -> (f32, f32) {
    let y = PANEL.y + [120.0, 210.0, 300.0, 300.0][fila];
    let x = match fila {
        2 => PANEL.x + [250.0, 330.0][mas],
        3 => PANEL.x + [400.0, 570.0][mas],
        _ => PANEL.x + [330.0, 540.0][mas],
    };
    (x, y)
}
const GUARDAR: R = R::new(PANEL.x + 30.0, PANEL.y + 420.0, 260.0, 60.0);
const CANCELAR: R = R::new(PANEL.x + 310.0, PANEL.y + 420.0, 260.0, 60.0);

fn paso_en(lista: &[u32], v: u32, d: i32) -> u32 {
    let i = lista.iter().position(|&x| x >= v).unwrap_or(lista.len() - 1) as i32;
    lista[(i + d).clamp(0, lista.len() as i32 - 1) as usize]
}

/// Mueve el volumen general (Master) y deja al máximo los controles intermedios
/// (PCM, Speaker...), que si quedan bajos frenan el volumen aunque Master esté al 100%.
fn cambiar_volumen(paso: &str) -> String {
    for c in ["PCM", "Speaker", "Headphone", "Front"] {
        let _ = Command::new("amixer").args(["-q", "sset", c, "100%", "unmute"]).status();
    }
    amixer(&["sset", "Master", paso, "unmute"])
}

fn amixer(args: &[&str]) -> String {
    Command::new("amixer")
        .args(args)
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).to_string();
            s.split('[').find(|p| p.contains('%')).map(|p| p.trim_end_matches(']').split(']').next().unwrap_or("").to_string())
        })
        .unwrap_or_default()
}

impl Padres {
    /// Pide la clave de adultos. Si no hay clave configurada, entra directo (y avisa).
    pub fn nuevo(_cx: &mut Cx, verificador: Option<(String, String)>, ajustes: Ajustes) -> Padres {
        let sin_clave = verificador.is_none();
        Padres {
            fase: if sin_clave { Fase::Menu } else { Fase::Pregunta },
            entrada: String::new(),
            error: 0.0,
            volumen: if sin_clave { amixer(&["get", "Master"]) } else { String::new() },
            verificador,
            ajustes,
        }
    }

    /// Para las pruebas automáticas: entra directo a Ajustes.
    pub fn en_ajustes(mut self) -> Padres {
        self.fase = Fase::Ajustes;
        self
    }

    fn correcto(&self) -> bool {
        match &self.verificador {
            Some((sal, h)) => crate::tiempo::hash(sal, &self.entrada) == *h,
            None => true,
        }
    }

    pub fn evento(&mut self, ev: &Ev, cx: &mut Cx) -> RP {
        if let Ev::Tecla(T::Escape) = ev {
            return RP::Cerrar;
        }
        match self.fase {
            Fase::Pregunta => match ev {
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
                    if self.correcto() {
                        self.fase = Fase::Menu;
                        self.volumen = amixer(&["get", "Master"]);
                        cx.sonar(Son::Ding);
                    } else {
                        self.entrada.clear();
                        self.error = 0.6;
                        cx.sonar(Son::Boop);
                    }
                }
                _ => {}
            },
            Fase::Menu => {
                if let Ev::Clic(x, y) = ev {
                    for i in 0..BOTONES.len() {
                        if boton(i).dentro((*x, *y)) {
                            cx.sonar(Son::Clic);
                            match i {
                                0 => self.volumen = cambiar_volumen("10%-"),
                                1 => self.volumen = cambiar_volumen("10%+"),
                                2 => return RP::MasTiempo,
                                3 => return RP::Bloquear,
                                4 => self.fase = Fase::Ajustes,
                                5 => return RP::Accion(Accion::Salir(20)),
                                6 => return RP::Accion(Accion::Reiniciar),
                                7 => return RP::Accion(Accion::Apagar),
                                _ => return RP::Cerrar,
                            }
                        }
                    }
                }
            }
            Fase::Ajustes => {
                if let Ev::Clic(x, y) = ev {
                    let p = (*x, *y);
                    if GUARDAR.dentro(p) {
                        cx.sonar(Son::Ding);
                        return RP::Ajustes(self.ajustes);
                    }
                    if CANCELAR.dentro(p) {
                        cx.sonar(Son::Clic);
                        self.fase = Fase::Menu;
                        return RP::Seguir;
                    }
                    for fila in 0..4 {
                        for mas in 0..2 {
                            if dist(p, ajuste_btn(fila, mas)) < 26.0 {
                                cx.sonar(Son::Clic);
                                self.cambiar(fila, if mas == 1 { 1 } else { -1 });
                            }
                        }
                    }
                }
            }
        }
        RP::Seguir
    }

    fn cambiar(&mut self, fila: usize, d: i32) {
        let a = &mut self.ajustes;
        match fila {
            0 => a.juegos_min = (a.juegos_min as i32 + d * 5).clamp(0, 180) as u32,
            1 => a.pausa_ojos = paso_en(&PAUSAS, a.pausa_ojos, d),
            2 => {
                // día: 0 = sin cumpleaños
                let (dia, mes) = a.cumpleanos.unwrap_or((0, 1));
                let dia = (dia as i32 + d).rem_euclid(32) as u32;
                a.cumpleanos = (dia > 0).then_some((dia, mes));
            }
            _ => {
                let (dia, mes) = a.cumpleanos.unwrap_or((1, 1));
                let mes = (mes as i32 - 1 + d).rem_euclid(12) as u32 + 1;
                a.cumpleanos = Some((dia, mes));
            }
        }
    }

    pub fn actualizar(&mut self, dt: f32) {
        self.error = (self.error - dt).max(0.0);
    }

    pub fn dibujar(&self, l: &mut Lienzo, cx: &Cx) {
        l.rect(0.0, 0.0, W as f32, H as f32, alfa(NEGRO, 0.6));
        let p = PANEL;
        l.rrect_r(p, 20.0, rgb(245, 246, 250));
        let titulo = if matches!(self.fase, Fase::Ajustes) { "Ajustes" } else { "Menú para adultos" };
        l.texto_c(titulo, p.cx(), p.y + 40.0, 34.0, TINTA);
        match self.fase {
            Fase::Pregunta => {
                let dx = if self.error > 0.0 { (self.error * 40.0).sin() * 10.0 } else { 0.0 };
                l.texto_c("Clave de adultos", p.cx(), p.y + 140.0, 40.0, TINTA);
                let caja = R::new(p.cx() - 100.0 + dx, p.y + 190.0, 200.0, 80.0);
                l.rrect_r(caja, 14.0, BLANCO);
                l.rrect_borde(caja, 14.0, 3.0, if self.error > 0.0 { ROJO } else { AZUL });
                let n = self.entrada.chars().count();
                for k in 0..n {
                    l.circulo(caja.cx() - (n as f32 - 1.0) * 9.0 + k as f32 * 18.0, caja.cy(), 6.0, TINTA);
                }
                l.texto_c("Escribe la clave y presiona Enter", p.cx(), p.y + 320.0, 22.0, GRIS);
                l.texto_c("Esc para volver", p.cx(), p.y + 352.0, 20.0, GRIS);
            }
            Fase::Menu => {
                if self.verificador.is_none() {
                    l.texto_c("Sin clave de adultos: créala en Terminal con  su -c nidito-clave", p.cx(), p.y + 478.0, 18.0, ROJO);
                }
                if !self.volumen.is_empty() {
                    l.texto_c(&format!("Volumen: {}", self.volumen), p.cx(), p.y + 80.0, 22.0, GRIS);
                }
                let colores = [AZUL, AZUL, VERDE, MORADO, rgb(40, 150, 140), rgb(90, 100, 120), NARANJO, ROJO, rgb(120, 130, 150)];
                for (i, t) in BOTONES.iter().enumerate() {
                    let r = boton(i);
                    let c = colores[i];
                    let r2 = if r.dentro(cx.raton) { r.crecer(3.0) } else { r };
                    l.rrect_r(r2, 14.0, c);
                    l.texto_c(t, r2.cx(), r2.cy(), 28.0, BLANCO);
                }
            }
            Fase::Ajustes => {
                let a = &self.ajustes;
                let etiqueta = |l: &mut Lienzo, fila: usize, t: &str| {
                    let (_, y) = ajuste_btn(fila, 0);
                    l.texto(t, p.x + 30.0, y - 16.0, 25.0, TINTA);
                };
                let valor = |l: &mut Lienzo, fila: usize, t: &str| {
                    let (x0, y) = ajuste_btn(fila, 0);
                    let (x1, _) = ajuste_btn(fila, 1);
                    l.texto_c(t, (x0 + x1) / 2.0, y, 26.0, AZUL);
                };
                etiqueta(l, 0, "Juegos al día");
                valor(l, 0, &if a.juegos_min == 0 { "sin tope".to_string() } else { format!("{} min", a.juegos_min) });
                etiqueta(l, 1, "Pausa para los ojos");
                valor(l, 1, &if a.pausa_ojos == 0 { "nunca".to_string() } else { format!("cada {} min", a.pausa_ojos) });
                etiqueta(l, 2, "Cumpleaños");
                let (dx0, y2) = ajuste_btn(2, 0);
                let (dx1, _) = ajuste_btn(2, 1);
                match a.cumpleanos {
                    Some((d, m)) => {
                        l.texto_c(&d.to_string(), (dx0 + dx1) / 2.0, y2, 26.0, AZUL);
                        let (mx0, _) = ajuste_btn(3, 0);
                        let (mx1, _) = ajuste_btn(3, 1);
                        l.texto_c(MESES[(m - 1) as usize], (mx0 + mx1) / 2.0, y2, 22.0, AZUL);
                        l.texto_c("de", (dx1 + mx0) / 2.0, y2, 20.0, GRIS);
                    }
                    None => l.texto_c("sin fecha", (dx0 + dx1) / 2.0, y2 + 34.0, 18.0, GRIS),
                }
                for fila in 0..4 {
                    if fila == 3 && a.cumpleanos.is_none() {
                        continue;
                    }
                    for mas in 0..2 {
                        let (x, y) = ajuste_btn(fila, mas);
                        let sobre = dist(cx.raton, (x, y)) < 26.0;
                        l.circulo(x, y, if sobre { 24.0 } else { 22.0 }, if mas == 1 { VERDE } else { NARANJO });
                        l.rect(x - 10.0, y - 2.5, 20.0, 5.0, BLANCO);
                        if mas == 1 {
                            l.rect(x - 2.5, y - 10.0, 5.0, 20.0, BLANCO);
                        }
                    }
                }
                l.texto_c("Juegos: solo cuenta si hay clave de adultos.", p.cx(), p.y + 372.0, 18.0, GRIS);
                for (r, t, c) in [(GUARDAR, "Guardar", VERDE), (CANCELAR, "Cancelar", rgb(120, 130, 150))] {
                    let r2 = if r.dentro(cx.raton) { r.crecer(3.0) } else { r };
                    l.rrect_r(r2, 14.0, c);
                    l.texto_c(t, r2.cx(), r2.cy(), 28.0, BLANCO);
                }
            }
        }
    }
}
