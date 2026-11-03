//! Voz: frases pregrabadas (edge-tts, generadas al construir la ISO) con respaldo de espeak-ng.
//!
//! Cada frase se busca como `<idioma>-<fnv64>.ogg` en el paquete de voces. Si no está,
//! se sintetiza con espeak-ng (cacheada) y se anota en ~/.cache/nidito/faltan.txt.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

static CONTADOR: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum L {
    Es,
    En,
}

impl L {
    pub fn codigo(self) -> &'static str {
        match self {
            L::Es => "es",
            L::En => "en",
        }
    }
}

pub type Parte = (L, String);
type Pedido = (u64, Vec<Parte>);
type Listo = (u64, Vec<Arc<[u8]>>);

pub fn fnv(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Nombre de archivo de una frase en el paquete de voces.
pub fn archivo(l: L, texto: &str) -> String {
    format!("{}-{:016x}.ogg", l.codigo(), fnv(&format!("{}|{}", l.codigo(), texto)))
}

pub fn dir_paquete() -> Option<PathBuf> {
    let mut c: Vec<PathBuf> = Vec::new();
    if let Some(d) = std::env::var_os("NIDITO_VOCES") {
        c.push(d.into());
    }
    c.push("/usr/share/nidito/voces".into());
    c.into_iter().find(|d| d.is_dir())
}

fn dir_cache() -> PathBuf {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    let d = base.join("nidito");
    let _ = fs::create_dir_all(d.join("espeak"));
    d
}

struct Motor {
    paquete: Option<PathBuf>,
    cache: PathBuf,
    voz_es: String,
    voz_en: String,
}

impl Motor {
    fn obtener(&self, l: L, texto: &str) -> Option<Arc<[u8]>> {
        let nombre = archivo(l, texto);
        if let Some(p) = &self.paquete {
            if let Ok(b) = fs::read(p.join(&nombre)) {
                return Some(b.into());
            }
        }
        self.anotar_faltante(l, texto);
        self.espeak(l, texto, &nombre)
    }

    fn anotar_faltante(&self, l: L, texto: &str) {
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(self.cache.join("faltan.txt")) {
            let _ = writeln!(f, "{}\t{}", l.codigo(), texto);
        }
    }

    fn espeak(&self, l: L, texto: &str, nombre: &str) -> Option<Arc<[u8]>> {
        let ruta = self.cache.join("espeak").join(nombre.replace(".ogg", ".wav"));
        if let Ok(b) = fs::read(&ruta) {
            return Some(b.into());
        }
        let tmp = self.cache.join(format!(".tmp-{}-{}.wav", std::process::id(), CONTADOR.fetch_add(1, Ordering::Relaxed)));
        let voz = match l {
            L::Es => &self.voz_es,
            L::En => &self.voz_en,
        };
        let ok = Command::new("espeak-ng")
            .args(["-v", voz, "-s", "135", "-p", "58", "-a", "170", "-w"])
            .arg(&tmp)
            .arg(texto)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok()?
            .success();
        if !ok {
            let _ = fs::remove_file(&tmp);
            return None;
        }
        fs::rename(&tmp, &ruta).ok()?;
        fs::read(&ruta).ok().map(Into::into)
    }
}

pub struct Voz {
    tx: Option<Sender<Pedido>>,
    rx: Receiver<Listo>,
    ultimo: u64,
}

impl Voz {
    pub fn nueva(activa: bool, voz_es: &str, voz_en: &str) -> Voz {
        let (tx_p, rx_p) = channel::<Pedido>();
        let (tx_r, rx_r) = channel::<Listo>();
        let tx = if activa {
            let motor = Motor { paquete: dir_paquete(), cache: dir_cache(), voz_es: voz_es.into(), voz_en: voz_en.into() };
            if motor.paquete.is_none() {
                eprintln!("nidito: sin paquete de voces, se usa espeak-ng");
            }
            thread::spawn(move || {
                while let Ok(mut p) = rx_p.recv() {
                    // si llegaron varios pedidos seguidos, solo vale el último
                    while let Ok(q) = rx_p.try_recv() {
                        p = q;
                    }
                    let audios: Vec<Arc<[u8]>> = p.1.iter().filter_map(|(l, t)| motor.obtener(*l, t)).collect();
                    if !audios.is_empty() && tx_r.send((p.0, audios)).is_err() {
                        break;
                    }
                }
            });
            Some(tx_p)
        } else {
            None
        };
        Voz { tx, rx: rx_r, ultimo: 0 }
    }

    pub fn pedir(&mut self, partes: Vec<Parte>) {
        self.ultimo += 1;
        if let Some(tx) = &self.tx {
            let _ = tx.send((self.ultimo, partes));
        }
    }

    pub fn callar(&mut self) {
        self.ultimo += 1;
    }

    pub fn listo(&mut self) -> Option<Vec<Arc<[u8]>>> {
        let mut r = None;
        while let Ok((id, b)) = self.rx.try_recv() {
            if id == self.ultimo {
                r = Some(b);
            }
        }
        r
    }
}

/// Lista de frases faltantes en el paquete (para revisar el catálogo).
pub fn faltantes(catalogo: &[Parte], paquete: &Path) -> Vec<Parte> {
    catalogo.iter().filter(|(l, t)| !paquete.join(archivo(*l, t)).exists()).cloned().collect()
}
