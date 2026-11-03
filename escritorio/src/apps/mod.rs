//! Apps de Nidito. Cada app dibuja en su propio pixmap de AW x AH (coordenadas locales).

pub mod clase;
pub mod cocina;
pub mod contar;
pub mod english;
pub mod escribir;
pub mod galeria;
pub mod globos;
pub mod letras;
pub mod memorice;
pub mod pintar;
pub mod teclas;

use crate::cx::Cx;
use crate::gfx::*;

pub const AW: f32 = 1000.0;
pub const AH: f32 = 518.0;

#[derive(Clone, Debug)]
pub enum Ev {
    Clic(f32, f32),
    Suelta(f32, f32),
    Mueve(f32, f32),
    Texto(String),
    Tecla(T),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum T {
    Enter,
    Borrar,
    Espacio,
    Escape,
    Izq,
    Der,
    Otra,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cursor {
    Normal,
    Laser,
    Cuchara,
    Ninguno,
}

pub trait App {
    fn evento(&mut self, ev: &Ev, cx: &mut Cx);
    fn actualizar(&mut self, _dt: f32, _cx: &mut Cx) {}
    fn dibujar(&mut self, l: &mut Lienzo, cx: &Cx);
    fn cerrar(&mut self, _cx: &mut Cx) {}
    fn cursor(&self) -> Cursor {
        Cursor::Normal
    }
    /// true mientras haya algo moviéndose (si no, se redibuja solo ante eventos).
    fn animando(&self) -> bool {
        true
    }
}

pub fn num(n: usize) -> &'static str {
    const N: [&str; 21] = [
        "cero", "uno", "dos", "tres", "cuatro", "cinco", "seis", "siete", "ocho", "nueve", "diez", "once", "doce", "trece",
        "catorce", "quince", "dieciséis", "diecisiete", "dieciocho", "diecinueve", "veinte",
    ];
    N.get(n).copied().unwrap_or("muchos")
}

pub fn num_en(n: usize) -> &'static str {
    const N: [&str; 21] = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve",
        "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen", "twenty",
    ];
    N.get(n).copied().unwrap_or("many")
}

/// Nombre de la letra para que la voz la diga.
pub fn nombre_letra(c: char) -> Option<String> {
    let c = c.to_lowercase().next()?;
    let s = match c {
        'a' | 'á' => "a",
        'b' => "be",
        'c' => "ce",
        'd' => "de",
        'e' | 'é' => "e",
        'f' => "efe",
        'g' => "ge",
        'h' => "hache",
        'i' | 'í' => "i",
        'j' => "jota",
        'k' => "ka",
        'l' => "ele",
        'm' => "eme",
        'n' => "ene",
        'ñ' => "eñe",
        'o' | 'ó' => "o",
        'p' => "pe",
        'q' => "cu",
        'r' => "erre",
        's' => "ese",
        't' => "te",
        'u' | 'ú' | 'ü' => "u",
        'v' => "ve",
        'w' => "doble ve",
        'x' => "equis",
        'y' => "ye",
        'z' => "zeta",
        d if d.is_ascii_digit() => num(d.to_digit(10)? as usize),
        _ => return None,
    };
    Some(s.to_string())
}

/// Letra base para comparar (minúscula, sin tilde).
pub fn base(c: char) -> char {
    let c = c.to_lowercase().next().unwrap_or(c);
    match c {
        'á' | 'à' => 'a',
        'é' | 'è' => 'e',
        'í' | 'ì' => 'i',
        'ó' | 'ò' => 'o',
        'ú' | 'ù' | 'ü' => 'u',
        _ => c,
    }
}

pub fn mayus(c: char) -> char {
    c.to_uppercase().next().unwrap_or(c)
}

pub const ALABANZAS: [&str; 5] = ["¡Muy bien!", "¡Excelente!", "¡Bravo!", "¡Lo lograste!", "¡Genial!"];
pub const ALABANZAS_EN: [&str; 5] = ["Very good!", "Excellent!", "Bravo!", "You did it!", "Awesome!"];

/// Elige una alabanza: (español, inglés).
pub fn alabanza(cx: &mut Cx) -> (&'static str, &'static str) {
    let i = cx.rng.n(ALABANZAS.len());
    (ALABANZAS[i], ALABANZAS_EN[i])
}

/// Catálogo de frases que se pregraban.
#[derive(Default)]
pub struct Frases(pub Vec<crate::voz::Parte>);

impl Frases {
    pub fn es<S: Into<String>>(&mut self, t: S) {
        self.0.push((crate::voz::L::Es, t.into()));
    }
    pub fn en<S: Into<String>>(&mut self, t: S) {
        self.0.push((crate::voz::L::En, t.into()));
    }
    pub fn bil<A: Into<String>, B: Into<String>>(&mut self, es: A, en: B) {
        self.es(es);
        self.en(en);
    }
}

