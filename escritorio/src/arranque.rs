//! Imágenes de la pantalla de carga (tema de Plymouth "nidito"), dibujadas con el mismo código
//! que el escritorio: cielo, cerros, los gatitos de la casa y el nombre en colores.
//!
//! `nidito --arranque <carpeta>` → fondo.png, logo.png y punto.png

use crate::gfx::*;
use crate::{H, W};
use tiny_skia::Pixmap;

pub fn generar(dir: &str) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut l = Lienzo::nuevo(W, H);
    let (w, h) = (W as f32, H as f32);

    // fondo: el mismo paisaje del escritorio, sin íconos
    l.degradado_v(0.0, 0.0, w, h, rgb(110, 190, 240), rgb(200, 235, 255));
    for &(x, y, s) in &[(140.0, 90.0, 1.0), (760.0, 70.0, 0.8), (40.0, 250.0, 0.6)] {
        for &(dx, dy, r) in &[(0.0, 0.0, 30.0), (34.0, -14.0, 38.0), (70.0, 0.0, 30.0), (34.0, 8.0, 28.0)] {
            l.circulo(x + dx * s, y + dy * s, r * s, alfa(BLANCO, 0.9));
        }
    }
    l.ovalo(300.0, h + 40.0, 520.0, 200.0, rgb(120, 200, 110));
    l.ovalo(850.0, h + 60.0, 420.0, 230.0, rgb(90, 180, 95));
    crate::gatitos::Manada::nueva().dibujar(&mut l);
    l.pix.save_png(format!("{dir}/fondo.png")).map_err(|e| e.to_string())?;

    // logo: "Nidito" con cada letra de un color
    let tam = 120.0;
    let ancho = l.medir("Nidito", tam) + 40.0;
    let alto = l.alto(tam) + 30.0;
    let mut logo = Pixmap::new(ancho.ceil() as u32, alto.ceil() as u32).ok_or("logo")?;
    l.con(&mut logo, |l| {
        let mut x = 20.0;
        let colores = [ROJO, NARANJO, AMARILLO, VERDE, CELESTE, MORADO];
        for (i, c) in "Nidito".chars().enumerate() {
            let s = c.to_string();
            let cw = l.medir(&s, tam);
            for (dx, dy) in [(-4.0, 0.0), (4.0, 0.0), (0.0, -4.0), (0.0, 4.0), (-3.0, -3.0), (3.0, -3.0), (-3.0, 3.0), (3.0, 3.0), (0.0, 7.0)] {
                l.texto(&s, x + dx, 12.0 + dy, tam, TINTA);
            }
            l.texto(&s, x, 12.0, tam, colores[i % colores.len()]);
            x += cw;
        }
    });
    logo.save_png(format!("{dir}/logo.png")).map_err(|e| e.to_string())?;

    // punto de "cargando"
    let mut punto = Pixmap::new(28, 28).ok_or("punto")?;
    punto.circulo(14.0, 14.0, 12.0, TINTA);
    punto.circulo(14.0, 14.0, 9.0, BLANCO);
    punto.save_png(format!("{dir}/punto.png")).map_err(|e| e.to_string())?;
    Ok(())
}
