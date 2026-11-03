//! Dibujo por CPU: figuras (tiny-skia) y texto (fontdue).

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use tiny_skia::*;

pub type C = [u8; 4];

pub const fn rgb(r: u8, g: u8, b: u8) -> C {
    [r, g, b, 255]
}

pub fn alfa(c: C, a: f32) -> C {
    [c[0], c[1], c[2], (c[3] as f32 * a.clamp(0.0, 1.0)) as u8]
}

pub fn mezclar(a: C, b: C, t: f32) -> C {
    let t = t.clamp(0.0, 1.0);
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    [m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2]), m(a[3], b[3])]
}

pub fn oscurecer(c: C, t: f32) -> C {
    mezclar(c, [0, 0, 0, c[3]], t)
}

pub fn aclarar(c: C, t: f32) -> C {
    mezclar(c, [255, 255, 255, c[3]], t)
}

/// h en grados, s y v en 0..1
pub fn hsv(h: f32, s: f32, v: f32) -> C {
    let h = h.rem_euclid(360.0) / 60.0;
    let i = h.floor() as i32;
    let f = h - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    let (r, g, b) = match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

pub const BLANCO: C = rgb(255, 255, 255);
pub const NEGRO: C = rgb(0, 0, 0);
pub const TINTA: C = rgb(43, 45, 66);
pub const ROJO: C = rgb(230, 57, 70);
pub const NARANJO: C = rgb(255, 140, 66);
pub const AMARILLO: C = rgb(255, 209, 59);
pub const VERDE: C = rgb(87, 196, 92);
pub const CELESTE: C = rgb(77, 200, 255);
pub const AZUL: C = rgb(58, 110, 230);
pub const MORADO: C = rgb(155, 93, 229);
pub const ROSADO: C = rgb(255, 117, 170);
pub const CAFE: C = rgb(140, 90, 50);
pub const GRIS: C = rgb(150, 150, 160);

pub const ARCOIRIS: [C; 8] = [ROJO, NARANJO, AMARILLO, VERDE, CELESTE, AZUL, MORADO, ROSADO];

pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t)
}

pub fn ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct R {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl R {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> R {
        R { x, y, w, h }
    }
    pub fn dentro(&self, p: (f32, f32)) -> bool {
        p.0 >= self.x && p.0 <= self.x + self.w && p.1 >= self.y && p.1 <= self.y + self.h
    }
    pub fn cx(&self) -> f32 {
        self.x + self.w / 2.0
    }
    pub fn cy(&self) -> f32 {
        self.y + self.h / 2.0
    }
    pub fn abajo(&self) -> f32 {
        self.y + self.h
    }
    pub fn der(&self) -> f32 {
        self.x + self.w
    }
    pub fn crecer(&self, d: f32) -> R {
        R::new(self.x - d, self.y - d, self.w + 2.0 * d, self.h + 2.0 * d)
    }
    pub fn mover(&self, dx: f32, dy: f32) -> R {
        R::new(self.x + dx, self.y + dy, self.w, self.h)
    }
    pub fn lerp(a: R, b: R, t: f32) -> R {
        let l = |x: f32, y: f32| x + (y - x) * t;
        R::new(l(a.x, b.x), l(a.y, b.y), l(a.w, b.w), l(a.h, b.h))
    }
}

pub fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

fn pintura(c: C) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color_rgba8(c[0], c[1], c[2], c[3]);
    p.anti_alias = true;
    p
}

fn trazo(ancho: f32) -> Stroke {
    Stroke {
        width: ancho,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Default::default()
    }
}

pub fn ruta_rrect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<Path> {
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    let k = 0.5523 * r;
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
    pb.finish()
}

pub fn ruta_corazon(cx: f32, cy: f32, s: f32) -> Option<Path> {
    let p = |x: f32, y: f32| (cx + x * s, cy + y * s);
    let mut pb = PathBuilder::new();
    let a = p(0.0, 0.42);
    pb.move_to(a.0, a.1);
    let c = |pb: &mut PathBuilder, v: [f32; 6]| {
        let (x1, y1) = p(v[0], v[1]);
        let (x2, y2) = p(v[2], v[3]);
        let (x3, y3) = p(v[4], v[5]);
        pb.cubic_to(x1, y1, x2, y2, x3, y3);
    };
    c(&mut pb, [-0.08, 0.36, -0.5, 0.1, -0.5, -0.12]);
    c(&mut pb, [-0.5, -0.32, -0.36, -0.45, -0.24, -0.45]);
    c(&mut pb, [-0.1, -0.45, -0.03, -0.36, 0.0, -0.27]);
    c(&mut pb, [0.03, -0.36, 0.1, -0.45, 0.24, -0.45]);
    c(&mut pb, [0.36, -0.45, 0.5, -0.32, 0.5, -0.12]);
    c(&mut pb, [0.5, 0.1, 0.08, 0.36, 0.0, 0.42]);
    pb.close();
    pb.finish()
}

