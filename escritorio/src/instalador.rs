//! Instalador con pantallas (ratatui) para la consola del computador: idioma, teclado, disco,
//! nombre, idioma de Nidito, voces, tiempo de juego y clave. El trabajo de disco lo hace
//! `nidito-instalar` (bash); esta parte pregunta, muestra el avance y nunca toca el disco sola.
//!
//! `nidito --instalador`              en el computador (lo lanza nidito-instalador.service)
//! `nidito --instalador --simular`    para probar en una terminal: discos falsos, no instala nada
//! `nidito --instalador-capturas DIR` dibuja cada pantalla en texto (para revisar el diseño)

use crate::voz::{archivo, L};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

const NIDITO: &str = "/usr/share/nidito";
/// Voces de Piper que puede traer la ISO: (modelo, es femenina)
const VOCES_ES: [(&str, bool); 2] = [("es_MX-claude-high", true), ("es_MX-ald-medium", false)];
const VOCES_EN: [(&str, bool); 2] = [("en_US-ljspeech-high", true), ("en_US-joe-medium", false)];
const PAUSAS: [u32; 6] = [0, 10, 15, 20, 30, 45];
/// teclado: (X, consola, nombre es, nombre en)
const TECLADOS: [(&str, &str, &str, &str); 3] = [
    ("latam", "la-latin1", "Latinoamericano (el de los computadores comprados en Chile)", "Latin American"),
    ("es", "es", "Español de España", "Spanish (Spain)"),
    ("us", "us", "Inglés (Estados Unidos)", "English (US)"),
];

// Colores (en la consola se cambia la paleta para que sean los de Nidito, ver `paleta`)
const FONDO: Color = Color::Black;
const CIELO: Color = Color::Cyan;
const SOL: Color = Color::Yellow;
const PASTO: Color = Color::Green;
const ROSA: Color = Color::Magenta;
const NARANJO: Color = Color::LightRed;
const TINTA: Color = Color::White;
const SUAVE: Color = Color::DarkGray;
const ROJO: Color = Color::Red;