/// Todas las frases de Nidito (para generar el paquete de voces).
pub fn catalogo(nombre: &str, apodo: &str) -> Vec<crate::voz::Parte> {
    use crate::dibujos::Obj;
    let mut f = Frases::default();
    for n in 0..=20 {
        f.bil(num(n), num_en(n));
    }
    for c in "abcdefghijklmnñopqrstuvwxyz".chars() {
        if let Some(n) = nombre_letra(c) {
            f.es(n);
        }
        f.en(mayus(c).to_string());
    }
    for (a, b) in ALABANZAS.iter().zip(ALABANZAS_EN.iter()) {
        f.bil(*a, *b);
    }
    for o in crate::dibujos::TODOS {
        let o: Obj = o;
        f.bil(o.singular(), o.ingles());
    }
    let llamado = if apodo.trim().is_empty() { nombre.trim() } else { apodo.trim() };
    crate::escritorio::frases(&mut f, llamado);
    crate::tiempo::frases(&mut f, llamado);
    contar::frases(&mut f);
    pintar::frases(&mut f);
    letras::frases(&mut f, nombre, apodo);
    teclas::frases(&mut f);
    globos::frases(&mut f);
    memorice::frases(&mut f);
    cocina::frases(&mut f);
    clase::frases(&mut f, nombre);
    english::frases(&mut f);
    galeria::frases(&mut f);
    escribir::frases(&mut f);
    let mut vistos = std::collections::HashSet::new();
    f.0.retain(|p| vistos.insert(p.clone()));
    f.0
}

/// Botón circular con ícono de altavoz.
pub fn boton_altavoz(l: &mut Lienzo, x: f32, y: f32, r: f32, c: C, cx: &Cx) {
    let sobre = dist(cx.rl, (x, y)) < r;
    let rr = if sobre { r * 1.08 } else { r };
    l.circulo(x, y + 3.0, rr, oscurecer(c, 0.25));
    l.circulo(x, y, rr, c);
    l.altavoz(x, y, rr * 1.1, BLANCO);
}

/// Botón con texto (píldora).
pub fn boton_texto(l: &mut Lienzo, r: R, texto: &str, c: C, activo: bool, cx: &Cx) {
    let sobre = r.dentro(cx.rl);
    let fondo = if activo { c } else if sobre { aclarar(c, 0.55) } else { aclarar(c, 0.75) };
    l.rrect_r(r.mover(0.0, 3.0), r.h / 2.0, oscurecer(c, 0.25));
    l.rrect_r(r, r.h / 2.0, fondo);
    let tc = if activo { BLANCO } else { oscurecer(c, 0.35) };
    l.texto_c(texto, r.cx(), r.cy(), r.h * 0.48, tc);
}

/// Botón papelera; si `armado` > 0, parpadea pidiendo confirmación.
pub fn boton_papelera(l: &mut Lienzo, x: f32, y: f32, r: f32, armado: f32, cx: &Cx) {
    let c = if armado > 0.0 && (cx.t * 6.0).sin() > 0.0 { ROJO } else { rgb(120, 130, 150) };
    l.circulo(x, y + 3.0, r, oscurecer(c, 0.3));
    l.circulo(x, y, r, c);
    let u = r / 10.0;
    l.rrect(x - 4.0 * u, y - 3.0 * u, 8.0 * u, 9.0 * u, 1.0 * u, BLANCO);
    l.rect(x - 5.5 * u, y - 5.5 * u, 11.0 * u, 1.8 * u, BLANCO);
    l.rect(x - 1.5 * u, y - 7.0 * u, 3.0 * u, 1.8 * u, BLANCO);
    for k in [-2.0, 0.0, 2.0] {
        l.rect(x + k * u - 0.4 * u, y - 1.5 * u, 0.8 * u, 6.0 * u, c);
    }
}

/// Flecha dentro de un círculo. dir: -1 izquierda, 1 derecha.
pub fn boton_flecha(l: &mut Lienzo, x: f32, y: f32, r: f32, dir: f32, c: C, cx: &Cx) {
    let sobre = dist(cx.rl, (x, y)) < r;
    let rr = if sobre { r * 1.08 } else { r };
    l.circulo(x, y + 3.0, rr, oscurecer(c, 0.25));
    l.circulo(x, y, rr, c);
    let u = rr * 0.45;
    l.poligono(&[(x + dir * u, y), (x - dir * u * 0.6, y - u), (x - dir * u * 0.6, y + u)], BLANCO);
}

/// Texto grande de felicitación que rebota.
pub fn felicitar(l: &mut Lienzo, texto: &str, cx: f32, cy: f32, t: f32) {
    let dy = (t * 8.0).sin().abs() * -14.0;
    l.texto_c_borde(texto, cx, cy + dy, 76.0, AMARILLO, TINTA, 4.0);
}

pub fn ruta_datos(sub: &str) -> std::path::PathBuf {
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from).unwrap_or_else(|| "/tmp".into());
    home.join(sub)
}
