//! Sonidos sintetizados (no hay archivos de audio).

use std::f32::consts::TAU;

pub const SR: u32 = 22050;

pub struct Sonidos {
    pub ding: Vec<f32>,
    pub fanfarria: Vec<f32>,
    pub boop: Vec<f32>,
    pub pop: Vec<f32>,
    pub whoosh: Vec<f32>,
    pub clic: Vec<f32>,
    pub sello: Vec<f32>,
    pub aplauso: Vec<f32>,
    pub tic: Vec<f32>,
    pub notas: Vec<Vec<f32>>,
    /// "Cumpleaños feliz" (melodía tradicional, dominio público)
    pub cumple: Vec<f32>,
}

/// Melodía de "Cumpleaños feliz" en Do, compás de 3/4. (frecuencia, duración en negras)
fn melodia_cumple() -> Vec<f32> {
    let (g4, a4, b4, c5, d5, e5, f5, g5) = (392.0, 440.0, 493.88, 523.25, 587.33, 659.25, 698.46, 783.99);
    let notas: [(f32, f32); 25] = [
        (g4, 0.75), (g4, 0.25), (a4, 1.0), (g4, 1.0), (c5, 1.0), (b4, 2.0),
        (g4, 0.75), (g4, 0.25), (a4, 1.0), (g4, 1.0), (d5, 1.0), (c5, 2.0),
        (g4, 0.75), (g4, 0.25), (g5, 1.0), (e5, 1.0), (c5, 1.0), (b4, 1.0), (a4, 2.0),
        (f5, 0.75), (f5, 0.25), (e5, 1.0), (c5, 1.0), (d5, 1.0), (c5, 2.5),
    ];
    let negra = 0.42;
    let mut v = Vec::new();
    let mut t0 = 0.0;
    for (f, d) in notas {
        let dur = d * negra;
        let n: Vec<f32> = (0..largo(dur + 0.25))
            .map(|i| {
                let t = i as f32 / SR as f32;
                let env = (t / 0.01).min(1.0) * (-t * 2.8).exp() * if t > dur { (1.0 - (t - dur) / 0.25).max(0.0) } else { 1.0 };
                0.22 * env * ((TAU * f * t).sin() + 0.3 * (TAU * 2.0 * f * t).sin() + 0.1 * (TAU * 3.0 * f * t).sin())
            })
            .collect();
        mezclar_en(&mut v, &n, t0);
        t0 += dur;
    }
    v
}

struct Ruido(u32);
impl Ruido {
    fn s(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.0 >> 8) as f32 / (1u32 << 23) as f32 - 1.0
    }
}

fn largo(seg: f32) -> usize {
    (seg * SR as f32) as usize
}

fn campana(f: f32, dur: f32, vol: f32, decaim: f32) -> Vec<f32> {
    (0..largo(dur))
        .map(|i| {
            let t = i as f32 / SR as f32;
            let env = (t / 0.004).min(1.0) * (-t * decaim).exp();
            vol * env * ((TAU * f * t).sin() + 0.3 * (TAU * 2.0 * f * t).sin() + 0.1 * (TAU * 3.0 * f * t).sin())
        })
        .collect()
}

fn mezclar_en(dest: &mut Vec<f32>, src: &[f32], desde: f32) {
    let o = largo(desde);
    if dest.len() < o + src.len() {
        dest.resize(o + src.len(), 0.0);
    }
    for (i, s) in src.iter().enumerate() {
        dest[o + i] += s;
    }
}

fn recortar(mut v: Vec<f32>) -> Vec<f32> {
    // fundido final corto para evitar chasquidos
    let n = v.len();
    let f = largo(0.01).min(n);
    for i in 0..f {
        v[n - 1 - i] *= i as f32 / f as f32;
    }
    for s in &mut v {
        *s = s.clamp(-1.0, 1.0);
    }
    v
}

