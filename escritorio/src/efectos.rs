//! Confeti y chispas.

use crate::cx::Rng;
use crate::gfx::*;
use std::f32::consts::TAU;
use tiny_skia::Pixmap;

struct Pz {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    rot: f32,
    vr: f32,
    c: C,
    vida: f32,
    t: f32,
    forma: u8,
    tam: f32,
    grav: f32,
}

#[derive(Default)]
pub struct Confeti {
    ps: Vec<Pz>,
}

impl Confeti {
    pub fn lanzar(&mut self, x: f32, y: f32, n: usize, rng: &mut Rng) {
        for _ in 0..n {
            let a = rng.rango(0.0, TAU);
            let v = rng.rango(150.0, 450.0);
            self.ps.push(Pz {
                x,
                y,
                vx: a.cos() * v,
                vy: a.sin() * v - 250.0,
                rot: rng.rango(0.0, TAU),
                vr: rng.rango(-8.0, 8.0),
                c: *rng.elegir(&ARCOIRIS),
                vida: rng.rango(1.4, 2.4),
                t: 0.0,
                forma: rng.n(3) as u8,
                tam: rng.rango(6.0, 11.0),
                grav: 520.0,
            });
        }
        self.limitar();
    }

    pub fn chispas(&mut self, x: f32, y: f32, n: usize, rng: &mut Rng, c: Option<C>) {
        for _ in 0..n {
            let a = rng.rango(0.0, TAU);
            let v = rng.rango(30.0, 120.0);
            self.ps.push(Pz {
                x,
                y,
                vx: a.cos() * v,
                vy: a.sin() * v,
                rot: rng.rango(0.0, TAU),
                vr: rng.rango(-4.0, 4.0),
                c: c.unwrap_or(*rng.elegir(&ARCOIRIS)),
                vida: rng.rango(0.5, 0.9),
                t: 0.0,
                forma: 2,
                tam: rng.rango(5.0, 9.0),
                grav: 60.0,
            });
        }
        self.limitar();
    }

    fn limitar(&mut self) {
        if self.ps.len() > 260 {
            let sobra = self.ps.len() - 260;
            self.ps.drain(0..sobra);
        }
    }

    pub fn actualizar(&mut self, dt: f32) {
        for p in &mut self.ps {
            p.vy += p.grav * dt;
            p.vx *= 1.0 - 0.8 * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.rot += p.vr * dt;
            p.t += dt;
        }
        self.ps.retain(|p| p.t < p.vida);
    }

    pub fn dibujar(&self, pix: &mut Pixmap) {
        for p in &self.ps {
            let a = 1.0 - (p.t / p.vida).powi(3);
            let c = alfa(p.c, a);
            match p.forma {
                0 => {
                    let (s, co) = p.rot.sin_cos();
                    let (w, h) = (p.tam, p.tam * 0.55);
                    let pt = |x: f32, y: f32| (p.x + x * co - y * s, p.y + x * s + y * co);
                    pix.poligono(&[pt(-w, -h), pt(w, -h), pt(w, h), pt(-w, h)], c);
                }
                1 => pix.circulo(p.x, p.y, p.tam * 0.6, c),
                _ => pix.estrella(p.x, p.y, p.tam, p.rot, c),
            }
        }
    }

    pub fn activo(&self) -> bool {
        !self.ps.is_empty()
    }
}
