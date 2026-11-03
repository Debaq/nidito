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
    /// cada cuántos minutos de uso se pide descansar la vista (0 = nunca)
    pub pausa_ojos: u32,
    /// (día, mes) del cumpleaños
    pub cumpleanos: Option<(u32, u32)>,
    /// idioma con que parte el escritorio: "es", "en" o "ambos"
    pub idioma: String,
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
            pausa_ojos: 20,
            cumpleanos: None,
            idioma: "es".into(),
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
                "pausa_ojos" => c.pausa_ojos = v.parse().unwrap_or(20),
                "cumpleanos" => c.cumpleanos = leer_fecha(&v),
                "idioma" if !v.is_empty() => c.idioma = v,
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

/// "15-03" o "15/03" -> Some((15, 3))
pub fn leer_fecha(v: &str) -> Option<(u32, u32)> {
    let (d, m) = v.split_once(['-', '/'])?;
    let (d, m) = (d.trim().parse().ok()?, m.trim().parse().ok()?);
    ((1..=31).contains(&d) && (1..=12).contains(&m)).then_some((d, m))
}

/// Ajustes que se pueden cambiar desde el menú de adultos.
#[derive(Clone, Copy, PartialEq)]
pub struct Ajustes {
    pub juegos_min: u32,
    pub pausa_ojos: u32,
    pub cumpleanos: Option<(u32, u32)>,
}

impl Config {
    pub fn ajustes(&self) -> Ajustes {
        Ajustes { juegos_min: self.juegos_min, pausa_ojos: self.pausa_ojos, cumpleanos: self.cumpleanos }
    }
}

/// Guarda los ajustes en ~/.config/nidito.conf (se lee después de /etc/nidito.conf, así que manda).
/// Conserva las demás líneas del archivo.
pub fn guardar_ajustes(a: &Ajustes) -> std::io::Result<()> {
    let Some(h) = std::env::var_os("HOME") else { return Ok(()) };
    let ruta = std::path::PathBuf::from(h).join(".config/nidito.conf");
    let viejo = std::fs::read_to_string(&ruta).unwrap_or_default();
    let mias = ["juegos_min", "pausa_ojos", "cumpleanos"];
    let mut txt: String = viejo
        .lines()
        .filter(|l| !mias.iter().any(|k| l.trim_start().starts_with(&format!("{k}="))))
        .map(|l| format!("{l}\n"))
        .collect();
    if txt.is_empty() {
        txt.push_str("# Ajustes hechos desde el menú de adultos de Nidito (mandan sobre /etc/nidito.conf)\n");
    }
    txt.push_str(&format!("juegos_min={}\npausa_ojos={}\n", a.juegos_min, a.pausa_ojos));
    txt.push_str(&match a.cumpleanos {
        Some((d, m)) => format!("cumpleanos={d:02}-{m:02}\n"),
        None => "cumpleanos=\n".into(),
    });
    std::fs::create_dir_all(ruta.parent().unwrap())?;
    std::fs::write(ruta, txt)
}