const LOGO: [&str; 5] = [
    r" _   _ _     _ _ _        ",
    r"| \ | (_) __| (_) |_ ___  ",
    r"|  \| | |/ _` | | __/ _ \ ",
    r"| |\  | | (_| | | || (_) |",
    r"|_| \_|_|\__,_|_|\__\___/ ",
];
const GATO: [&str; 5] = [r"  /\_/\  ", r" ( o.o ) ", r"  > ^ <  ", r" /     \ ", r"(_)---(_)"];
const GATO_FELIZ: [&str; 5] = [r"  /\_/\  ", r" ( ^.^ ) ", r"  > ^ <  ", r" /     \ ", r"(_)---(_)"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum P {
    Idioma,
    Bienvenida,
    Teclado,
    Disco,
    Borrar,
    Nino,
    IdiomaNidito,
    Voces,
    Tiempo,
    Clave,
    Resumen,
    Instalando,
    Listo,
    Error,
}

const FLUJO: [P; 11] = [P::Idioma, P::Bienvenida, P::Teclado, P::Disco, P::Borrar, P::Nino, P::IdiomaNidito, P::Voces, P::Tiempo, P::Clave, P::Resumen];

struct Disco {
    dev: String,
    tam: String,
    modelo: String,
}

#[derive(Default)]
struct Avance {
    paso: usize,
    pct: u16,
    voz: Option<(usize, usize)>,
    registro: Vec<String>,
}

pub struct Inst {
    en: bool,
    p: P,
    sel: usize,
    campo: usize,
    discos: Vec<Disco>,
    disco: usize,
    borrar: String,
    teclado: usize,
    nombre: String,
    apodo: String,
    idioma: usize,
    voz_es: usize,
    voz_en: usize,
    hay_voces: bool,
    juegos: u32,
    pausa: usize,
    dia: String,
    mes: String,
    clave1: String,
    clave2: String,
    aviso: Option<String>,
    avance: Avance,
    rx: Option<Receiver<String>>,
    error: Vec<String>,
    simular: bool,
    t: u32,
    audio: Option<(rodio::OutputStream, rodio::Sink)>,
}

impl Inst {
    fn nuevo(simular: bool) -> Inst {
        let cfg = crate::config::cargar();
        let hay_voces = std::path::Path::new(NIDITO).join("piper/piper").exists()
            && VOCES_ES.iter().chain(VOCES_EN.iter()).all(|(m, _)| std::path::Path::new(NIDITO).join(format!("voces-{m}")).is_dir());
        let (dia, mes) = cfg.cumpleanos.map(|(d, m)| (d.to_string(), m.to_string())).unwrap_or_default();
        Inst {
            en: false,
            p: P::Idioma,
            sel: 0,
            campo: 0,
            discos: vec![],
            disco: 0,
            borrar: String::new(),
            teclado: 0,
            nombre: cfg.nombre.clone(),
            apodo: cfg.apodo.clone(),
            idioma: match cfg.idioma.as_str() {
                "en" => 1,
                "ambos" => 2,
                _ => 0,
            },
            voz_es: 0,
            voz_en: 0,
            hay_voces: hay_voces || simular,
            juegos: cfg.juegos_min,
            pausa: PAUSAS.iter().position(|&p| p == cfg.pausa_ojos).unwrap_or(3),
            dia,
            mes,
            clave1: String::new(),
            clave2: String::new(),
            aviso: None,
            avance: Avance::default(),
            rx: None,
            error: vec![],
            simular,
            t: 0,
            audio: None,
        }
    }

    fn tx<'a>(&self, es: &'a str, en: &'a str) -> &'a str {
        if self.en {
            en
        } else {
            es
        }
    }

    fn flujo(&self) -> Vec<P> {
        FLUJO.iter().copied().filter(|&p| p != P::Voces || self.hay_voces).collect()
    }

    fn ir(&mut self, p: P) {
        self.p = p;
        self.sel = 0;
        self.campo = 0;
        self.aviso = None;
        match p {
            P::Teclado => self.sel = self.teclado,
            P::Disco => {
                self.cargar_discos();
                self.sel = self.disco;
            }
            P::IdiomaNidito => self.sel = self.idioma,
            P::Borrar => self.borrar.clear(),
            P::Clave => {
                self.clave1.clear();
                self.clave2.clear();
            }
            _ => {}
        }
    }

    fn siguiente(&mut self) {
        let f = self.flujo();
        if let Some(i) = f.iter().position(|&p| p == self.p) {
            if i + 1 < f.len() {
                self.ir(f[i + 1]);
            }
        }
    }

    fn anterior(&mut self) {
        let f = self.flujo();
        if let Some(i) = f.iter().position(|&p| p == self.p) {
            if i > 0 {
                self.ir(f[i - 1]);
            }
        }
    }

    fn cargar_discos(&mut self) {
        self.discos = if self.simular {
            vec![
                Disco { dev: "/dev/sda".into(), tam: "232,9G".into(), modelo: "ST250LT012-9WS14C".into() },
                Disco { dev: "/dev/sdb".into(), tam: "14,9G".into(), modelo: "Otro disco".into() },
            ]
        } else {
            Command::new("/usr/local/bin/nidito-instalar")
                .arg("discos")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
                .unwrap_or_default()
                .lines()
                .filter_map(|l| {
                    let mut c = l.split('\t');
                    Some(Disco { dev: c.next()?.into(), tam: c.next().unwrap_or("").into(), modelo: c.next().unwrap_or("").into() })
                })
                .collect()
        };
        self.disco = self.disco.min(self.discos.len().saturating_sub(1));
    }

    fn palabra_borrar(&self) -> &'static str {
        if self.en {
            "ERASE"
        } else {
            "BORRAR"
        }
    }

    fn cumple(&self) -> Option<(u32, u32)> {
        let d: u32 = self.dia.parse().ok()?;
        let m: u32 = self.mes.parse().ok()?;
        ((1..=31).contains(&d) && (1..=12).contains(&m)).then_some((d, m))
    }

    fn idioma_codigo(&self) -> &'static str {
        ["es", "en", "ambos"][self.idioma.min(2)]
    }

    /// Escucha la voz elegida con una frase ya grabada.
    fn escuchar(&mut self, ingles: bool) {
        let (modelo, l, texto) = if ingles {
            (VOCES_EN[self.voz_en].0, L::En, crate::escritorio::saludo("").1)
        } else {
            (VOCES_ES[self.voz_es].0, L::Es, crate::escritorio::saludo("").0)
        };
        let ruta = std::path::Path::new(NIDITO).join(format!("voces-{modelo}")).join(archivo(l, &texto));
        let Ok(bytes) = std::fs::read(ruta) else { return };
        if self.audio.is_none() {
            if let Ok((s, h)) = rodio::OutputStream::try_default() {
                if let Ok(sink) = rodio::Sink::try_new(&h) {
                    self.audio = Some((s, sink));
                }
            }
        }
        if let Some((_, sink)) = &self.audio {
            sink.stop();
            if let Ok(d) = rodio::Decoder::new(std::io::Cursor::new(bytes)) {
                sink.append(d);
                sink.play();
            }
        }
    }

    /// Lanza `nidito-instalar instalar` (o lo simula) y lee su avance.
    fn instalar(&mut self) {
        let (tx, rx) = channel();
        self.rx = Some(rx);
        self.avance = Avance::default();
        let limpio = |s: &str| s.replace(['\n', '\r', '"'], "").trim().to_string();
        let mut conf = format!(
            "nombre={}\napodo={}\nidioma={}\njuegos_min={}\npausa_ojos={}\n",
            limpio(&self.nombre),
            limpio(&self.apodo),
            self.idioma_codigo(),
            self.juegos,
            PAUSAS[self.pausa]
        );
        conf.push_str(&match self.cumple() {
            Some((d, m)) => format!("cumpleanos={d:02}-{m:02}\n"),
            None => "cumpleanos=\n".into(),
        });
        let ruta_conf = std::env::temp_dir().join("nidito-instalar.conf");
        let _ = std::fs::write(&ruta_conf, &conf);
        let (xkb, kmap, _, _) = TECLADOS[self.teclado];
        let mut env: Vec<(String, String)> = vec![
            ("DISCO".into(), self.discos.get(self.disco).map(|d| d.dev.clone()).unwrap_or_default()),
            ("XKB".into(), xkb.into()),
            ("KMAP".into(), kmap.into()),
            ("CLAVE".into(), self.clave1.clone()),
            ("CONF".into(), ruta_conf.to_string_lossy().into_owned()),
        ];
        if self.hay_voces {
            env.push(("VOZ_ES".into(), VOCES_ES[self.voz_es].0.into()));
            env.push(("VOZ_EN".into(), VOCES_EN[self.voz_en].0.into()));
        }
        if self.simular {
            std::thread::spawn(move || {
                let pasos = ["disco", "copia", "ajustes", "voces", "arranque", "fin"];
                for (i, p) in pasos.iter().enumerate() {
                    let _ = tx.send(format!("PASO {}/6 {p}", i + 1));
                    match *p {
                        "copia" => {
                            for n in (0..=100).step_by(4) {
                                let _ = tx.send(format!("PCT {n}"));
                                std::thread::sleep(Duration::from_millis(60));
                            }
                        }
                        "voces" => {
                            for n in 1..=10 {
                                let _ = tx.send(format!("VOZ {n}/10 frase de prueba"));
                                std::thread::sleep(Duration::from_millis(150));
                            }
                        }
                        _ => std::thread::sleep(Duration::from_millis(500)),
                    }
                }
                let _ = tx.send("LISTO".into());
            });
            return;
        }
        let hijo = Command::new("/bin/sh")
            .args(["-c", "exec /usr/local/bin/nidito-instalar instalar 2>&1"])
            .envs(env)
            .stdout(Stdio::piped())
            .stdin(Stdio::null())
            .spawn();
        match hijo {
            Ok(mut h) => {
                let out = h.stdout.take();
                std::thread::spawn(move || {
                    if let Some(o) = out {
                        for l in BufReader::new(o).lines().map_while(Result::ok) {
                            let _ = tx.send(l);
                        }
                    }
                    let ok = h.wait().map(|s| s.success()).unwrap_or(false);
                    let _ = tx.send(if ok { "FIN_OK".into() } else { "FIN_ERROR".into() });
                });
            }
            Err(e) => {
                self.error = vec![format!("{e}")];
                self.p = P::Error;
            }
        }
    }

    fn leer_avance(&mut self) {
        let Some(rx) = &self.rx else { return };
        let mut lineas = vec![];
        while let Ok(l) = rx.try_recv() {
            lineas.push(l);
        }
        for l in lineas {
            if let Some(r) = l.strip_prefix("PASO ") {
                let n = r.split('/').next().and_then(|n| n.parse().ok()).unwrap_or(0);
                self.avance.paso = n;
            } else if let Some(r) = l.strip_prefix("PCT ") {
                self.avance.pct = r.trim().parse().unwrap_or(0).min(100);
            } else if let Some(r) = l.strip_prefix("VOZ ") {
                let (a, b) = r.split_once(' ').map(|(a, _)| a).unwrap_or(r).split_once('/').unwrap_or(("0", "0"));
                self.avance.voz = Some((a.parse().unwrap_or(0), b.parse().unwrap_or(0)));
            } else if let Some(r) = l.strip_prefix("ERROR ") {
                self.error.push(r.to_string());
            } else if l == "LISTO" || l == "FIN_OK" {
                if self.error.is_empty() {
                    self.p = P::Listo;
                }
            } else if l == "FIN_ERROR" {
                if self.error.is_empty() {
                    self.error.push(self.tx("la instalación se detuvo.", "the installation stopped.").into());
                }
                self.error.extend(self.avance.registro.iter().rev().take(10).rev().cloned());
                self.p = P::Error;
            } else if !l.trim().is_empty() {
                if !self.error.is_empty() {
                    self.error.push(l.clone());
                }
                self.avance.registro.push(l);
                if self.avance.registro.len() > 50 {
                    self.avance.registro.remove(0);
                }
            }
        }
    }

    // ---------------- teclas ----------------

    /// Devuelve true para salir del instalador.
    fn tecla(&mut self, k: KeyCode) -> bool {
        self.aviso = None;
        let lista = |sel: &mut usize, n: usize, k: KeyCode| match k {
            KeyCode::Up => *sel = (*sel + n - 1) % n,
            KeyCode::Down | KeyCode::Tab => *sel = (*sel + 1) % n,
            _ => {}
        };
        match self.p {
            P::Idioma => match k {
                KeyCode::Up | KeyCode::Down | KeyCode::Tab => self.sel = 1 - self.sel,
                KeyCode::Enter => {
                    self.en = self.sel == 1;
                    if self.en && self.idioma == 0 {
                        self.idioma = 1;
                        self.teclado = 2;
                    }
                    self.siguiente();
                }
                _ => {}
            },
            P::Bienvenida => match k {
                KeyCode::Up | KeyCode::Down | KeyCode::Tab => self.sel = 1 - self.sel,
                KeyCode::Enter if self.sel == 0 => self.siguiente(),
                KeyCode::Enter => return self.apagar(),
                KeyCode::Esc => self.anterior(),
                _ => {}
            },
            P::Teclado => match k {
                KeyCode::Enter => {
                    self.teclado = self.sel;
                    if !self.simular {
                        let _ = Command::new("loadkeys").arg(TECLADOS[self.teclado].1).stdout(Stdio::null()).stderr(Stdio::null()).status();
                    }
                    self.siguiente();
                }
                KeyCode::Esc => self.anterior(),
                k => lista(&mut self.sel, TECLADOS.len(), k),
            },
            P::Disco => match k {
                KeyCode::Enter if !self.discos.is_empty() => {
                    self.disco = self.sel;
                    self.siguiente();
                }
                KeyCode::Esc => self.anterior(),
                k if !self.discos.is_empty() => lista(&mut self.sel, self.discos.len(), k),
                _ => {}
            },
            P::Borrar => match k {
                KeyCode::Enter => {
                    if self.borrar.trim().to_uppercase() == self.palabra_borrar() {
                        self.siguiente();
                    } else {
                        self.aviso = Some(format!("{} {}", self.tx("Escribe", "Type"), self.palabra_borrar()));
                    }
                }
                KeyCode::Esc => self.anterior(),
                KeyCode::Backspace => {
                    self.borrar.pop();
                }
                KeyCode::Char(c) if self.borrar.chars().count() < 10 => self.borrar.push(c),
                _ => {}
            },
            P::Nino => match k {
                KeyCode::Up | KeyCode::Down | KeyCode::Tab => self.campo = 1 - self.campo,
                KeyCode::Enter if self.campo == 0 => self.campo = 1,
                KeyCode::Enter => self.siguiente(),
                KeyCode::Esc => self.anterior(),
                KeyCode::Backspace => {
                    if self.campo == 0 { &mut self.nombre } else { &mut self.apodo }.pop();
                }
                KeyCode::Char(c) => {
                    let s = if self.campo == 0 { &mut self.nombre } else { &mut self.apodo };
                    if s.chars().count() < 20 && (c.is_alphabetic() || c == ' ' || c == '-' || c == '\'') {
                        s.push(c);
                    }
                }
                _ => {}
            },
            P::IdiomaNidito => match k {
                KeyCode::Enter => {
                    self.idioma = self.sel;
                    self.siguiente();
                }
                KeyCode::Esc => self.anterior(),
                k => lista(&mut self.sel, 3, k),
            },
            P::Voces => match k {
                KeyCode::Up | KeyCode::Down | KeyCode::Tab => self.campo = 1 - self.campo,
                KeyCode::Left | KeyCode::Right => {
                    if self.campo == 0 {
                        self.voz_es = 1 - self.voz_es;
                    } else {
                        self.voz_en = 1 - self.voz_en;
                    }
                    self.escuchar(self.campo == 1);
                }
                KeyCode::Char('e' | 'E' | 'l' | 'L' | ' ') => self.escuchar(self.campo == 1),
                KeyCode::Enter => self.siguiente(),
                KeyCode::Esc => self.anterior(),
                _ => {}
            },
            P::Tiempo => match k {
                KeyCode::Up => self.campo = (self.campo + 3) % 4,
                KeyCode::Down | KeyCode::Tab => self.campo = (self.campo + 1) % 4,
                KeyCode::Left | KeyCode::Right => {
                    let mas = k == KeyCode::Right;
                    match self.campo {
                        0 => self.juegos = if mas { (self.juegos + 5).min(180) } else { self.juegos.saturating_sub(5) },
                        1 => self.pausa = if mas { (self.pausa + 1).min(PAUSAS.len() - 1) } else { self.pausa.saturating_sub(1) },
                        _ => {}
                    }
                }
                KeyCode::Backspace => {
                    match self.campo {
                        2 => self.dia.pop(),
                        3 => self.mes.pop(),
                        _ => None,
                    };
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    let s = match self.campo {
                        2 => &mut self.dia,
                        3 => &mut self.mes,
                        _ => return false,
                    };
                    if s.len() < 2 {
                        s.push(c);
                    }
                }
                KeyCode::Enter => {
                    let vacio = self.dia.is_empty() && self.mes.is_empty();
                    if vacio || self.cumple().is_some() {
                        self.siguiente();
                    } else {
                        self.aviso = Some(self.tx("El cumpleaños no es una fecha válida (o déjalo vacío).", "The birthday is not a valid date (or leave it empty).").into());
                        self.campo = 2;
                    }
                }
                KeyCode::Esc => self.anterior(),
                _ => {}
            },
            P::Clave => match k {
                KeyCode::Up | KeyCode::Down | KeyCode::Tab => self.campo = 1 - self.campo,
                KeyCode::Backspace => {
                    if self.campo == 0 { &mut self.clave1 } else { &mut self.clave2 }.pop();
                }
                KeyCode::Char(c) => {
                    let s = if self.campo == 0 { &mut self.clave1 } else { &mut self.clave2 };
                    if s.chars().count() < 40 {
                        s.push(c);
                    }
                }
                KeyCode::Enter if self.campo == 0 => self.campo = 1,
                KeyCode::Enter => {
                    if self.clave1.chars().count() < 4 {
                        self.aviso = Some(self.tx("Muy corta: mínimo 4 caracteres.", "Too short: at least 4 characters.").into());
                        self.clave1.clear();
                        self.clave2.clear();
                        self.campo = 0;
                    } else if self.clave1 != self.clave2 {
                        self.aviso = Some(self.tx("No coinciden, escríbela otra vez.", "They don't match, type it again.").into());
                        self.clave1.clear();
                        self.clave2.clear();
                        self.campo = 0;
                    } else {
                        self.siguiente();
                    }
                }
                KeyCode::Esc => self.anterior(),
                _ => {}
            },
            P::Resumen => match k {
                KeyCode::Enter => {
                    self.p = P::Instalando;
                    self.instalar();
                }
                KeyCode::Esc => self.anterior(),
                _ => {}
            },
            P::Instalando => {}
            P::Listo => {
                if k == KeyCode::Enter {
                    if !self.simular {
                        let _ = Command::new("systemctl").arg("reboot").status();
                    }
                    return true;
                }
            }
            P::Error => {
                if k == KeyCode::Enter {
                    return self.apagar();
                }
            }
        }
        false
    }

    fn apagar(&self) -> bool {
        if !self.simular {
            let _ = Command::new("systemctl").arg("poweroff").status();
        }
        true
    }

    // ---------------- dibujo ----------------

    fn dibujar(&self, f: &mut Frame) {
        let area = f.area();
        f.render_widget(Block::new().style(Style::new().bg(FONDO).fg(TINTA)), area);
        let alto_logo = if area.height >= 28 { 6 } else { 0 };
        let [cab, cuerpo, pie] = Layout::vertical([Constraint::Length(alto_logo), Constraint::Min(8), Constraint::Length(1)]).areas(area);
        if alto_logo > 0 {
            self.dibujar_logo(f, cab);
        }
        let ancho = cuerpo.width.min(78);
        let alto = cuerpo.height.min(20);
        let tarjeta = Rect { x: cuerpo.x + (cuerpo.width - ancho) / 2, y: cuerpo.y + (cuerpo.height - alto) / 2, width: ancho, height: alto };
        let (titulo, color) = self.titulo();
        let paso = self.flujo().iter().position(|&p| p == self.p).filter(|&i| i >= 2).map(|i| format!(" {}/{} ", i - 1, self.flujo().len() - 2));
        let mut bloque = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(color))
            .title(Line::from(format!(" {titulo} ")).style(Style::new().fg(color).bold()))
            .padding(Padding::new(2, 2, 1, 0));
        if let Some(p) = paso {
            bloque = bloque.title(Line::from(p).right_aligned().style(Style::new().fg(SUAVE)));
        }
        let dentro = bloque.inner(tarjeta);
        f.render_widget(bloque, tarjeta);
        self.dibujar_contenido(f, dentro);
        let ayuda = self.ayuda();
        f.render_widget(Paragraph::new(Line::from(ayuda).style(Style::new().fg(SUAVE))).alignment(Alignment::Center), pie);
    }

    fn dibujar_logo(&self, f: &mut Frame, area: Rect) {
        let gato = if self.p == P::Listo { GATO_FELIZ } else { GATO };
        let colores = [ROJO, NARANJO, SOL, PASTO, CIELO];
        let lineas: Vec<Line> = (0..5)
            .map(|i| {
                Line::from(vec![
                    Span::styled(LOGO[i], Style::new().fg(colores[i]).bold()),
                    Span::raw("    "),
                    Span::styled(gato[i], Style::new().fg(NARANJO)),
                ])
            })
            .collect();
        let alto = 5;
        let r = Rect { y: area.y + area.height.saturating_sub(alto + 1) / 2 + 1, height: alto, ..area };
        f.render_widget(Paragraph::new(lineas).alignment(Alignment::Center), r);
    }

    fn titulo(&self) -> (String, Color) {
        let t = |es: &str, en: &str| self.tx(es, en).to_string();
        match self.p {
            P::Idioma => ("Idioma · Language".into(), CIELO),
            P::Bienvenida => (t("Bienvenida", "Welcome"), CIELO),
            P::Teclado => (t("Teclado", "Keyboard"), CIELO),
            P::Disco => (t("Disco", "Disk"), CIELO),
            P::Borrar => (t("¡Cuidado!", "Careful!"), ROJO),
            P::Nino => (t("¿Para quién es?", "Who is it for?"), ROSA),
            P::IdiomaNidito => (t("Idioma de Nidito", "Nidito's language"), ROSA),
            P::Voces => (t("Voces", "Voices"), ROSA),
            P::Tiempo => (t("Tiempo de juego", "Play time"), SOL),
            P::Clave => (t("Clave de adultos", "Grown-up password"), SOL),
            P::Resumen => (t("Todo listo para instalar", "Ready to install"), PASTO),
            P::Instalando => (t("Instalando Nidito", "Installing Nidito"), PASTO),
            P::Listo => (t("¡Listo!", "Done!"), PASTO),
            P::Error => (t("Algo salió mal", "Something went wrong"), ROJO),
        }
    }

    fn ayuda(&self) -> String {
        let t = |es: &str, en: &str| self.tx(es, en).to_string();
        match self.p {
            P::Idioma => "↑↓  ·  Enter".into(),
            P::Bienvenida | P::Teclado | P::Disco | P::IdiomaNidito => t("↑↓ elegir  ·  Enter seguir  ·  Esc volver", "↑↓ choose  ·  Enter next  ·  Esc back"),
            P::Borrar | P::Resumen => t("Enter seguir  ·  Esc volver", "Enter next  ·  Esc back"),
            P::Nino | P::Clave => t("↑↓ cambiar de campo  ·  Enter seguir  ·  Esc volver", "↑↓ switch field  ·  Enter next  ·  Esc back"),
            P::Voces => t("↑↓ idioma  ·  ←→ elegir voz  ·  E escuchar  ·  Enter seguir", "↑↓ language  ·  ←→ choose voice  ·  E listen  ·  Enter next"),
            P::Tiempo => t("↑↓ campo  ·  ←→ cambiar  ·  números para el cumpleaños  ·  Enter seguir", "↑↓ field  ·  ←→ change  ·  numbers for the birthday  ·  Enter next"),
            P::Instalando => t("No apagues el computador", "Don't turn off the computer"),
            P::Listo => t("Saca el pendrive y presiona Enter para reiniciar", "Remove the USB stick and press Enter to restart"),
            P::Error => t("Enter apaga el computador", "Enter turns off the computer"),
        }
    }

    fn opciones(&self, f: &mut Frame, area: Rect, items: &[String], sel: usize) {
        let lineas: Vec<Line> = items
            .iter()
            .enumerate()
            .flat_map(|(i, s)| {
                let l = if i == sel {
                    Line::from(vec![Span::styled(" ▶ ", Style::new().fg(SOL).bold()), Span::styled(format!(" {s} "), Style::new().fg(FONDO).bg(CIELO).bold())])
                } else {
                    Line::from(vec![Span::raw("   "), Span::styled(format!(" {s} "), Style::new().fg(TINTA))])
                };
                [l, Line::raw("")]
            })
            .collect();
        f.render_widget(Paragraph::new(lineas), area);
    }

    fn campo(&self, etiqueta: &str, valor: &str, activo: bool, ancho: usize) -> Vec<Line<'static>> {
        let cursor = if activo && self.t % 10 < 6 { "▏" } else { " " };
        let relleno = ancho.saturating_sub(valor.chars().count() + 1);
        let estilo = if activo { Style::new().fg(FONDO).bg(TINTA) } else { Style::new().fg(TINTA).bg(SUAVE) };
        vec![
            Line::from(Span::styled(etiqueta.to_string(), Style::new().fg(if activo { SOL } else { TINTA }).bold())),
            Line::from(vec![Span::styled(format!(" {valor}{cursor}{} ", " ".repeat(relleno)), estilo)]),
            Line::raw(""),
        ]
    }

    fn texto(&self, f: &mut Frame, area: Rect, mut lineas: Vec<Line>) {
        if let Some(a) = &self.aviso {
            lineas.push(Line::raw(""));
            lineas.push(Line::from(Span::styled(format!("⚠ {a}"), Style::new().fg(ROJO).bold())));
        }
        f.render_widget(Paragraph::new(lineas).wrap(Wrap { trim: false }), area);
    }

    fn dibujar_contenido(&self, f: &mut Frame, area: Rect) {
        let t = |es: &str, en: &str| self.tx(es, en).to_string();
        let resaltado = Style::new().fg(SOL).bold();
        match self.p {
            P::Idioma => {
                let [a, b] = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).areas(area);
                f.render_widget(Paragraph::new(vec![Line::raw("¿En qué idioma quieres instalar?"), Line::from(Span::styled("Which language do you want to install in?", Style::new().fg(SUAVE)))]), a);
                self.opciones(f, b, &["Español".into(), "English".into()], self.sel);
            }
            P::Bienvenida => {
                let [a, b] = Layout::vertical([Constraint::Length(7), Constraint::Min(1)]).areas(area);
                self.texto(
                    f,
                    a,
                    vec![
                        Line::raw(t("Nidito es un computador para niños pequeños:", "Nidito is a computer for small children:")),
                        Line::raw(t("pintar, letras, números, música, figuras, inglés y más.", "painting, letters, numbers, music, shapes, English and more.")),
                        Line::raw(""),
                        Line::from(vec![
                            Span::raw(t("Este instalador ", "This installer ")),
                            Span::styled(t("BORRA todo el disco", "ERASES the whole disk"), Style::new().fg(ROJO).bold()),
                            Span::raw(t(" del computador.", " of this computer.")),
                        ]),
                    ],
                );
                self.opciones(f, b, &[t("Instalar Nidito", "Install Nidito"), t("Apagar el computador", "Turn off the computer")], self.sel);
            }
            P::Teclado => {
                let [a, b] = Layout::vertical([Constraint::Length(2), Constraint::Min(1)]).areas(area);
                f.render_widget(Paragraph::new(t("¿Qué teclado tiene este computador?", "Which keyboard does this computer have?")), a);
                let items: Vec<String> = TECLADOS.iter().map(|k| if self.en { k.3 } else { k.2 }.to_string()).collect();
                self.opciones(f, b, &items, self.sel);
            }
            P::Disco => {
                if self.discos.is_empty() {
                    self.texto(f, area, vec![Line::styled(t("No encontré ningún disco donde instalar.", "I couldn't find any disk to install on."), Style::new().fg(ROJO))]);
                    return;
                }
                let [a, b] = Layout::vertical([Constraint::Length(2), Constraint::Min(1)]).areas(area);
                f.render_widget(Paragraph::new(t("¿En qué disco instalo Nidito? (el pendrive no aparece)", "Which disk should Nidito go on? (the USB stick is not listed)")), a);
                let items: Vec<String> = self.discos.iter().map(|d| format!("{}   {}   {}", d.dev, d.tam, d.modelo)).collect();
                self.opciones(f, b, &items, self.sel);
            }
            P::Borrar => {
                let d = self.discos.get(self.disco);
                let mut l = vec![
                    Line::from(vec![
                        Span::raw(t("Se va a borrar ", "This will erase ")),
                        Span::styled(t("TODO", "EVERYTHING"), Style::new().fg(ROJO).bold()),
                        Span::raw(t(" lo que hay en:", " on:")),
                    ]),
                    Line::raw(""),
                    Line::styled(d.map(|d| format!("   {}  {}  {}", d.dev, d.tam, d.modelo)).unwrap_or_default(), resaltado),
                    Line::raw(""),
                    Line::raw(t("Fotos, documentos, Windows... no se pueden recuperar.", "Photos, documents, Windows... they can't be recovered.")),
                    Line::raw(""),
                ];
                l.extend(self.campo(&format!("{} {}:", t("Para seguir escribe", "To continue type"), self.palabra_borrar()), &self.borrar, true, 12));
                self.texto(f, area, l);
            }
            P::Nino => {
                let mut l = vec![
                    Line::raw(t("Nidito saluda por su nombre. Si lo dejas vacío, dice «¡Hola!».", "Nidito says hello by name. Leave it empty for a plain «Hello!».")),
                    Line::raw(""),
                ];
                l.extend(self.campo(&t("Nombre", "Name"), &self.nombre, self.campo == 0, 24));
                l.extend(self.campo(&t("Apodo cariñoso (opcional, se usa para saludar)", "Nickname (optional, used to say hello)"), &self.apodo, self.campo == 1, 24));
                self.texto(f, area, l);
            }
            P::IdiomaNidito => {
                let [a, b] = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).areas(area);
                f.render_widget(
                    Paragraph::new(t("¿En qué idioma habla Nidito? (se puede cambiar en el escritorio)", "Which language does Nidito speak? (it can be changed on the desktop)")).wrap(Wrap { trim: false }),
                    a,
                );
                self.opciones(f, b, &["Español".into(), "English".into(), t("Los dos (español e inglés)", "Both (Spanish and English)")], self.sel);
            }
            P::Voces => {
                let fila = |etq: String, opciones: [String; 2], sel: usize, activa: bool| -> Vec<Line<'static>> {
                    let mut spans = vec![Span::styled(format!("{etq:<10}"), Style::new().fg(if activa { SOL } else { TINTA }).bold())];
                    for (i, o) in opciones.iter().enumerate() {
                        let st = if i == sel {
                            if activa {
                                Style::new().fg(FONDO).bg(ROSA).bold()
                            } else {
                                Style::new().fg(FONDO).bg(TINTA)
                            }
                        } else {
                            Style::new().fg(SUAVE)
                        };
                        spans.push(Span::raw("  "));
                        spans.push(Span::styled(format!(" {o} "), st));
                    }
                    vec![Line::from(spans), Line::raw("")]
                };
                let mut l = vec![
                    Line::raw(t("Elige cómo suena Nidito. Con E escuchas la voz.", "Choose how Nidito sounds. Press E to listen.")),
                    Line::raw(""),
                ];
                l.extend(fila(t("Español", "Spanish"), [t("Mujer", "Woman"), t("Hombre", "Man")], self.voz_es, self.campo == 0));
                l.extend(fila(t("Inglés", "English"), [t("Mujer", "Woman"), t("Hombre", "Man")], self.voz_en, self.campo == 1));
                l.push(Line::styled(
                    t("Al instalar se graban las frases con su nombre (1 o 2 minutos).", "While installing, the phrases with the name are recorded (1 or 2 minutes)."),
                    Style::new().fg(SUAVE),
                ));
                self.texto(f, area, l);
            }
            P::Tiempo => {
                let juegos = if self.juegos == 0 { t("sin tope", "no limit") } else { format!("{} min", self.juegos) };
                let pausa = if PAUSAS[self.pausa] == 0 { t("nunca", "never") } else { format!("{} {} min", t("cada", "every"), PAUSAS[self.pausa]) };
                let control = |etq: String, v: String, activo: bool| -> Line<'static> {
                    let st = if activo { Style::new().fg(FONDO).bg(SOL).bold() } else { Style::new().fg(TINTA) };
                    Line::from(vec![
                        Span::styled(format!("{etq:<26}"), Style::new().fg(if activo { SOL } else { TINTA }).bold()),
                        Span::styled(if activo { " ◀ " } else { "   " }, Style::new().fg(SOL)),
                        Span::styled(format!(" {v} "), st),
                        Span::styled(if activo { " ▶" } else { "" }, Style::new().fg(SOL)),
                    ])
                };
                let fecha = |etq: String, v: &str, activo: bool| -> Span<'static> {
                    let cursor = if activo && self.t % 10 < 6 { "▏" } else { " " };
                    let st = if activo { Style::new().fg(FONDO).bg(TINTA) } else { Style::new().fg(TINTA).bg(SUAVE) };
                    Span::styled(format!(" {etq} {v:>2}{cursor}"), st)
                };
                let l = vec![
                    control(t("Juegos por día", "Games per day"), juegos, self.campo == 0),
                    Line::styled(t("   Pintar, Letras, Escribir, Figuras... no cuentan.", "   Painting, Letters, Writing, Shapes... don't count."), Style::new().fg(SUAVE)),
                    Line::raw(""),
                    control(t("Pausa para los ojos", "Eye break"), pausa, self.campo == 1),
                    Line::raw(""),
                    Line::from(vec![
                        Span::styled(format!("{:<29}", t("Cumpleaños (opcional)", "Birthday (optional)")), Style::new().fg(if self.campo >= 2 { SOL } else { TINTA }).bold()),
                        fecha(t("día", "day"), &self.dia, self.campo == 2),
                        Span::raw("  "),
                        fecha(t("mes", "month"), &self.mes, self.campo == 3),
                    ]),
                    Line::styled(t("   Ese día hay globos y «Cumpleaños feliz».", "   That day there are balloons and «Happy birthday»."), Style::new().fg(SUAVE)),
                ];
                self.texto(f, area, l);
            }
            P::Clave => {
                let mut l = vec![
                    Line::raw(t("Se pide al encender para elegir el tiempo de uso, y para el menú de adultos.", "It's asked when turning on, to choose the play time, and for the grown-up menu.")),
                    Line::styled(t("Mínimo 4 caracteres. No la olvides.", "At least 4 characters. Don't forget it."), Style::new().fg(SUAVE)),
                    Line::raw(""),
                ];
                l.extend(self.campo(&t("Clave", "Password"), &"●".repeat(self.clave1.chars().count()), self.campo == 0, 24));
                l.extend(self.campo(&t("Repite la clave", "Repeat the password"), &"●".repeat(self.clave2.chars().count()), self.campo == 1, 24));
                self.texto(f, area, l);
            }
            P::Resumen => {
                let d = self.discos.get(self.disco).map(|d| format!("{} ({})", d.dev, d.tam)).unwrap_or_default();
                let quien = match (self.nombre.trim().is_empty(), self.apodo.trim().is_empty()) {
                    (true, _) => t("(sin nombre)", "(no name)"),
                    (false, true) => self.nombre.trim().to_string(),
                    (false, false) => format!("{} «{}»", self.nombre.trim(), self.apodo.trim()),
                };
                let idioma = [t("Español", "Spanish"), "English".into(), t("Los dos", "Both")][self.idioma.min(2)].clone();
                let voz = |fem: bool| if fem { t("mujer", "woman") } else { t("hombre", "man") };
                let fila = |k: String, v: String| Line::from(vec![Span::styled(format!("{k:<22}"), Style::new().fg(SUAVE)), Span::styled(v, Style::new().fg(TINTA).bold())]);
                let mut l = vec![
                    fila(t("Disco (se borra)", "Disk (erased)"), d),
                    fila(t("Teclado", "Keyboard"), if self.en { TECLADOS[self.teclado].3 } else { TECLADOS[self.teclado].2 }.split(" (").next().unwrap_or("").to_string()),
                    fila(t("Para", "For"), quien),
                    fila(t("Idioma", "Language"), idioma),
                ];
                if self.hay_voces {
                    l.push(fila(t("Voces", "Voices"), format!("{} {} · {} {}", t("español:", "Spanish:"), voz(VOCES_ES[self.voz_es].1), t("inglés:", "English:"), voz(VOCES_EN[self.voz_en].1))));
                }
                l.push(fila(t("Juegos por día", "Games per day"), if self.juegos == 0 { t("sin tope", "no limit") } else { format!("{} min", self.juegos) }));
                l.push(fila(t("Cumpleaños", "Birthday"), self.cumple().map(|(d, m)| format!("{d:02}-{m:02}")).unwrap_or_else(|| "—".into())));
                l.push(Line::raw(""));
                l.push(Line::styled(t("Presiona Enter para instalar. Tarda unos 10 minutos.", "Press Enter to install. It takes about 10 minutes."), resaltado));
                self.texto(f, area, l);
            }
            P::Instalando => self.dibujar_avance(f, area),
            P::Listo => {
                let n = self.apodo.trim().to_string();
                let n = if n.is_empty() { self.nombre.trim().to_string() } else { n };
                self.texto(
                    f,
                    area,
                    vec![
                        Line::styled(t("Nidito quedó instalado.", "Nidito is installed."), Style::new().fg(PASTO).bold()),
                        Line::raw(""),
                        Line::raw(t("1. Saca el pendrive.", "1. Remove the USB stick.")),
                        Line::raw(t("2. Presiona Enter: el computador se reinicia y entra a Nidito.", "2. Press Enter: the computer restarts into Nidito.")),
                        Line::raw(""),
                        Line::styled(
                            if n.is_empty() { t("¡A jugar!", "Let's play!") } else { format!("{} {n}!", t("¡A jugar,", "Let's play,")) },
                            Style::new().fg(ROSA).bold(),
                        ),
                        Line::raw(""),
                        Line::styled(t("Menú de adultos: Ctrl+Alt+P (pide la clave).", "Grown-up menu: Ctrl+Alt+P (asks for the password)."), Style::new().fg(SUAVE)),
                    ],
                );
            }
            P::Error => {
                let mut l = vec![
                    Line::styled(self.error.first().cloned().unwrap_or_default(), Style::new().fg(ROJO).bold()),
                    Line::raw(t("La instalación no terminó. Puedes intentarlo otra vez arrancando desde el pendrive.", "The installation didn't finish. You can try again by booting from the USB stick.")),
                    Line::raw(""),
                    Line::styled(t("Detalles para quien te ayude:", "Details for whoever helps you:"), Style::new().fg(SUAVE)),
                ];
                let n = area.height.saturating_sub(5) as usize;
                l.extend(self.error.iter().skip(1).rev().take(n).rev().map(|s| Line::styled(s.clone(), Style::new().fg(SUAVE))));
                self.texto(f, area, l);
            }
        }
    }

    fn dibujar_avance(&self, f: &mut Frame, area: Rect) {
        let t = |es: &str, en: &str| self.tx(es, en).to_string();
        let pasos = [
            t("Preparando el disco", "Preparing the disk"),
            t("Copiando Nidito", "Copying Nidito"),
            t("Ajustando el sistema", "Setting up the system"),
            t("Grabando las voces con su nombre", "Recording the voices with the name"),
            t("Preparando el arranque", "Preparing startup"),
            t("Terminando", "Finishing"),
        ];
        let actual = self.avance.paso;
        let giro = ["◐", "◓", "◑", "◒"][(self.t / 3) as usize % 4];
        let mut l: Vec<Line> = pasos
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let n = i + 1;
                let (marca, st) = if n < actual {
                    ("✔".to_string(), Style::new().fg(PASTO))
                } else if n == actual {
                    (giro.to_string(), Style::new().fg(SOL).bold())
                } else {
                    ("·".to_string(), Style::new().fg(SUAVE))
                };
                let mut extra = String::new();
                if n == actual && n == 4 {
                    if let Some((a, b)) = self.avance.voz {
                        extra = format!("  {a}/{b}");
                    }
                }
                Line::from(vec![Span::styled(format!(" {marca}  "), st), Span::styled(format!("{p}{extra}"), st)])
            })
            .collect();
        l.push(Line::raw(""));
        let [a, b, c] = Layout::vertical([Constraint::Length(l.len() as u16), Constraint::Length(1), Constraint::Min(1)]).areas(area);
        f.render_widget(Paragraph::new(l), a);
        let pct = match actual {
            0 | 1 => 0,
            2 => self.avance.pct * 70 / 100,
            n => (70 + (n as u16 - 2) * 7).min(100),
        };
        f.render_widget(
            Gauge::default()
                .gauge_style(Style::new().fg(PASTO).bg(SUAVE))
                .percent(pct)
                .label(Span::styled(format!("{pct}%"), Style::new().fg(TINTA).bold())),
            b,
        );
        let reg: Vec<Line> = self.avance.registro.iter().rev().take(c.height.saturating_sub(1) as usize).rev().map(|s| Line::styled(s.clone(), Style::new().fg(SUAVE))).collect();
        f.render_widget(Paragraph::new(reg), Rect { y: c.y + 1, height: c.height.saturating_sub(1), ..c });
    }
}

