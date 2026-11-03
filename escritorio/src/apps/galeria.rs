//! Mis dibujos: carpeta con los dibujos guardados desde Pintar.

use super::*;
use crate::cx::Son;
use std::path::PathBuf;
use tiny_skia::{FilterQuality, Pixmap, PixmapPaint, Transform};

pub struct Galeria {
    archivos: Vec<PathBuf>,
    i: usize,
    img: Option<Pixmap>,
    cargado: Option<usize>,
}

const MARCO: R = R::new(100.0, 24.0, 800.0, 420.0);

pub fn frases(f: &mut Frases) {
    f.bil("Todavía no hay dibujos. ¡Pinta uno!", "No drawings yet. Paint one!");
}

impl Galeria {
    pub fn nuevo(cx: &mut Cx) -> Box<dyn App> {
        let mut archivos: Vec<PathBuf> = std::fs::read_dir(ruta_datos("Dibujos"))
            .map(|rd| rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "png")).collect())
            .unwrap_or_default();
        archivos.sort();
        archivos.reverse();
        if archivos.is_empty() {
            cx.decir_bil("Todavía no hay dibujos. ¡Pinta uno!", "No drawings yet. Paint one!");
        }
        Box::new(Galeria { archivos, i: 0, img: None, cargado: None })
    }
}

impl App for Galeria {
    fn animando(&self) -> bool {
        false
    }

    fn evento(&mut self, ev: &Ev, cx: &mut Cx) {
        let n = self.archivos.len();
        if n == 0 {
            return;
        }
        let d = match ev {
            Ev::Clic(x, y) if dist((*x, *y), (50.0, 234.0)) < 36.0 => -1,
            Ev::Clic(x, y) if dist((*x, *y), (950.0, 234.0)) < 36.0 => 1,
            Ev::Tecla(T::Izq) => -1,
            Ev::Tecla(T::Der) | Ev::Tecla(T::Espacio) => 1,
            _ => 0,
        };
        if d != 0 {
            self.i = ((self.i as i32 + d).rem_euclid(n as i32)) as usize;
            cx.sonar(Son::Whoosh);
        }
    }

    fn dibujar(&mut self, l: &mut Lienzo, cx: &Cx) {
        l.degradado_v(0.0, 0.0, AW, AH, rgb(70, 76, 96), rgb(45, 48, 64));
        if self.archivos.is_empty() {
            // carpeta vacía
            l.rrect(380.0, 130.0, 240.0, 170.0, 18.0, rgb(240, 190, 70));
            l.rrect(380.0, 110.0, 110.0, 50.0, 14.0, rgb(240, 190, 70));
            l.rrect(380.0, 150.0, 240.0, 150.0, 18.0, rgb(255, 210, 100));
            l.texto_c("Todavía no hay dibujos", AW / 2.0, 360.0, 40.0, BLANCO);
            l.texto_c("¡Pinta uno!", AW / 2.0, 410.0, 34.0, AMARILLO);
            return;
        }
        if self.cargado != Some(self.i) {
            self.img = Pixmap::load_png(&self.archivos[self.i]).ok();
            self.cargado = Some(self.i);
        }
        if let Some(img) = &self.img {
            let esc = (MARCO.w / img.width() as f32).min(MARCO.h / img.height() as f32);
            let (w, h) = (img.width() as f32 * esc, img.height() as f32 * esc);
            let (x, y) = (MARCO.cx() - w / 2.0, MARCO.cy() - h / 2.0);
            l.rect(x + 6.0, y + 8.0, w, h, alfa(NEGRO, 0.35));
            l.rect(x - 8.0, y - 8.0, w + 16.0, h + 16.0, BLANCO);
            let pp = PixmapPaint { quality: FilterQuality::Bilinear, ..Default::default() };
            l.pix.draw_pixmap(0, 0, img.as_ref(), &pp, Transform::from_row(esc, 0.0, 0.0, esc, x, y), None);
        }
        boton_flecha(l, 50.0, 234.0, 34.0, -1.0, NARANJO, cx);
        boton_flecha(l, 950.0, 234.0, 34.0, 1.0, NARANJO, cx);
        l.texto_c(&format!("{} / {}", self.i + 1, self.archivos.len()), AW / 2.0, 482.0, 30.0, BLANCO);
    }
}
