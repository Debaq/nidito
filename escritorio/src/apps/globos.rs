//! Globos: globos con números que suben; se revientan tocándolos o con la tecla del número.

use super::*;
use crate::cx::Son;
use crate::efectos::Confeti;
use tiny_skia::Pixmap;

struct Globo {
    x: f32,
    y: f32,
    vy: f32,
    fase: f32,
    c: C,
    n: usize,
}

pub struct Globos {
    fondo: Pixmap,
    gl: Vec<Globo>,
    prox: f32,
    reventados: usize,
    confeti: Confeti,
    celebrar: f32,
}

const RX: f32 = 46.0;
const RY: f32 = 56.0;

pub fn frases(f: &mut Frases) {
    f.bil("¡Diez globos más!", "Ten more balloons!");
}

impl Globos {
    pub fn nuevo(_cx: &mut Cx) -> Box<dyn App> {
        let mut fondo = Pixmap::new(AW as u32, AH as u32).unwrap();
        fondo.degradado_v(0.0, 0.0, AW, AH, rgb(120, 205, 255), rgb(225, 247, 255));
        for &(x, y, s) in &[(140.0, 90.0, 1.0), (560.0, 60.0, 0.8), (830.0, 150.0, 1.2), (330.0, 300.0, 0.7)] {
            for &(dx, dy, r) in &[(0.0, 0.0, 34.0), (36.0, -14.0, 40.0), (74.0, 0.0, 32.0), (36.0, 10.0, 30.0)] {
                fondo.circulo(x + dx * s, y + dy * s, r * s, alfa(BLANCO, 0.85));
            }
        }
        fondo.ovalo(500.0, AH + 40.0, 700.0, 110.0, rgb(120, 200, 110));
        Box::new(Globos { fondo, gl: Vec::new(), prox: 0.3, reventados: 0, confeti: Confeti::default(), celebrar: 0.0 })
    }

    fn reventar(&mut self, i: usize, cx: &mut Cx) {
        let g = self.gl.remove(i);
        cx.sonar(Son::Pop);
        self.confeti.chispas(g.x, g.y, 18, &mut cx.rng, Some(g.c));
        self.reventados += 1;
        if self.reventados % 10 == 0 {
            cx.sonar(Son::Fanfarria);
            self.confeti.lanzar(AW / 2.0, AH / 2.0, 80, &mut cx.rng);
            self.celebrar = 2.0;
            cx.decir_bil("¡Diez globos más!", "Ten more balloons!");
        } else {
            cx.decir_num(g.n);
        }
    }
}

impl App for Globos {
    fn evento(&mut self, ev: &Ev, cx: &mut Cx) {
        match ev {
            Ev::Clic(x, y) => {
                let hit = self.gl.iter().rposition(|g| {
                    let (dx, dy) = ((x - g.x) / (RX + 8.0), (y - g.y) / (RY + 8.0));
                    dx * dx + dy * dy <= 1.0
                });
                if let Some(i) = hit {
                    self.reventar(i, cx);
                }
            }
            Ev::Texto(t) => {
                if let Some(d) = t.chars().next().and_then(|c| c.to_digit(10)) {
                    let n = if d == 0 { 10 } else { d as usize };
                    // el más alto que siga visible
                    let mut mejor: Option<usize> = None;
                    for (i, g) in self.gl.iter().enumerate() {
                        if g.n == n && g.y < AH + 20.0 && mejor.is_none_or(|m| g.y < self.gl[m].y) {
                            mejor = Some(i);
                        }
                    }
                    match mejor {
                        Some(i) => self.reventar(i, cx),
                        None => cx.sonar(Son::Boop),
                    }
                }
            }
            _ => {}
        }
    }

    fn actualizar(&mut self, dt: f32, cx: &mut Cx) {
        self.prox -= dt;
        if self.prox <= 0.0 && self.gl.len() < 7 {
            self.prox = cx.rng.rango(0.9, 1.6);
            let mut x = cx.rng.rango(70.0, AW - 70.0);
            for _ in 0..5 {
                if self.gl.iter().all(|g| (g.x - x).abs() > 110.0 || g.y < AH - 150.0) {
                    break;
                }
                x = cx.rng.rango(70.0, AW - 70.0);
            }
            let n = 1 + cx.rng.n(10);
            self.gl.push(Globo { x, y: AH + 80.0, vy: -cx.rng.rango(38.0, 62.0), fase: cx.rng.rango(0.0, 6.0), c: ARCOIRIS[(n - 1) % 8], n });
        }
        for g in &mut self.gl {
            g.y += g.vy * dt;
            g.x += (cx.t * 1.3 + g.fase).sin() * 14.0 * dt;
        }
        self.gl.retain(|g| g.y > -RY - 90.0);
        self.confeti.actualizar(dt);
        self.celebrar = (self.celebrar - dt).max(0.0);
    }

    fn dibujar(&mut self, l: &mut Lienzo, cx: &Cx) {
        copiar_opaco(&mut l.pix, &self.fondo, 0, 0);
        for g in &self.gl {
            // hilo
            let pts: Vec<(f32, f32)> = (0..=8)
                .map(|i| {
                    let t = i as f32 / 8.0;
                    (g.x + (t * 9.0 + cx.t * 3.0 + g.fase).sin() * 5.0, g.y + RY + 6.0 + t * 80.0)
                })
                .collect();
            l.polilinea(&pts, 2.0, rgb(110, 110, 130));
            l.poligono(&[(g.x, g.y + RY - 4.0), (g.x - 9.0, g.y + RY + 10.0), (g.x + 9.0, g.y + RY + 10.0)], oscurecer(g.c, 0.15));
            let sobre = {
                let (dx, dy) = ((cx.rl.0 - g.x) / RX, (cx.rl.1 - g.y) / RY);
                dx * dx + dy * dy <= 1.0
            };
            let k = if sobre { 1.07 } else { 1.0 };
            l.ovalo(g.x, g.y, RX * k, RY * k, g.c);
            l.ovalo(g.x - 16.0, g.y - 22.0, 9.0, 15.0, alfa(BLANCO, 0.55));
            l.texto_c_borde(&g.n.to_string(), g.x, g.y + 2.0, 56.0, BLANCO, oscurecer(g.c, 0.45), 3.0);
        }
        // contador
        l.rrect(14.0, 14.0, 150.0, 64.0, 32.0, alfa(BLANCO, 0.8));
        l.ovalo(48.0, 42.0, 17.0, 21.0, ROJO);
        l.linea(48.0, 64.0, 48.0, 74.0, 2.0, GRIS);
        l.texto(&self.reventados.to_string(), 78.0, 18.0, 44.0, TINTA);
        if self.celebrar > 0.0 {
            felicitar(l, "¡Bravo!", AW / 2.0, AH / 2.0, cx.t);
        }
        self.confeti.dibujar(l);
    }
}