pub fn ruta_estrella(cx: f32, cy: f32, r: f32, ri: f32, puntas: usize, rot: f32) -> Option<Path> {
    let mut pb = PathBuilder::new();
    let n = puntas * 2;
    for i in 0..n {
        let a = rot + i as f32 * std::f32::consts::PI / puntas as f32;
        let rr = if i % 2 == 0 { r } else { ri };
        let (x, y) = (cx + a.cos() * rr, cy + a.sin() * rr);
        if i == 0 {
            pb.move_to(x, y);
        } else {
            pb.line_to(x, y);
        }
    }
    pb.close();
    pb.finish()
}

pub fn ruta_poligono(pts: &[(f32, f32)]) -> Option<Path> {
    if pts.len() < 3 {
        return None;
    }
    let mut pb = PathBuilder::new();
    pb.move_to(pts[0].0, pts[0].1);
    for p in &pts[1..] {
        pb.line_to(p.0, p.1);
    }
    pb.close();
    pb.finish()
}

/// Figuras sobre cualquier Pixmap.
pub trait Fig {
    fn llenar(&mut self, ruta: Option<Path>, c: C);
    fn delinear(&mut self, ruta: Option<Path>, ancho: f32, c: C);
    fn degradado_v(&mut self, x: f32, y: f32, w: f32, h: f32, c1: C, c2: C);

    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: C) {
        if let Some(r) = Rect::from_xywh(x, y, w, h) {
            self.llenar(Some(PathBuilder::from_rect(r)), c)
        }
    }
    fn rrect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, c: C) {
        self.llenar(ruta_rrect(x, y, w, h, r), c)
    }
    fn rrect_r(&mut self, r: R, radio: f32, c: C) {
        self.llenar(ruta_rrect(r.x, r.y, r.w, r.h, radio), c)
    }
    fn rrect_borde(&mut self, r: R, radio: f32, ancho: f32, c: C) {
        self.delinear(ruta_rrect(r.x, r.y, r.w, r.h, radio), ancho, c)
    }
    fn circulo(&mut self, cx: f32, cy: f32, r: f32, c: C) {
        if r > 0.1 {
            self.llenar(PathBuilder::from_circle(cx, cy, r), c)
        }
    }
    fn anillo(&mut self, cx: f32, cy: f32, r: f32, ancho: f32, c: C) {
        if r > 0.1 {
            self.delinear(PathBuilder::from_circle(cx, cy, r), ancho, c)
        }
    }
    fn ovalo(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, c: C) {
        if let Some(r) = Rect::from_xywh(cx - rx, cy - ry, 2.0 * rx, 2.0 * ry) {
            self.llenar(PathBuilder::from_oval(r), c)
        }
    }
    fn linea(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, ancho: f32, c: C) {
        if dist((x1, y1), (x2, y2)) < 0.5 {
            self.circulo(x1, y1, ancho / 2.0, c);
            return;
        }
        let mut pb = PathBuilder::new();
        pb.move_to(x1, y1);
        pb.line_to(x2, y2);
        self.delinear(pb.finish(), ancho, c)
    }
    fn polilinea(&mut self, pts: &[(f32, f32)], ancho: f32, c: C) {
        if pts.len() < 2 {
            return;
        }
        let mut pb = PathBuilder::new();
        pb.move_to(pts[0].0, pts[0].1);
        for p in &pts[1..] {
            pb.line_to(p.0, p.1);
        }
        self.delinear(pb.finish(), ancho, c)
    }
    fn poligono(&mut self, pts: &[(f32, f32)], c: C) {
        self.llenar(ruta_poligono(pts), c)
    }
    fn estrella(&mut self, cx: f32, cy: f32, r: f32, rot: f32, c: C) {
        self.llenar(ruta_estrella(cx, cy, r, r * 0.45, 5, rot), c)
    }
    fn corazon(&mut self, cx: f32, cy: f32, s: f32, c: C) {
        self.llenar(ruta_corazon(cx, cy, s), c)
    }
    /// Hoja (lente) entre dos puntos.
    fn hoja(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, ancho: f32, c: C) {
        let (dx, dy) = (x2 - x1, y2 - y1);
        let l = (dx * dx + dy * dy).sqrt().max(0.001);
        let (nx, ny) = (-dy / l, dx / l);
        let mut pts = Vec::with_capacity(20);
        for i in 0..=9 {
            let t = i as f32 / 9.0;
            let w = (t * std::f32::consts::PI).sin() * ancho;
            pts.push((x1 + dx * t + nx * w, y1 + dy * t + ny * w));
        }
        for i in (1..9).rev() {
            let t = i as f32 / 9.0;
            let w = (t * std::f32::consts::PI).sin() * ancho;
            pts.push((x1 + dx * t - nx * w, y1 + dy * t - ny * w));
        }
        self.poligono(&pts, c)
    }
    fn altavoz(&mut self, cx: f32, cy: f32, s: f32, c: C) {
        let u = s / 10.0;
        self.poligono(
            &[
                (cx - 4.0 * u, cy - 2.0 * u),
                (cx - 1.5 * u, cy - 2.0 * u),
                (cx + 1.5 * u, cy - 4.5 * u),
                (cx + 1.5 * u, cy + 4.5 * u),
                (cx - 1.5 * u, cy + 2.0 * u),
                (cx - 4.0 * u, cy + 2.0 * u),
            ],
            c,
        );
        for k in 0..2 {
            let r = (3.5 + 2.2 * k as f32) * u;
            let pts: Vec<(f32, f32)> = (0..=8)
                .map(|i| {
                    let a = -0.8 + 1.6 * i as f32 / 8.0;
                    (cx + 1.0 * u + a.cos() * r, cy + a.sin() * r)
                })
                .collect();
            self.polilinea(&pts, 1.1 * u, c);
        }
    }
}

