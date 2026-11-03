//! Letras: escribir libre (letras grandes que suenan) y copiar palabras/frases.

use super::*;
use crate::cx::Son;
use crate::efectos::Confeti;

const PALABRAS: [&str; 14] = [
    "MAMÁ", "PAPÁ", "SOL", "LUNA", "OSO", "CASA", "GATO", "PATO", "PAN", "HOLA", "TE AMO", "MI MAMÁ", "MI PAPÁ", "UNO DOS TRES",
];

#[derive(PartialEq, Clone, Copy)]
enum Modo {
    Libre,
    Copiar,
}

pub struct Letras {
    modo: Modo,
    texto: Vec<(char, C)>,
    palabras: Vec<String>,
    ip: usize,
    progreso: usize,
    celebrando: f32,
    error_t: f32,
    borrar_t: f32,
    confeti: Confeti,
    color_i: usize,
}

const B_LIBRE: R = R::new(20.0, 14.0, 200.0, 54.0);
const B_COPIAR: R = R::new(232.0, 14.0, 200.0, 54.0);
const ALTAVOZ: (f32, f32) = (880.0, 41.0);
const PAPELERA: (f32, f32) = (950.0, 41.0);
const TAM_LIBRE: f32 = 84.0;
const LINEA: f32 = 100.0;
const MAX_LINEAS: usize = 4;

fn lista_palabras(nombre: &str, apodo: &str) -> Vec<String> {
    let mut v = Vec::new();
    if !nombre.trim().is_empty() {
        v.push(nombre.trim().to_uppercase());
    }
    if !apodo.trim().is_empty() {
        v.push(apodo.trim().to_uppercase());
    }
    v.extend(PALABRAS.iter().map(|s| s.to_string()));
    v
}

fn frase_busca(c: char) -> (String, String) {
    let n = nombre_letra(c).unwrap_or_default();
    (format!("Busca la {n}."), format!("Find the letter {}.", mayus(base(c))))
}

pub fn frases(f: &mut Frases, nombre: &str, apodo: &str) {
    for p in lista_palabras(nombre, apodo) {
        f.es(p.to_lowercase());
        f.en(format!("You wrote {}.", p.to_lowercase()));
        for c in p.chars().filter(|c| c.is_alphanumeric()) {
            let (a, b) = frase_busca(c);
            f.bil(a, b);
        }
    }
    for c in "abcdefghijklmnñopqrstuvwxyz".chars() {
        let (a, b) = frase_busca(c);
        f.bil(a, b);
    }
    f.bil("Escribe lo que quieras.", "Write anything you want.");
    f.bil("Copia la palabra.", "Copy the word.");
    f.bil("¿Borrar todo? Toca otra vez.", "Erase everything? Touch again.");
    f.bil("Busca el espacio.", "Find the space bar.");
}

impl Letras {
    pub fn nuevo(cx: &mut Cx) -> Box<dyn App> {
        let palabras = lista_palabras(&cx.nombre, &cx.apodo);
        let mut l = Letras {
            modo: Modo::Copiar,
            texto: Vec::new(),
            palabras,
            ip: 0,
            progreso: 0,
            celebrando: 0.0,
            error_t: 0.0,
            borrar_t: 0.0,
            confeti: Confeti::default(),
            color_i: 0,
        };
        l.anunciar(cx);
        Box::new(l)
    }

    fn palabra(&self) -> Vec<char> {
        self.palabras[self.ip].chars().collect()
    }

    fn anunciar(&mut self, cx: &mut Cx) {
        match self.modo {
            Modo::Libre => cx.decir_bil("Escribe lo que quieras.", "Write anything you want."),
            Modo::Copiar => {
                let p = self.palabras[self.ip].to_lowercase();
                cx.decir_v(&["Copia la palabra.", p.as_str()], &["Copy the word."]);
            }
        }
    }

    fn decir_letra(&self, c: char, cx: &mut Cx) {
        if let Some(n) = nombre_letra(c) {
            cx.decir_bil(&n, &mayus(base(c)).to_string());
        }
    }

    fn cambiar_palabra(&mut self, d: i32, cx: &mut Cx) {
        let n = self.palabras.len() as i32;
        self.ip = ((self.ip as i32 + d).rem_euclid(n)) as usize;
        self.progreso = 0;
        self.anunciar(cx);
    }

    fn tecla_copiar(&mut self, c: char, cx: &mut Cx) {
        if self.celebrando > 0.0 {
            return;
        }
        let p = self.palabra();
        let Some(&esperada) = p.get(self.progreso) else { return };
        if base(c) == base(esperada) {
            self.progreso += 1;
            if c == ' ' {
                cx.sonar(Son::Clic);
            } else {
                self.decir_letra(esperada, cx);
            }
            if self.progreso >= p.len() {
                self.celebrando = 3.0;
                cx.sonar(Son::Fanfarria);
                self.confeti.lanzar(500.0, 260.0, 90, &mut cx.rng);
                let (a_es, a_en) = alabanza(cx);
                let pal = self.palabras[self.ip].to_lowercase();
                let en = format!("You wrote {pal}.");
                cx.decir_v(&[a_es, pal.as_str()], &[a_en, en.as_str()]);
            }
        } else {
            self.error_t = 0.6;
            cx.sonar(Son::Boop);
            if esperada == ' ' {
                cx.decir_bil("Busca el espacio.", "Find the space bar.");
            } else {
                let (a, b) = frase_busca(esperada);
                cx.decir_bil(&a, &b);
            }
        }
    }