impl Sonidos {
    pub fn generar() -> Sonidos {
        let mut r = Ruido(12345);

        let mut ding = Vec::new();
        mezclar_en(&mut ding, &campana(1046.5, 0.4, 0.3, 7.0), 0.0);
        mezclar_en(&mut ding, &campana(1568.0, 0.55, 0.28, 6.0), 0.1);

        let mut fanfarria = Vec::new();
        for (i, f) in [523.25, 659.25, 783.99, 1046.5].iter().enumerate() {
            let dur = if i == 3 { 0.9 } else { 0.3 };
            mezclar_en(&mut fanfarria, &campana(*f, dur, 0.25, if i == 3 { 3.5 } else { 8.0 }), i as f32 * 0.11);
        }

        let mut fase = 0.0f32;
        let boop: Vec<f32> = (0..largo(0.3))
            .map(|i| {
                let t = i as f32 / SR as f32;
                let f = 330.0 - 130.0 * (t / 0.3);
                fase += TAU * f / SR as f32;
                0.3 * (t / 0.01).min(1.0) * (1.0 - t / 0.3) * fase.sin()
            })
            .collect();

        let mut fase = 0.0f32;
        let pop: Vec<f32> = (0..largo(0.14))
            .map(|i| {
                let t = i as f32 / SR as f32;
                let f = 900.0 - 700.0 * (t / 0.14);
                fase += TAU * f / SR as f32;
                0.45 * r.s() * (-t * 45.0).exp() + 0.4 * fase.sin() * (-t * 28.0).exp()
            })
            .collect();

        let dur = 0.4;
        let mut y = 0.0f32;
        let whoosh: Vec<f32> = (0..largo(dur))
            .map(|i| {
                let t = i as f32 / SR as f32;
                let k = 0.04 + 0.25 * (t / dur * std::f32::consts::PI).sin();
                y += (r.s() - y) * k;
                0.5 * y * (t / dur * std::f32::consts::PI).sin()
            })
            .collect();

        let clic: Vec<f32> = (0..largo(0.04))
            .map(|i| {
                let t = i as f32 / SR as f32;
                0.3 * (TAU * 1800.0 * t).sin() * (-t * 120.0).exp()
            })
            .collect();

        let tic: Vec<f32> = (0..largo(0.06))
            .map(|i| {
                let t = i as f32 / SR as f32;
                0.35 * (TAU * 2400.0 * t).sin() * (-t * 80.0).exp()
            })
            .collect();

        let mut sello = campana(392.0, 0.3, 0.2, 10.0);
        let golpe: Vec<f32> = (0..largo(0.15))
            .map(|i| {
                let t = i as f32 / SR as f32;
                0.4 * (TAU * 140.0 * t).sin() * (-t * 25.0).exp()
            })
            .collect();
        mezclar_en(&mut sello, &golpe, 0.0);

        // aplausos: muchas palmadas de ruido
        let mut aplauso = vec![0.0; largo(2.2)];
        let mut pos = 0.0f32;
        while pos < 2.0 {
            let palmada: Vec<f32> = (0..largo(0.05))
                .map(|i| {
                    let t = i as f32 / SR as f32;
                    0.22 * r.s() * (-t * 70.0).exp()
                })
                .collect();
            let vol = 1.0 - pos / 2.2;
            let p: Vec<f32> = palmada.iter().map(|s| s * vol).collect();
            mezclar_en(&mut aplauso, &p, pos);
            pos += 0.012 + (r.s().abs()) * 0.03;
        }

        let pent = [523.25, 587.33, 659.25, 783.99, 880.0, 1046.5, 1174.66, 1318.51];
        let notas = pent
            .iter()
            .map(|f| {
                (0..largo(0.7))
                    .map(|i| {
                        let t = i as f32 / SR as f32;
                        let env = (t / 0.003).min(1.0) * (-t * 7.0).exp();
                        0.3 * env * ((TAU * f * t).sin() + 0.25 * (TAU * 4.0 * f * t).sin() * (-t * 20.0).exp())
                    })
                    .collect()
            })
            .map(recortar)
            .collect();

        Sonidos {
            ding: recortar(ding),
            fanfarria: recortar(fanfarria),
            boop: recortar(boop),
            pop: recortar(pop),
            whoosh: recortar(whoosh),
            clic: recortar(clic),
            sello: recortar(sello),
            aplauso: recortar(aplauso),
            tic: recortar(tic),
            notas,
            cumple: recortar(melodia_cumple()),
        }
    }
}