impl Fig for Pixmap {
    fn llenar(&mut self, ruta: Option<Path>, c: C) {
        if let Some(p) = ruta {
            self.fill_path(&p, &pintura(c), FillRule::Winding, Transform::identity(), None);
        }
    }
    fn delinear(&mut self, ruta: Option<Path>, ancho: f32, c: C) {
        if let Some(p) = ruta {
            self.stroke_path(&p, &pintura(c), &trazo(ancho), Transform::identity(), None);
        }
    }
    fn degradado_v(&mut self, x: f32, y: f32, w: f32, h: f32, c1: C, c2: C) {
        let col = |c: C| Color::from_rgba8(c[0], c[1], c[2], c[3]);
        let shader = LinearGradient::new(
            Point::from_xy(x, y),
            Point::from_xy(x, y + h),
            vec![GradientStop::new(0.0, col(c1)), GradientStop::new(1.0, col(c2))],
            SpreadMode::Pad,
            Transform::identity(),
        );
        if let (Some(shader), Some(r)) = (shader, Rect::from_xywh(x, y, w, h)) {
            let paint = Paint { shader, anti_alias: false, ..Default::default() };
            self.fill_rect(r, &paint, Transform::identity(), None);
        }
    }
}

/// Copia opaca (sin mezcla) de `src` dentro de `dst` en (x, y).
pub fn copiar_opaco(dst: &mut Pixmap, src: &Pixmap, x: i32, y: i32) {
    let (dw, dh) = (dst.width() as i32, dst.height() as i32);
    let (sw, sh) = (src.width() as i32, src.height() as i32);
    let x0 = x.max(0);
    let x1 = (x + sw).min(dw);
    if x1 <= x0 {
        return;
    }
    let s = src.data();
    let d = dst.data_mut();
    for sy in 0..sh {
        let dy = y + sy;
        if dy < 0 || dy >= dh {
            continue;
        }
        let so = ((sy * sw + (x0 - x)) * 4) as usize;
        let doff = ((dy * dw + x0) * 4) as usize;
        let n = ((x1 - x0) * 4) as usize;
        d[doff..doff + n].copy_from_slice(&s[so..so + n]);
    }
}

const FUENTE: &[u8] = include_bytes!("../recursos/Andika-Bold.ttf");

type Glifo = (fontdue::Metrics, Vec<u8>);

pub struct Lienzo {
    pub pix: Pixmap,
    fuente: fontdue::Font,
    glifos: HashMap<(char, u32), Glifo>,
}

impl Deref for Lienzo {
    type Target = Pixmap;
    fn deref(&self) -> &Pixmap {
        &self.pix
    }
}

impl DerefMut for Lienzo {
    fn deref_mut(&mut self) -> &mut Pixmap {
        &mut self.pix
    }
}