    fn leer_ultima_palabra(&self, cx: &mut Cx) {
        let s: String = self.texto.iter().map(|x| x.0).collect();
        if let Some(p) = s.split_whitespace().last() {
            if p.chars().count() > 1 {
                crate::palabras::leer(cx, &p.to_lowercase());
            }
        }
    }
}

impl App for Letras {
    fn animando(&self) -> bool {
        self.celebrando > 0.0 || self.error_t > 0.0 || self.borrar_t > 0.0 || self.confeti.activo()
    }

    fn evento(&mut self, ev: &Ev, cx: &mut Cx) {
        match ev {
            Ev::Clic(x, y) => {
                let p = (*x, *y);
                if B_LIBRE.dentro(p) && self.modo != Modo::Libre {
                    self.modo = Modo::Libre;
                    cx.sonar(Son::Clic);
                    self.anunciar(cx);
                } else if B_COPIAR.dentro(p) && self.modo != Modo::Copiar {
                    self.modo = Modo::Copiar;
                    self.progreso = 0;
                    cx.sonar(Son::Clic);
                    self.anunciar(cx);
                } else if dist(p, ALTAVOZ) < 30.0 {
                    match self.modo {
                        Modo::Libre => {
                            let s: String = self.texto.iter().map(|x| x.0).collect::<String>().to_lowercase();
                            if !s.trim().is_empty() {
                                crate::palabras::leer(cx, &s);
                            }
                        }
                        Modo::Copiar => self.anunciar(cx),
                    }
                } else if dist(p, PAPELERA) < 30.0 {
                    if self.borrar_t > 0.0 {
                        self.texto.clear();
                        self.progreso = 0;
                        self.borrar_t = 0.0;
                        cx.sonar(Son::Whoosh);
                    } else {
                        self.borrar_t = 2.5;
                        cx.decir_bil("¿Borrar todo? Toca otra vez.", "Erase everything? Touch again.");
                    }
                } else if self.modo == Modo::Copiar {
                    if dist(p, (60.0, 260.0)) < 36.0 {
                        self.cambiar_palabra(-1, cx);
                    } else if dist(p, (940.0, 260.0)) < 36.0 {
                        self.cambiar_palabra(1, cx);
                    }
                }
            }
            Ev::Texto(t) => {
                for c in t.chars().filter(|c| !c.is_control()) {
                    match self.modo {
                        Modo::Libre => {
                            let col = ARCOIRIS[self.color_i % ARCOIRIS.len()];
                            self.color_i += 1;
                            self.texto.push((mayus(c), col));
                            if c.is_alphanumeric() {
                                self.decir_letra(c, cx);
                            } else {
                                cx.sonar(Son::Clic);
                            }
                        }
                        Modo::Copiar => self.tecla_copiar(c, cx),
                    }
                }
            }
            Ev::Tecla(T::Espacio) => match self.modo {
                Modo::Libre => {
                    self.leer_ultima_palabra(cx);
                    self.texto.push((' ', BLANCO));
                }
                Modo::Copiar => self.tecla_copiar(' ', cx),
            },
            Ev::Tecla(T::Enter) => {
                if self.modo == Modo::Libre {
                    self.leer_ultima_palabra(cx);
                    self.texto.push(('\n', BLANCO));
                }
            }
            Ev::Tecla(T::Borrar) => {
                if self.modo == Modo::Libre && self.texto.pop().is_some() {
                    cx.sonar(Son::Clic);
                }
            }
            _ => {}
        }
    }

    fn actualizar(&mut self, dt: f32, cx: &mut Cx) {
        self.confeti.actualizar(dt);
        self.error_t = (self.error_t - dt).max(0.0);
        if self.borrar_t > 0.0 {
            self.borrar_t -= dt;
        }
        if self.celebrando > 0.0 {
            self.celebrando -= dt;
            if self.celebrando <= 0.0 {
                self.cambiar_palabra(1, cx);
            }
        }
    }