/// En la consola de Linux se pueden cambiar los 16 colores: se usan los de Nidito.
fn paleta(poner: bool) {
    if std::env::var("TERM").as_deref() != Ok("linux") {
        return;
    }
    let mut o = std::io::stdout();
    if poner {
        let colores = [
            "1b2033", "e63946", "2eaa60", "ffd13b", "3a6ee6", "ff75aa", "4dc8ff", "eef1f8", //
            "5a6380", "ff8c42", "57c45c", "ffe38a", "6ea0ff", "9b5de5", "a6e3ff", "ffffff",
        ];
        for (i, c) in colores.iter().enumerate() {
            let _ = write!(o, "\x1b]P{i:X}{c}");
        }
    } else {
        let _ = write!(o, "\x1b]R");
    }
    let _ = o.flush();
}

pub fn ejecutar(simular: bool) {
    let mut ins = Inst::nuevo(simular);
    if !simular && std::env::var("TERM").as_deref() == Ok("linux") {
        // letra grande y con tildes (terminus); si no está, queda la de siempre
        let _ = Command::new("setfont").arg("ter-v20b").stdout(Stdio::null()).stderr(Stdio::null()).status();
    }
    paleta(true);
    let mut term = ratatui::init();
    loop {
        ins.t = ins.t.wrapping_add(1);
        ins.leer_avance();
        let _ = term.draw(|f| ins.dibujar(f));
        if event::poll(Duration::from_millis(100)).unwrap_or(false) {
            if let Ok(Event::Key(k)) = event::read() {
                if k.kind == KeyEventKind::Press && ins.tecla(k.code) {
                    break;
                }
            }
        }
    }
    ratatui::restore();
    paleta(false);
}

