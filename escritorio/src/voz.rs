//! Voz: frases pregrabadas (Piper o edge-tts, generadas al construir) con respaldo.
//!
//! Cada frase se busca como `<idioma>-<fnv64>.ogg` en el paquete de voces.
//! En el computador, si no está, se sintetiza con espeak-ng (cacheada) y se anota en ~/.cache/nidito/faltan.txt.
//! En la versión web el paquete es un solo archivo (`voces.bin`, ver `empaquetar`) y el respaldo es la voz del navegador.

use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
pub use nativo::Voz;
#[cfg(target_arch = "wasm32")]
pub use web::Voz;

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

#[cfg(not(target_arch = "wasm32"))]
mod nativo {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::thread;

    static CONTADOR: AtomicU64 = AtomicU64::new(0);
    type Pedido = (u64, Vec<Parte>);
    type Listo = (u64, Vec<Arc<[u8]>>);

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
                // las frases con el nombre las genera el instalador en .wav
                if let Ok(b) = fs::read(p.join(nombre.replace(".ogg", ".wav"))) {
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
}

/// Arma `voces.bin` (para la versión web) con todos los .ogg de una carpeta:
/// "NIDV1\n", cantidad (u32), y por cada audio: largo del nombre (u16), nombre, inicio y largo (u32); luego los datos.
#[cfg(not(target_arch = "wasm32"))]
pub fn empaquetar(dir: &std::path::Path, salida: &std::path::Path) -> std::io::Result<usize> {
    let mut nombres: Vec<String> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".ogg"))
        .collect();
    nombres.sort();
    let mut indice = Vec::new();
    let mut datos = Vec::new();
    indice.extend_from_slice(b"NIDV1\n");
    indice.extend_from_slice(&(nombres.len() as u32).to_le_bytes());
    for n in &nombres {
        let b = std::fs::read(dir.join(n))?;
        indice.extend_from_slice(&(n.len() as u16).to_le_bytes());
        indice.extend_from_slice(n.as_bytes());
        indice.extend_from_slice(&(datos.len() as u32).to_le_bytes());
        indice.extend_from_slice(&(b.len() as u32).to_le_bytes());
        datos.extend_from_slice(&b);
    }
    indice.extend_from_slice(&datos);
    std::fs::write(salida, indice)?;
    Ok(nombres.len())
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use wasm_bindgen::JsCast;

    struct Paquete {
        datos: Vec<u8>,
        indice: HashMap<String, (usize, usize)>,
    }

    fn leer_paquete(b: Vec<u8>) -> Option<Paquete> {
        let cab = b"NIDV1\n";
        if !b.starts_with(cab) {
            return None;
        }
        let u32_en = |i: usize| -> Option<usize> { Some(u32::from_le_bytes(b.get(i..i + 4)?.try_into().ok()?) as usize) };
        let n = u32_en(cab.len())?;
        let mut i = cab.len() + 4;
        let mut ent = Vec::with_capacity(n);
        for _ in 0..n {
            let l = u16::from_le_bytes(b.get(i..i + 2)?.try_into().ok()?) as usize;
            let nombre = String::from_utf8(b.get(i + 2..i + 2 + l)?.to_vec()).ok()?;
            i += 2 + l;
            ent.push((nombre, u32_en(i)?, u32_en(i + 4)?));
            i += 8;
        }
        let datos = b[i..].to_vec();
        let indice = ent.into_iter().filter(|(_, o, l)| o + l <= datos.len()).map(|(n, o, l)| (n, (o, l))).collect();
        Some(Paquete { datos, indice })
    }

    async fn bajar(url: &str) -> Option<Vec<u8>> {
        let w = web_sys::window()?;
        let r: web_sys::Response = wasm_bindgen_futures::JsFuture::from(w.fetch_with_str(url)).await.ok()?.dyn_into().ok()?;
        if !r.ok() {
            return None;
        }
        let buf = wasm_bindgen_futures::JsFuture::from(r.array_buffer().ok()?).await.ok()?;
        Some(js_sys::Uint8Array::new(&buf).to_vec())
    }

    pub struct Voz {
        activa: bool,
        paquete: Rc<RefCell<Option<Paquete>>>,
        listo: Option<Vec<Arc<[u8]>>>,
    }

    impl Voz {
        pub fn nueva(activa: bool, _voz_es: &str, _voz_en: &str) -> Voz {
            let paquete = Rc::new(RefCell::new(None));
            if activa {
                let p = paquete.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match bajar("voces.bin").await.and_then(leer_paquete) {
                        Some(paq) => *p.borrow_mut() = Some(paq),
                        None => web_sys::console::warn_1(&"nidito: sin voces.bin, se usa la voz del navegador".into()),
                    }
                });
            }
            Voz { activa, paquete, listo: None }
        }

        pub fn pedir(&mut self, partes: Vec<Parte>) {
            if !self.activa {
                return;
            }
            self.callar();
            let grabadas: Option<Vec<Arc<[u8]>>> = self.paquete.borrow().as_ref().and_then(|p| {
                partes.iter().map(|(l, t)| p.indice.get(&archivo(*l, t)).map(|&(o, n)| Arc::from(&p.datos[o..o + n]))).collect()
            });
            match grabadas {
                Some(a) if !a.is_empty() => self.listo = Some(a),
                _ => decir_navegador(&partes),
            }
        }

        pub fn callar(&mut self) {
            self.listo = None;
            if let Some(s) = web_sys::window().and_then(|w| w.speech_synthesis().ok()) {
                s.cancel();
            }
        }

        pub fn listo(&mut self) -> Option<Vec<Arc<[u8]>>> {
            self.listo.take()
        }
    }

    /// Frases que no están grabadas (p. ej. lo que escribe en Letras): voz del navegador.
    fn decir_navegador(partes: &[Parte]) {
        let Some(s) = web_sys::window().and_then(|w| w.speech_synthesis().ok()) else { return };
        for (l, t) in partes {
            if let Ok(u) = web_sys::SpeechSynthesisUtterance::new_with_text(t) {
                u.set_lang(match l {
                    L::Es => "es-ES",
                    L::En => "en-US",
                });
                u.set_rate(0.9);
                s.speak(&u);
            }
        }
    }
}
