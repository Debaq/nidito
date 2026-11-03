//! Menú para adultos (Ctrl+Alt+P): pregunta de multiplicar y luego volumen, terminal, reiniciar, apagar.

use crate::apps::{Ev, T};
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
}

enum Fase {
    Pregunta,
    Menu,
}

pub struct Padres {
    fase: Fase,
    a: u32,
    b: u32,
    entrada: String,
    error: f32,
    volumen: String,
    verificador: Option<(String, String)>,
}

const PANEL: R = R::new(212.0, 90.0, 600.0, 420.0);

fn boton(i: usize) -> R {
    R::new(PANEL.x + 30.0 + (i % 2) as f32 * 280.0, PANEL.y + 110.0 + (i / 2) as f32 * 74.0, 260.0, 62.0)
}

const BOTONES: [&str; 8] = ["Volumen −", "Volumen +", "+10 minutos", "Bloquear ahora", "Terminal", "Reiniciar", "Apagar", "Volver"];

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
    pub fn nuevo(cx: &mut Cx, verificador: Option<(String, String)>) -> Padres {
        Padres {
            fase: Fase::Pregunta,
            a: 6 + cx.rng.n(4) as u32,
            b: 6 + cx.rng.n(4) as u32,
            entrada: String::new(),
            error: 0.0,
            volumen: String::new(),
            verificador,
        }
    }

    fn correcto(&self) -> bool {
        match &self.verificador {
            Some((sal, h)) => crate::tiempo::hash(sal, &self.entrada) == *h,
            None => self.entrada.parse::<u32>().ok() == Some(self.a * self.b),
        }
    }

    pub fn evento(&mut self, ev: &Ev, cx: &mut Cx) -> RP {
        if let Ev::Tecla(T::Escape) = ev {
            return RP::Cerrar;
        }
        match self.fase {
            Fase::Pregunta => match ev {
                Ev::Texto(t) => {
                    let clave = self.verificador.is_some();
                    for c in t.chars().filter(|c| if clave { !c.is_control() } else { c.is_ascii_digit() }) {
                        if self.entrada.chars().count() < 32 {
                            self.entrada.push(c);
                        }
                    }
                }
                Ev::Tecla(T::Espacio) if self.verificador.is_some() => self.entrada.push(' '),
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
                                4 => return RP::Accion(Accion::Salir(20)),
                                5 => return RP::Accion(Accion::Reiniciar),
                                6 => return RP::Accion(Accion::Apagar),
                                _ => return RP::Cerrar,
                            }
                        }
                    }
                }
            }
        }
        RP::Seguir
    }

    pub fn actualizar(&mut self, dt: f32) {
        self.error = (self.error - dt).max(0.0);
    }

    pub fn dibujar(&self, l: &mut Lienzo, cx: &Cx) {
        l.rect(0.0, 0.0, W as f32, H as f32, alfa(NEGRO, 0.6));
        let p = PANEL;
        l.rrect_r(p, 20.0, rgb(245, 246, 250));
        l.texto_c("Menú para adultos", p.cx(), p.y + 40.0, 34.0, TINTA);
        match self.fase {
            Fase::Pregunta => {
                let dx = if self.error > 0.0 { (self.error * 40.0).sin() * 10.0 } else { 0.0 };
                if self.verificador.is_some() {
                    l.texto_c("Clave", p.cx(), p.y + 140.0, 40.0, TINTA);
                } else {
                    l.texto_c(&format!("¿Cuánto es {} × {}?", self.a, self.b), p.cx(), p.y + 140.0, 40.0, TINTA);
                }
                let caja = R::new(p.cx() - 100.0 + dx, p.y + 190.0, 200.0, 80.0);
                l.rrect_r(caja, 14.0, BLANCO);
                l.rrect_borde(caja, 14.0, 3.0, if self.error > 0.0 { ROJO } else { AZUL });
                if self.verificador.is_some() {
                    let n = self.entrada.chars().count();
                    for k in 0..n {
                        l.circulo(caja.cx() - (n as f32 - 1.0) * 9.0 + k as f32 * 18.0, caja.cy(), 6.0, TINTA);
                    }
                } else {
                    l.texto_c(&self.entrada, caja.cx(), caja.cy(), 50.0, TINTA);
                }
                l.texto_c("Escribe el resultado y presiona Enter", p.cx(), p.y + 320.0, 22.0, GRIS);
                l.texto_c("Esc para volver", p.cx(), p.y + 352.0, 20.0, GRIS);
            }
            Fase::Menu => {
                if !self.volumen.is_empty() {
                    l.texto_c(&format!("Volumen: {}", self.volumen), p.cx(), p.y + 80.0, 22.0, GRIS);
                }
                let colores = [AZUL, AZUL, VERDE, MORADO, rgb(90, 100, 120), NARANJO, ROJO, rgb(120, 130, 150)];
                for (i, t) in BOTONES.iter().enumerate() {
                    let r = boton(i);
                    let c = colores[i];
                    let r2 = if r.dentro(cx.raton) { r.crecer(3.0) } else { r };
                    l.rrect_r(r2, 14.0, c);
                    l.texto_c(t, r2.cx(), r2.cy(), 28.0, BLANCO);
                }
            }
        }
    }
}