/// Color de la paleta de Nidito (la misma que `paleta`) para las capturas.
fn hex(c: Color, fondo: bool) -> &'static str {
    match c {
        Color::Black => "#1b2033",
        Color::Red => "#e63946",
        Color::Green => "#2eaa60",
        Color::Yellow => "#ffd13b",
        Color::Blue => "#3a6ee6",
        Color::Magenta => "#ff75aa",
        Color::Cyan => "#4dc8ff",
        Color::White | Color::Gray => "#eef1f8",
        Color::DarkGray => "#5a6380",
        Color::LightRed => "#ff8c42",
        Color::LightGreen => "#57c45c",
        Color::LightYellow => "#ffe38a",
        Color::LightBlue => "#6ea0ff",
        Color::LightMagenta => "#9b5de5",
        Color::LightCyan => "#a6e3ff",
        _ if fondo => "#1b2033",
        _ => "#eef1f8",
    }
}

/// La pantalla como imagen SVG, celda por celda (10x20 px, como la consola del netbook con ter-v20b).
fn a_svg(buf: &ratatui::buffer::Buffer) -> String {
    let (cw, ch) = (10, 20);
    let (w, h) = (buf.area.width as usize * cw, buf.area.height as usize * ch);
    let mut o = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='{w}' height='{h}' font-family='DejaVu Sans Mono, monospace' font-size='16'>\n<rect width='100%' height='100%' fill='#1b2033'/>\n"
    );
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            let c = &buf[(x, y)];
            let bg = hex(c.bg, true);
            if bg != "#1b2033" {
                o.push_str(&format!("<rect x='{}' y='{}' width='{cw}' height='{ch}' fill='{bg}'/>", x as usize * cw, y as usize * ch));
            }
            let sym = c.symbol();
            if sym.trim().is_empty() {
                continue;
            }
            // bordes como líneas continuas (la letra es más baja que la celda)
            let (x0, y0) = (x as usize * cw, y as usize * ch);
            let (mx, my) = (x0 + cw / 2, y0 + ch / 2);
            let linea = match sym {
                "─" => Some(format!("M{x0} {my}H{}", x0 + cw)),
                "│" => Some(format!("M{mx} {y0}V{}", y0 + ch)),
                "╭" => Some(format!("M{} {my}H{}Q{mx} {my} {mx} {}V{}", x0 + cw, mx + 4, my + 4, y0 + ch)),
                "╮" => Some(format!("M{x0} {my}H{}Q{mx} {my} {mx} {}V{}", mx - 4, my + 4, y0 + ch)),
                "╰" => Some(format!("M{mx} {y0}V{}Q{mx} {my} {} {my}H{}", my - 4, mx + 4, x0 + cw)),
                "╯" => Some(format!("M{mx} {y0}V{}Q{mx} {my} {} {my}H{x0}", my - 4, mx - 4)),
                _ => None,
            };
            if let Some(d) = linea {
                o.push_str(&format!("<path d='{d}' stroke='{}' stroke-width='1.5' fill='none'/>", hex(c.fg, false)));
                continue;
            }
            let txt = sym.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
            let negrita = if c.modifier.contains(Modifier::BOLD) { " font-weight='bold'" } else { "" };
            o.push_str(&format!(
                "<text x='{}' y='{}' fill='{}'{negrita} xml:space='preserve'>{txt}</text>",
                x as usize * cw,
                y as usize * ch + 15,
                hex(c.fg, false)
            ));
        }
        o.push('\n');
    }
    o.push_str("</svg>\n");
    o
}

