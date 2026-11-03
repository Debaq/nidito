//! Objetos dibujados con figuras (sin imágenes externas).

use crate::gfx::*;
use std::f32::consts::PI;
use tiny_skia::Pixmap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Obj {
    Manzana,
    Estrella,
    Corazon,
    Pelota,
    Flor,
    Pez,
    Galleta,
    Sol,
    Huevo,
    Harina,
    Leche,
    Azucar,
    Mantequilla,
    Platano,
    Frutilla,
    Uva,
    Naranja,
    Tomate,
    Queso,
    Chocolate,
    Gato,
    Perro,
    Pajaro,
    Casa,
    Auto,
    Arbol,
}

pub const TODOS: [Obj; 26] = [
    Obj::Manzana, Obj::Estrella, Obj::Corazon, Obj::Pelota, Obj::Flor, Obj::Pez, Obj::Galleta, Obj::Sol,
    Obj::Huevo, Obj::Harina, Obj::Leche, Obj::Azucar, Obj::Mantequilla, Obj::Platano, Obj::Frutilla, Obj::Uva,
    Obj::Naranja, Obj::Tomate, Obj::Queso, Obj::Chocolate, Obj::Gato, Obj::Perro, Obj::Pajaro, Obj::Casa,
    Obj::Auto, Obj::Arbol,
];

/// Vocabulario de la app English.
pub const VOCABULARIO: [Obj; 18] = [
    Obj::Gato,
    Obj::Perro,
    Obj::Pajaro,
    Obj::Pez,
    Obj::Manzana,
    Obj::Platano,
    Obj::Naranja,
    Obj::Frutilla,
    Obj::Huevo,
    Obj::Galleta,
    Obj::Casa,
    Obj::Auto,
    Obj::Arbol,
    Obj::Flor,
    Obj::Sol,
    Obj::Estrella,
    Obj::Pelota,
    Obj::Corazon,
];

/// Objetos para contar y memorice.
pub const CONTABLES: [Obj; 8] = [
    Obj::Manzana,
    Obj::Estrella,
    Obj::Corazon,
    Obj::Pelota,
    Obj::Flor,
    Obj::Pez,
    Obj::Galleta,
    Obj::Sol,
];

impl Obj {
    /// (singular, plural, femenino)
    pub fn nombres(self) -> (&'static str, &'static str, bool) {
        use Obj::*;
        match self {
            Manzana => ("manzana", "manzanas", true),
            Estrella => ("estrella", "estrellas", true),
            Corazon => ("corazón", "corazones", false),
            Pelota => ("pelota", "pelotas", true),
            Flor => ("flor", "flores", true),
            Pez => ("pez", "peces", false),
            Galleta => ("galleta", "galletas", true),
            Sol => ("sol", "soles", false),
            Huevo => ("huevo", "huevos", false),
            Harina => ("taza de harina", "tazas de harina", true),
            Leche => ("vaso de leche", "vasos de leche", false),
            Azucar => ("cucharada de azúcar", "cucharadas de azúcar", true),
            Mantequilla => ("trozo de mantequilla", "trozos de mantequilla", false),
            Platano => ("plátano", "plátanos", false),
            Frutilla => ("frutilla", "frutillas", true),
            Uva => ("uva", "uvas", true),
            Naranja => ("naranja", "naranjas", true),
            Tomate => ("tomate", "tomates", false),
            Queso => ("queso", "quesos", false),
            Chocolate => ("chocolate", "chocolates", false),
            Gato => ("gato", "gatos", false),
            Perro => ("perro", "perros", false),
            Pajaro => ("pájaro", "pájaros", false),
            Casa => ("casa", "casas", true),
            Auto => ("auto", "autos", false),
            Arbol => ("árbol", "árboles", false),
        }
    }
    pub fn ingles(self) -> &'static str {
        use Obj::*;
        match self {
            Manzana => "apple",
            Estrella => "star",
            Corazon => "heart",
            Pelota => "ball",
            Flor => "flower",
            Pez => "fish",
            Galleta => "cookie",
            Sol => "sun",
            Huevo => "egg",
            Harina => "flour",
            Leche => "milk",
            Azucar => "sugar",
            Mantequilla => "butter",
            Platano => "banana",
            Frutilla => "strawberry",
            Uva => "grapes",
            Naranja => "orange",
            Tomate => "tomato",
            Queso => "cheese",
            Chocolate => "chocolate",
            Gato => "cat",
            Perro => "dog",
            Pajaro => "bird",
            Casa => "house",
            Auto => "car",
            Arbol => "tree",
        }
    }
    /// Inglés con medida: "cup of flour", "egg"
    pub fn ingles_unidad(self) -> &'static str {
        use Obj::*;
        match self {
            Harina => "cup of flour",
            Leche => "glass of milk",
            Azucar => "spoon of sugar",
            Mantequilla => "piece of butter",
            Queso => "piece of cheese",
            o => o.ingles(),
        }
    }
    pub fn ingles_plural(self) -> String {
        use Obj::*;
        match self {
            Pez => "fish".into(),
            Frutilla => "strawberries".into(),
            Galleta => "cookies".into(),
            Uva => "grapes".into(),
            Tomate => "tomatoes".into(),
            Harina => "cups of flour".into(),
            Leche => "glasses of milk".into(),
            Azucar => "spoons of sugar".into(),
            Mantequilla => "pieces of butter".into(),
            Queso => "pieces of cheese".into(),
            Casa => "houses".into(),
            o => format!("{}s", o.ingles()),
        }
    }
    /// "one egg", "three apples"
    pub fn en_cantidad(self, n: usize) -> String {
        if n == 1 {
            format!("one {}", self.ingles_unidad())
        } else {
            format!("{} {}", crate::apps::num_en(n), self.ingles_plural())
        }
    }
    /// "el gato", "la casa"
    pub fn con_articulo(self) -> String {
        format!("{} {}", if self.fem() { "la" } else { "el" }, self.singular())
    }
    pub fn singular(self) -> &'static str {
        self.nombres().0
    }
    pub fn plural(self) -> &'static str {
        self.nombres().1
    }
    pub fn fem(self) -> bool {
        self.nombres().2
    }
    /// "una manzana", "tres huevos"
    pub fn con_cantidad(self, n: usize) -> String {
        if n == 1 {
            format!("{} {}", if self.fem() { "una" } else { "un" }, self.singular())
        } else {
            format!("{} {}", crate::apps::num(n), self.plural())
        }
    }
}