fn mezclar_glifo(pix: &mut Pixmap, x: f32, y: f32, m: &fontdue::Metrics, bmp: &[u8], c: C) {
    let (pw, ph) = (pix.width() as i32, pix.height() as i32);
    let xi = x.round() as i32;
    let yi = y.round() as i32;
    let d = pix.data_mut();
    for row in 0..m.height as i32 {
        let py = yi + row;
        if py < 0 || py >= ph {
            continue;
        }
        for col in 0..m.width as i32 {
            let px = xi + col;
            if px < 0 || px >= pw {
                continue;
            }
            let cov = bmp[(row * m.width as i32 + col) as usize] as u32;
            if cov == 0 {
                continue;
            }
            let a = cov * c[3] as u32 / 255;
            if a == 0 {
                continue;
            }
            let inv = 255 - a;
            let i = ((py * pw + px) * 4) as usize;
            for k in 0..3 {
                d[i + k] = ((c[k] as u32 * a + d[i + k] as u32 * inv) / 255) as u8;
            }
            d[i + 3] = (a + d[i + 3] as u32 * inv / 255) as u8;
        }
    }
}

impl Lienzo {
    pub fn nuevo(w: u32, h: u32) -> Lienzo {
        let fuente = fontdue::Font::from_bytes(FUENTE, fontdue::FontSettings::default())
            .expect("fuente Andika");
        Lienzo { pix: Pixmap::new(w, h).expect("pixmap"), fuente, glifos: HashMap::new() }
    }

    /// Dibuja sobre otro pixmap usando la misma fuente y caché.
    pub fn con<F: FnOnce(&mut Lienzo)>(&mut self, otro: &mut Pixmap, f: F) {
        std::mem::swap(&mut self.pix, otro);
        f(self);
        std::mem::swap(&mut self.pix, otro);
    }

    pub fn metricas(&self, tam: f32) -> (f32, f32) {
        match self.fuente.horizontal_line_metrics(tam) {
            Some(lm) => (lm.ascent, lm.descent),
            None => (tam * 0.9, -tam * 0.25),
        }
    }

    pub fn medir(&mut self, s: &str, tam: f32) -> f32 {
        let t = tam.round().max(1.0) as u32;
        let fuente = &self.fuente;
        let mut w = 0.0;
        for ch in s.chars() {
            let g = self.glifos.entry((ch, t)).or_insert_with(|| fuente.rasterize(ch, t as f32));
            w += g.0.advance_width;
        }
        w
    }

    /// Texto con esquina superior izquierda en (x, y). Devuelve el ancho.
    pub fn texto(&mut self, s: &str, x: f32, y: f32, tam: f32, c: C) -> f32 {
        let t = tam.round().max(1.0) as u32;
        let (asc, _) = self.metricas(t as f32);
        let base = y + asc;
        let mut pen = x;
        let fuente = &self.fuente;
        for ch in s.chars() {
            let g = self.glifos.entry((ch, t)).or_insert_with(|| fuente.rasterize(ch, t as f32));
            let gx = pen + g.0.xmin as f32;
            let gy = base - (g.0.height as i32 + g.0.ymin) as f32;
            mezclar_glifo(&mut self.pix, gx, gy, &g.0, &g.1, c);
            pen += g.0.advance_width;
        }
        pen - x
    }

    /// Alto de caja de una línea (ascenso - descenso).
    pub fn alto(&self, tam: f32) -> f32 {
        let (a, d) = self.metricas(tam.round());
        a - d
    }

    pub fn texto_c(&mut self, s: &str, cx: f32, cy: f32, tam: f32, c: C) {
        let w = self.medir(s, tam);
        let h = self.alto(tam);
        self.texto(s, cx - w / 2.0, cy - h / 2.0, tam, c);
    }

    pub fn texto_c_borde(&mut self, s: &str, cx: f32, cy: f32, tam: f32, c: C, borde: C, g: f32) {
        let w = self.medir(s, tam);
        let h = self.alto(tam);
        let (x, y) = (cx - w / 2.0, cy - h / 2.0);
        for (dx, dy) in [(-1.0, -1.0), (0.0, -1.0), (1.0, -1.0), (-1.0, 0.0), (1.0, 0.0), (-1.0, 1.0), (0.0, 1.0), (1.0, 1.0)] {
            self.texto(s, x + dx * g, y + dy * g, tam, borde);
        }
        self.texto(s, x, y, tam, c);
    }
}