/// Dibuja cada pantalla como texto y como imagen SVG (para revisar el diseño y para el README).
pub fn capturas(dir: &str) {
    use ratatui::backend::TestBackend;
    let _ = std::fs::create_dir_all(dir);
    for en in [false, true] {
        let mut ins = Inst::nuevo(true);
        ins.en = en;
        ins.nombre = "Sofía".into();
        ins.apodo = "Sofi".into();
        ins.dia = "15".into();
        ins.mes = "3".into();
        ins.cargar_discos();
        let mut pantallas: Vec<P> = FLUJO.to_vec();
        pantallas.extend([P::Instalando, P::Listo, P::Error]);
        for (i, p) in pantallas.iter().enumerate() {
            ins.ir(*p);
            match p {
                P::Borrar => ins.borrar = "BOR".into(),
                P::Clave => {
                    ins.clave1 = "1234".into();
                    ins.campo = 1;
                }
                P::Instalando => {
                    ins.avance = Avance { paso: 4, pct: 100, voz: Some((3, 10)), registro: vec!["MODELO /usr/share/nidito/modelos/es_MX-claude-high.onnx".into()] };
                }
                P::Error => ins.error = vec!["no se pudo preparar el disco /dev/sda.".into(), "sda  232,9G".into(), "[  12.3] I/O error, dev sda".into()],
                _ => {}
            }
            let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
            let _ = term.draw(|f| ins.dibujar(f));
            let buf = term.backend().buffer().clone();
            let mut txt = String::new();
            for y in 0..buf.area.height {
                for x in 0..buf.area.width {
                    txt.push_str(buf[(x, y)].symbol());
                }
                txt.push('\n');
            }
            let nombre = format!("{dir}/{}{:02}-{:?}", if en { "en-" } else { "es-" }, i, p);
            let _ = std::fs::write(format!("{nombre}.txt"), txt);
            let _ = std::fs::write(format!("{nombre}.svg"), a_svg(&buf));
        }
    }
    println!("capturas en {dir}");
}