    fn dibujar(&mut self, l: &mut Lienzo, cx: &Cx) {
        l.rect(0.0, 0.0, AW, AH, rgb(250, 250, 255));
        // renglones de cuaderno
        for i in 0..5 {
            l.rect(0.0, 180.0 + i as f32 * LINEA, AW, 2.0, rgb(215, 225, 245));
        }
        l.rect(0.0, 0.0, AW, 82.0, rgb(232, 238, 255));
        boton_texto(l, B_LIBRE, "Libre", AZUL, self.modo == Modo::Libre, cx);
        boton_texto(l, B_COPIAR, "Copiar", MORADO, self.modo == Modo::Copiar, cx);
        boton_altavoz(l, ALTAVOZ.0, ALTAVOZ.1, 28.0, AZUL, cx);
        boton_papelera(l, PAPELERA.0, PAPELERA.1, 28.0, self.borrar_t, cx);

        match self.modo {
            Modo::Libre => {
                // disposición con salto automático
                let mut lineas: Vec<Vec<(char, C, f32)>> = vec![vec![]];
                let mut x = 30.0;
                let mut cortes: Vec<usize> = vec![];
                for (i, &(c, col)) in self.texto.iter().enumerate() {
                    if c == '\n' {
                        lineas.push(vec![]);
                        cortes.push(i + 1);
                        x = 30.0;
                        continue;
                    }
                    let w = l.medir(&c.to_string(), TAM_LIBRE);
                    if x + w > AW - 30.0 {
                        lineas.push(vec![]);
                        cortes.push(i);
                        x = 30.0;
                    }
                    lineas.last_mut().unwrap().push((c, col, x));
                    x += w;
                }
                if lineas.len() > MAX_LINEAS {
                    // sube el texto: borra la primera línea
                    let n = cortes[0];
                    self.texto.drain(0..n);
                }
                let mut cy = 92.0;
                let mut fin = (30.0, 92.0);
                for linea in lineas.iter().take(MAX_LINEAS) {
                    for &(c, col, x) in linea {
                        let s = c.to_string();
                        l.texto(&s, x + 2.0, cy + 3.0, TAM_LIBRE, alfa(TINTA, 0.25));
                        let w = l.texto(&s, x, cy, TAM_LIBRE, col);
                        fin = (x + w, cy);
                    }
                    if linea.is_empty() {
                        fin = (30.0, cy);
                    }
                    cy += LINEA;
                }
                if self.texto.last().map(|x| x.0) == Some('\n') {
                    fin = (30.0, 92.0 + (lineas.len().min(MAX_LINEAS) - 1) as f32 * LINEA);
                }
                if (cx.t * std::f32::consts::TAU).sin() > 0.0 {
                    l.rrect(fin.0 + 4.0, fin.1 + 12.0, 7.0, 80.0, 3.0, AZUL);
                }
            }
            Modo::Copiar => {
                let p = self.palabra();
                let s: String = p.iter().collect();
                let mut tam = 140.0;
                let w = l.medir(&s, tam);
                if w > 780.0 {
                    tam = (tam * 780.0 / w).floor();
                }
                let w = l.medir(&s, tam);
                let mut x = AW / 2.0 - w / 2.0;
                let y = 262.0 - l.alto(tam) / 2.0;
                let sac = if self.error_t > 0.0 { (self.error_t * 40.0).sin() * 8.0 } else { 0.0 };
                for (i, &c) in p.iter().enumerate() {
                    let cs = c.to_string();
                    let cw = l.medir(&cs, tam);
                    if i < self.progreso {
                        let col = ARCOIRIS[i % ARCOIRIS.len()];
                        l.texto(&cs, x + 3.0, y + 4.0, tam, alfa(TINTA, 0.25));
                        l.texto(&cs, x, y, tam, col);
                    } else if i == self.progreso && self.celebrando <= 0.0 {
                        let pulso = 0.5 + 0.5 * (cx.t * 5.0).sin();
                        if c != ' ' {
                            l.texto(&cs, x + sac, y, tam, mezclar(rgb(190, 190, 205), rgb(120, 120, 150), pulso));
                        }
                        let alto = l.alto(tam);
                        l.rrect(x + 4.0 + sac, y + alto - 6.0, (cw - 8.0).max(30.0), 10.0, 5.0, alfa(NARANJO, 0.6 + 0.4 * pulso));
                    } else {
                        l.texto(&cs, x, y, tam, rgb(205, 205, 218));
                    }
                    x += cw;
                }
                boton_flecha(l, 60.0, 260.0, 34.0, -1.0, MORADO, cx);
                boton_flecha(l, 940.0, 260.0, 34.0, 1.0, MORADO, cx);
                // pista: la letra que toca
                if let Some(&c) = p.get(self.progreso) {
                    if self.celebrando <= 0.0 {
                        let txt = if c == ' ' { "espacio".to_string() } else { c.to_string() };
                        l.rrect(AW / 2.0 - 90.0, 410.0, 180.0, 90.0, 24.0, rgb(255, 240, 200));
                        l.texto_c(&txt, AW / 2.0, 455.0, if c == ' ' { 40.0 } else { 70.0 }, NARANJO);
                    }
                }
                if self.celebrando > 0.0 {
                    felicitar(l, "¡Muy bien!", AW / 2.0, 450.0, cx.t);
                }
            }
        }
        self.confeti.dibujar(l);
    }
}
