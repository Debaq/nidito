//! Configuración simple clave=valor: /etc/nidito.conf y ~/.config/nidito.conf

#[derive(Clone)]
pub struct Config {
    pub nombre: String,
    pub apodo: String,
    pub voz: String,
    pub voz_en: String,
    /// sal y hash sha256 de la clave de adultos (vacío = sin bloqueo)
    pub sal: String,
    pub clave: String,
    /// minutos diarios de juegos (0 = sin tope)
    pub juegos_min: u32,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            nombre: String::new(),
            apodo: String::new(),
            voz: "es-419".into(),
            voz_en: "en-us".into(),
            sal: String::new(),
            clave: String::new(),
            juegos_min: 20,
        }
    }
}

pub fn leer_en(c: &mut Config, txt: &str) {
    for linea in txt.lines() {
        let l = linea.trim();
        if l.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = l.split_once('=') {
            let v = v.trim().trim_matches('"').to_string();
            match k.trim() {
                "nombre" => c.nombre = v,
                "apodo" => c.apodo = v,
                "voz" if !v.is_empty() => c.voz = v,
                "voz_en" if !v.is_empty() => c.voz_en = v,
                "sal" => c.sal = v,
                "clave" => c.clave = v,
                "juegos_min" => c.juegos_min = v.parse().unwrap_or(20),
                _ => {}
            }
        }
    }
}

pub fn cargar() -> Config {
    let mut c = Config::default();
    let mut rutas = vec![std::path::PathBuf::from("/etc/nidito.conf")];
    if let Some(h) = std::env::var_os("HOME") {
        rutas.push(std::path::PathBuf::from(h).join(".config/nidito.conf"));
    }
    for r in rutas {
        if let Ok(txt) = std::fs::read_to_string(&r) {
            leer_en(&mut c, &txt);
        }
    }
    c
}