pub fn dibujar(p: &mut Pixmap, o: Obj, cx: f32, cy: f32, s: f32) {
    use Obj::*;
    match o {
        Manzana => {
            let r = rgb(220, 40, 50);
            p.circulo(cx - 0.14 * s, cy + 0.04 * s, 0.3 * s, r);
            p.circulo(cx + 0.14 * s, cy + 0.04 * s, 0.3 * s, r);
            p.circulo(cx, cy + 0.12 * s, 0.3 * s, r);
            p.ovalo(cx - 0.18 * s, cy - 0.04 * s, 0.07 * s, 0.11 * s, alfa(BLANCO, 0.45));
            p.linea(cx, cy - 0.2 * s, cx + 0.04 * s, cy - 0.42 * s, 0.06 * s, CAFE);
            p.hoja(cx + 0.04 * s, cy - 0.33 * s, cx + 0.32 * s, cy - 0.45 * s, 0.07 * s, VERDE);
        }
        Estrella => {
            p.estrella(cx, cy + 0.03 * s, 0.53 * s, -PI / 2.0, rgb(240, 160, 0));
            p.estrella(cx, cy + 0.03 * s, 0.45 * s, -PI / 2.0, AMARILLO);
            p.circulo(cx - 0.1 * s, cy - 0.05 * s, 0.05 * s, alfa(BLANCO, 0.7));
        }
        Corazon => {
            p.corazon(cx, cy + 0.02 * s, 1.0 * s, rgb(230, 60, 120));
            p.corazon(cx, cy, 0.86 * s, ROSADO);
            p.ovalo(cx - 0.22 * s, cy - 0.17 * s, 0.07 * s, 0.05 * s, alfa(BLANCO, 0.6));
        }
        Pelota => {
            p.circulo(cx, cy, 0.45 * s, AZUL);
            p.ovalo(cx, cy, 0.17 * s, 0.45 * s, BLANCO);
            p.ovalo(cx, cy, 0.06 * s, 0.45 * s, ROJO);
            p.circulo(cx - 0.22 * s, cy - 0.2 * s, 0.07 * s, alfa(BLANCO, 0.6));
        }
        Flor => {
            p.linea(cx, cy + 0.15 * s, cx, cy + 0.5 * s, 0.07 * s, VERDE);
            for k in 0..6 {
                let a = k as f32 * PI / 3.0;
                p.circulo(cx + a.cos() * 0.24 * s, cy - 0.05 * s + a.sin() * 0.24 * s, 0.17 * s, MORADO);
            }
            p.circulo(cx, cy - 0.05 * s, 0.16 * s, AMARILLO);
        }
        Pez => {
            let c = rgb(255, 130, 40);
            p.poligono(&[(cx + 0.2 * s, cy), (cx + 0.48 * s, cy - 0.22 * s), (cx + 0.48 * s, cy + 0.22 * s)], c);
            p.poligono(&[(cx - 0.12 * s, cy - 0.16 * s), (cx + 0.05 * s, cy - 0.32 * s), (cx + 0.1 * s, cy - 0.12 * s)], c);
            p.ovalo(cx - 0.05 * s, cy, 0.34 * s, 0.22 * s, c);
            p.circulo(cx - 0.22 * s, cy - 0.05 * s, 0.065 * s, BLANCO);
            p.circulo(cx - 0.23 * s, cy - 0.05 * s, 0.035 * s, NEGRO);
        }
        Galleta => {
            p.circulo(cx, cy, 0.45 * s, rgb(200, 140, 70));
            p.circulo(cx, cy, 0.4 * s, rgb(225, 165, 90));
            for (dx, dy) in [(-0.18, -0.15), (0.12, -0.2), (0.2, 0.08), (-0.05, 0.05), (-0.2, 0.2), (0.05, 0.25)] {
                p.circulo(cx + dx * s, cy + dy * s, 0.055 * s, rgb(90, 50, 25));
            }
        }
        Sol => {
            for k in 0..8 {
                let a = k as f32 * PI / 4.0;
                p.linea(cx + a.cos() * 0.33 * s, cy + a.sin() * 0.33 * s, cx + a.cos() * 0.48 * s, cy + a.sin() * 0.48 * s, 0.07 * s, NARANJO);
            }
            p.circulo(cx, cy, 0.3 * s, AMARILLO);
            p.circulo(cx - 0.1 * s, cy - 0.06 * s, 0.035 * s, TINTA);
            p.circulo(cx + 0.1 * s, cy - 0.06 * s, 0.035 * s, TINTA);
            let pts: Vec<(f32, f32)> = (0..=8)
                .map(|i| {
                    let a = 0.4 + (PI - 0.8) * i as f32 / 8.0;
                    (cx + a.cos() * 0.14 * s, cy + 0.02 * s + a.sin() * 0.12 * s)
                })
                .collect();
            p.polilinea(&pts, 0.035 * s, TINTA);
        }
        Huevo => {
            p.ovalo(cx, cy + 0.02 * s, 0.32 * s, 0.42 * s, rgb(215, 200, 175));
            p.ovalo(cx, cy, 0.3 * s, 0.4 * s, rgb(255, 248, 235));
            p.ovalo(cx - 0.1 * s, cy - 0.15 * s, 0.06 * s, 0.1 * s, BLANCO);
        }
        Harina => {
            p.ovalo(cx, cy - 0.14 * s, 0.38 * s, 0.26 * s, rgb(235, 235, 240));
            p.circulo(cx - 0.18 * s, cy - 0.2 * s, 0.13 * s, BLANCO);
            p.circulo(cx + 0.05 * s, cy - 0.3 * s, 0.16 * s, BLANCO);
            p.circulo(cx + 0.22 * s, cy - 0.18 * s, 0.12 * s, BLANCO);
            p.poligono(&[(cx - 0.36 * s, cy - 0.1 * s), (cx + 0.36 * s, cy - 0.1 * s), (cx + 0.26 * s, cy + 0.4 * s), (cx - 0.26 * s, cy + 0.4 * s)], rgb(90, 160, 220));
            p.circulo(cx + 0.36 * s, cy + 0.1 * s, 0.12 * s, rgb(90, 160, 220));
            p.circulo(cx + 0.36 * s, cy + 0.1 * s, 0.06 * s, BLANCO);
        }
        Leche => {
            let vidrio = rgb(200, 220, 235);
            p.poligono(&[(cx - 0.3 * s, cy - 0.45 * s), (cx + 0.3 * s, cy - 0.45 * s), (cx + 0.22 * s, cy + 0.45 * s), (cx - 0.22 * s, cy + 0.45 * s)], vidrio);
            p.poligono(&[(cx - 0.26 * s, cy - 0.25 * s), (cx + 0.26 * s, cy - 0.25 * s), (cx + 0.2 * s, cy + 0.4 * s), (cx - 0.2 * s, cy + 0.4 * s)], BLANCO);
        }
        Azucar => {
            p.linea(cx + 0.1 * s, cy + 0.05 * s, cx + 0.48 * s, cy + 0.38 * s, 0.1 * s, GRIS);
            p.ovalo(cx - 0.1 * s, cy - 0.02 * s, 0.32 * s, 0.22 * s, GRIS);
            p.ovalo(cx - 0.1 * s, cy - 0.1 * s, 0.26 * s, 0.16 * s, BLANCO);
            for (dx, dy) in [(-0.2, -0.12), (0.0, -0.18), (-0.05, -0.05)] {
                p.rect(cx + dx * s, cy + dy * s, 0.05 * s, 0.05 * s, rgb(220, 220, 230));
            }
        }
        Mantequilla => {
            p.ovalo(cx, cy + 0.22 * s, 0.48 * s, 0.16 * s, CELESTE);
            p.rrect(cx - 0.3 * s, cy - 0.2 * s, 0.6 * s, 0.38 * s, 0.06 * s, rgb(255, 225, 110));
            p.rrect(cx - 0.3 * s, cy - 0.2 * s, 0.6 * s, 0.1 * s, 0.05 * s, rgb(255, 240, 160));
        }
        Platano => {
            let pts: Vec<(f32, f32)> = (0..=10)
                .map(|i| {
                    let a = PI * 0.15 + PI * 0.7 * i as f32 / 10.0;
                    (cx + a.cos() * 0.4 * s, cy - 0.25 * s + a.sin() * 0.4 * s)
                })
                .collect();
            p.polilinea(&pts, 0.2 * s, rgb(240, 200, 40));
            p.polilinea(&pts[1..9], 0.1 * s, rgb(255, 230, 90));
            p.circulo(pts[0].0, pts[0].1, 0.05 * s, CAFE);
            p.circulo(pts[10].0, pts[10].1, 0.05 * s, CAFE);
        }
        Frutilla => {
            p.poligono(&[(cx - 0.36 * s, cy - 0.2 * s), (cx + 0.36 * s, cy - 0.2 * s), (cx + 0.06 * s, cy + 0.42 * s), (cx - 0.06 * s, cy + 0.42 * s)], ROJO);
            p.ovalo(cx, cy - 0.12 * s, 0.37 * s, 0.17 * s, ROJO);
            for (dx, dy) in [(-0.2, -0.05), (0.0, 0.0), (0.18, -0.06), (-0.08, 0.15), (0.1, 0.17), (0.0, 0.3)] {
                p.ovalo(cx + dx * s, cy + dy * s, 0.025 * s, 0.035 * s, AMARILLO);
            }
            p.estrella(cx, cy - 0.27 * s, 0.2 * s, -PI / 2.0, VERDE);
        }
        Uva => {
            p.linea(cx, cy - 0.45 * s, cx + 0.05 * s, cy - 0.3 * s, 0.05 * s, CAFE);
            let c = rgb(130, 60, 170);
            for (dx, dy) in [(-0.2, -0.22), (0.0, -0.22), (0.2, -0.22), (-0.1, -0.02), (0.1, -0.02), (0.0, 0.18), (-0.2, 0.15), (0.2, 0.15), (0.0, 0.36)] {
                p.circulo(cx + dx * s, cy + dy * s, 0.12 * s, c);
                p.circulo(cx + (dx - 0.04) * s, cy + (dy - 0.04) * s, 0.03 * s, alfa(BLANCO, 0.5));
            }
        }
        Naranja => {
            p.circulo(cx, cy + 0.04 * s, 0.42 * s, rgb(255, 150, 20));
            p.circulo(cx - 0.14 * s, cy - 0.1 * s, 0.07 * s, alfa(BLANCO, 0.4));
            p.hoja(cx, cy - 0.36 * s, cx + 0.25 * s, cy - 0.48 * s, 0.07 * s, VERDE);
        }
        Tomate => {
            p.circulo(cx, cy + 0.05 * s, 0.42 * s, rgb(235, 50, 40));
            p.estrella(cx, cy - 0.32 * s, 0.17 * s, -PI / 2.0, VERDE);
            p.circulo(cx - 0.16 * s, cy - 0.08 * s, 0.07 * s, alfa(BLANCO, 0.4));
        }
        Queso => {
            let c = rgb(255, 205, 60);
            p.poligono(&[(cx - 0.45 * s, cy + 0.3 * s), (cx + 0.45 * s, cy + 0.3 * s), (cx + 0.45 * s, cy - 0.1 * s), (cx - 0.45 * s, cy + 0.0 * s)], c);
            p.poligono(&[(cx - 0.45 * s, cy), (cx + 0.45 * s, cy - 0.1 * s), (cx + 0.2 * s, cy - 0.35 * s)], rgb(255, 225, 120));
            for (dx, dy, r) in [(-0.2, 0.14, 0.07), (0.15, 0.1, 0.06), (0.3, 0.22, 0.04)] {
                p.circulo(cx + dx * s, cy + dy * s, r * s, rgb(230, 170, 30));
            }
        }
        Chocolate => {
            p.rrect(cx - 0.3 * s, cy - 0.42 * s, 0.6 * s, 0.84 * s, 0.06 * s, rgb(100, 55, 30));
            for i in 0..2 {
                for j in 0..3 {
                    p.rrect(cx - 0.25 * s + i as f32 * 0.26 * s, cy - 0.37 * s + j as f32 * 0.27 * s, 0.22 * s, 0.22 * s, 0.03 * s, rgb(135, 80, 45));
                }
            }
        }
        Gato => {
            let c = rgb(255, 170, 70);
            p.poligono(&[(cx - 0.38 * s, cy - 0.05 * s), (cx - 0.34 * s, cy - 0.48 * s), (cx - 0.08 * s, cy - 0.3 * s)], c);
            p.poligono(&[(cx + 0.38 * s, cy - 0.05 * s), (cx + 0.34 * s, cy - 0.48 * s), (cx + 0.08 * s, cy - 0.3 * s)], c);
            p.poligono(&[(cx - 0.31 * s, cy - 0.12 * s), (cx - 0.3 * s, cy - 0.38 * s), (cx - 0.14 * s, cy - 0.27 * s)], ROSADO);
            p.poligono(&[(cx + 0.31 * s, cy - 0.12 * s), (cx + 0.3 * s, cy - 0.38 * s), (cx + 0.14 * s, cy - 0.27 * s)], ROSADO);
            p.ovalo(cx, cy + 0.05 * s, 0.4 * s, 0.34 * s, c);
            for k in [-1.0, 1.0] {
                p.ovalo(cx + k * 0.15 * s, cy - 0.02 * s, 0.07 * s, 0.09 * s, rgb(80, 170, 60));
                p.ovalo(cx + k * 0.15 * s, cy - 0.02 * s, 0.025 * s, 0.07 * s, NEGRO);
                p.linea(cx + k * 0.12 * s, cy + 0.15 * s, cx + k * 0.46 * s, cy + 0.1 * s, 0.015 * s, TINTA);
                p.linea(cx + k * 0.12 * s, cy + 0.18 * s, cx + k * 0.46 * s, cy + 0.22 * s, 0.015 * s, TINTA);
            }
            p.poligono(&[(cx - 0.05 * s, cy + 0.1 * s), (cx + 0.05 * s, cy + 0.1 * s), (cx, cy + 0.16 * s)], ROSADO);
        }
        Perro => {
            let c = rgb(200, 145, 90);
            let oreja = rgb(120, 75, 40);
            p.ovalo(cx - 0.36 * s, cy, 0.12 * s, 0.28 * s, oreja);
            p.ovalo(cx + 0.36 * s, cy, 0.12 * s, 0.28 * s, oreja);
            p.ovalo(cx, cy, 0.32 * s, 0.38 * s, c);
            p.ovalo(cx, cy + 0.17 * s, 0.2 * s, 0.15 * s, rgb(240, 215, 180));
            p.ovalo(cx, cy + 0.36 * s, 0.07 * s, 0.08 * s, ROSADO);
            p.ovalo(cx, cy + 0.12 * s, 0.09 * s, 0.06 * s, NEGRO);
            p.circulo(cx - 0.13 * s, cy - 0.1 * s, 0.05 * s, NEGRO);
            p.circulo(cx + 0.13 * s, cy - 0.1 * s, 0.05 * s, NEGRO);
            p.circulo(cx - 0.14 * s, cy - 0.12 * s, 0.015 * s, BLANCO);
            p.circulo(cx + 0.12 * s, cy - 0.12 * s, 0.015 * s, BLANCO);
        }
        Pajaro => {
            let c = rgb(70, 160, 240);
            p.poligono(&[(cx + 0.25 * s, cy + 0.05 * s), (cx + 0.5 * s, cy - 0.1 * s), (cx + 0.48 * s, cy + 0.15 * s)], AZUL);
            p.circulo(cx, cy + 0.05 * s, 0.32 * s, c);
            p.circulo(cx - 0.2 * s, cy - 0.2 * s, 0.2 * s, c);
            p.ovalo(cx + 0.05 * s, cy + 0.08 * s, 0.18 * s, 0.11 * s, AZUL);
            p.ovalo(cx - 0.05 * s, cy + 0.2 * s, 0.18 * s, 0.1 * s, AMARILLO);
            p.poligono(&[(cx - 0.36 * s, cy - 0.24 * s), (cx - 0.5 * s, cy - 0.17 * s), (cx - 0.36 * s, cy - 0.12 * s)], NARANJO);
            p.circulo(cx - 0.24 * s, cy - 0.24 * s, 0.04 * s, NEGRO);
            p.linea(cx - 0.05 * s, cy + 0.36 * s, cx - 0.08 * s, cy + 0.48 * s, 0.03 * s, NARANJO);
            p.linea(cx + 0.08 * s, cy + 0.36 * s, cx + 0.1 * s, cy + 0.48 * s, 0.03 * s, NARANJO);
        }
        Casa => {
            p.rect(cx - 0.34 * s, cy - 0.08 * s, 0.68 * s, 0.52 * s, rgb(255, 225, 150));
            p.poligono(&[(cx - 0.46 * s, cy - 0.06 * s), (cx, cy - 0.46 * s), (cx + 0.46 * s, cy - 0.06 * s)], ROJO);
            p.rrect(cx - 0.09 * s, cy + 0.14 * s, 0.18 * s, 0.3 * s, 0.03 * s, CAFE);
            p.rect(cx + 0.14 * s, cy + 0.02 * s, 0.14 * s, 0.14 * s, CELESTE);
            p.rect(cx - 0.28 * s, cy + 0.02 * s, 0.14 * s, 0.14 * s, CELESTE);
        }
        Auto => {
            p.rrect(cx - 0.24 * s, cy - 0.3 * s, 0.44 * s, 0.26 * s, 0.08 * s, ROJO);
            p.rrect(cx - 0.18 * s, cy - 0.25 * s, 0.15 * s, 0.17 * s, 0.03 * s, CELESTE);
            p.rrect(cx + 0.0 * s, cy - 0.25 * s, 0.15 * s, 0.17 * s, 0.03 * s, CELESTE);
            p.rrect(cx - 0.48 * s, cy - 0.1 * s, 0.96 * s, 0.3 * s, 0.1 * s, ROJO);
            p.circulo(cx + 0.42 * s, cy - 0.02 * s, 0.04 * s, AMARILLO);
            for k in [-1.0, 1.0] {
                p.circulo(cx + k * 0.27 * s, cy + 0.2 * s, 0.13 * s, TINTA);
                p.circulo(cx + k * 0.27 * s, cy + 0.2 * s, 0.06 * s, GRIS);
            }
        }
        Arbol => {
            p.rect(cx - 0.07 * s, cy + 0.05 * s, 0.14 * s, 0.42 * s, CAFE);
            let v = rgb(60, 170, 70);
            p.circulo(cx, cy - 0.2 * s, 0.27 * s, v);
            p.circulo(cx - 0.22 * s, cy - 0.02 * s, 0.22 * s, v);
            p.circulo(cx + 0.22 * s, cy - 0.02 * s, 0.22 * s, v);
            p.circulo(cx, cy + 0.04 * s, 0.22 * s, v);
            p.circulo(cx - 0.1 * s, cy - 0.25 * s, 0.07 * s, alfa(BLANCO, 0.25));
            p.circulo(cx + 0.15 * s, cy - 0.05 * s, 0.05 * s, ROJO);
            p.circulo(cx - 0.18 * s, cy + 0.05 * s, 0.05 * s, ROJO);
        }
    }
}
