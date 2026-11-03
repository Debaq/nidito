//! Dónde se guardan los dibujos, documentos y clases.
//!
//! Las rutas son relativas a la carpeta personal ("Dibujos/dibujo-123.png").
//! En el computador son archivos de verdad; en la versión web van al localStorage del navegador
//! (en base64, con la hora en que se guardó para poder ordenarlos).

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::path::PathBuf;

    fn ruta(rel: &str) -> PathBuf {
        crate::apps::ruta_datos(rel)
    }

    pub fn leer(rel: &str) -> Option<Vec<u8>> {
        std::fs::read(ruta(rel)).ok()
    }

    pub fn escribir(rel: &str, datos: &[u8]) -> bool {
        let r = ruta(rel);
        if let Some(d) = r.parent() {
            if std::fs::create_dir_all(d).is_err() {
                return false;
            }
        }
        std::fs::write(r, datos).is_ok()
    }

    /// Archivos de una carpeta con esa extensión: (ruta relativa, hora de modificación).
    pub fn listar(dir: &str, ext: &str) -> Vec<(String, u64)> {
        let Ok(rd) = std::fs::read_dir(ruta(dir)) else { return vec![] };
        rd.filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|x| x == ext))
            .map(|e| {
                let t = e.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
                (format!("{dir}/{}", e.file_name().to_string_lossy()), t)
            })
            .collect()
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    const PREFIJO: &str = "nidito:";
    const HORA: &str = "nidito-hora:";

    fn almacen() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    pub fn leer(rel: &str) -> Option<Vec<u8>> {
        let s = almacen()?.get_item(&format!("{PREFIJO}{rel}")).ok()??;
        super::base64::decodificar(&s)
    }

    pub fn escribir(rel: &str, datos: &[u8]) -> bool {
        let Some(s) = almacen() else { return false };
        let ok = s.set_item(&format!("{PREFIJO}{rel}"), &super::base64::codificar(datos)).is_ok();
        if ok {
            let _ = s.set_item(&format!("{HORA}{rel}"), &(js_sys::Date::now() as u64).to_string());
        }
        ok
    }

    pub fn listar(dir: &str, ext: &str) -> Vec<(String, u64)> {
        let Some(s) = almacen() else { return vec![] };
        let n = s.length().unwrap_or(0);
        let base = format!("{PREFIJO}{dir}/");
        let mut v = vec![];
        for i in 0..n {
            let Ok(Some(k)) = s.key(i) else { continue };
            let Some(nombre) = k.strip_prefix(&base) else { continue };
            if nombre.contains('/') || !nombre.ends_with(&format!(".{ext}")) {
                continue;
            }
            let rel = format!("{dir}/{nombre}");
            let t = s.get_item(&format!("{HORA}{rel}")).ok().flatten().and_then(|h| h.parse().ok()).unwrap_or(0);
            v.push((rel, t));
        }
        v
    }
}

#[cfg(target_arch = "wasm32")]
mod base64 {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn codificar(d: &[u8]) -> String {
        let mut s = String::with_capacity(d.len().div_ceil(3) * 4);
        for c in d.chunks(3) {
            let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
            for k in 0..4 {
                if k <= c.len() {
                    s.push(A[(n >> (18 - 6 * k) & 63) as usize] as char);
                } else {
                    s.push('=');
                }
            }
        }
        s
    }

    pub fn decodificar(s: &str) -> Option<Vec<u8>> {
        let mut v = Vec::with_capacity(s.len() / 4 * 3);
        let (mut acc, mut bits) = (0u32, 0);
        for b in s.bytes() {
            if b == b'=' {
                break;
            }
            let x = A.iter().position(|&a| a == b)? as u32;
            acc = acc << 6 | x;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                v.push((acc >> bits) as u8);
            }
        }
        Some(v)
    }
}

pub use imp::{escribir, leer, listar};

pub fn leer_texto(rel: &str) -> Option<String> {
    leer(rel).and_then(|b| String::from_utf8(b).ok())
}

/// Segundos desde 1970 (para nombrar archivos).
pub fn segundos() -> u64 {
    web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}
