//! Genera en el computador mismo (al instalar) las pocas frases que llevan el nombre del niño o niña,
//! con Piper. El resto de las frases viene pregrabado en la ISO.
//!
//! `nidito --voces-nombre <nidito.conf> <carpeta-voces> <carpeta-piper> <modelo-es.onnx> <modelo-en.onnx>`
//!
//! Piper escribe un .wav; aquí se le aplica el mismo arreglo que a las voces pregrabadas
//! (filtro pasa-altos, compresor, +13 dB y limitador, ver voces/filtro.txt) para que suenen igual de fuerte.

use crate::voz::{archivo, L};
use std::path::Path;
use std::process::{Command, Stdio};

/// Más lento que lo normal: es para niños pequeños (igual que voces/piper_gen.py).
const LARGO: &str = "1.12";
const SR: u32 = 22050;

pub fn generar(conf: &Path, dir: &Path, piper: &Path, modelo_es: &Path, modelo_en: &Path) -> Result<usize, String> {
    let mut cfg = crate::config::Config::default();
    let txt = std::fs::read_to_string(conf).map_err(|e| format!("no se pudo leer {}: {e}", conf.display()))?;
    crate::config::leer_en(&mut cfg, &txt);
    let faltan: Vec<(L, String)> = crate::apps::catalogo(&cfg.nombre, &cfg.apodo)
        .into_iter()
        .filter(|(l, t)| {
            let a = archivo(*l, t);
            !dir.join(&a).exists() && !dir.join(a.replace(".ogg", ".wav")).exists()
        })
        .collect();
    let total = faltan.len();
    let tmp = std::env::temp_dir().join(format!("nidito-voz-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let mut hechas = 0;
    // un solo piper por idioma: el modelo se carga una vez (en un disco lento eso es lo que más tarda)
    for (idioma, modelo) in [(L::Es, modelo_es), (L::En, modelo_en)] {
        let lote: Vec<&(L, String)> = faltan.iter().filter(|(l, _)| *l == idioma).collect();
        if lote.is_empty() {
            continue;
        }
        println!("MODELO {}", modelo.display());
        let mut hijo = Command::new(piper.join("piper"))
            .env("LD_LIBRARY_PATH", piper)
            .arg("--model")
            .arg(modelo)
            .arg("--espeak_data")
            .arg(piper.join("espeak-ng-data"))
            .args(["--length_scale", LARGO, "--json-input"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("no se pudo ejecutar piper: {e}"))?;
        if let Some(mut entrada) = hijo.stdin.take() {
            use std::io::Write;
            for (k, (_, t)) in lote.iter().enumerate() {
                let salida = tmp.join(format!("{k}.wav"));
                let linea = format!("{{\"text\": {}, \"output_file\": {}}}", json(t), json(&salida.to_string_lossy()));
                writeln!(entrada, "{linea}").map_err(|e| e.to_string())?;
            }
        }
        // avanza a medida que piper termina cada frase (escribe una línea por frase en stderr)
        if let Some(err) = hijo.stderr.take() {
            use std::io::BufRead;
            let mut k = 0;
            for linea in std::io::BufReader::new(err).lines().map_while(Result::ok) {
                if linea.contains("Real-time factor") && k < lote.len() {
                    k += 1;
                    println!("VOZ {}/{} {}", hechas + k, total, lote[k - 1].1);
                }
            }
        }
        if !hijo.wait().map(|s| s.success()).unwrap_or(false) {
            return Err("piper falló".into());
        }
        for (k, (l, t)) in lote.iter().enumerate() {
            let (muestras, sr) = leer_wav(&tmp.join(format!("{k}.wav")))?;
            let salida = arreglar(&remuestrear(&muestras, sr, SR));
            escribir_wav(&dir.join(archivo(*l, t).replace(".ogg", ".wav")), &salida)?;
        }
        hechas += lote.len();
    }
    let _ = std::fs::remove_dir_all(&tmp);
    Ok(total)
}

fn json(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn leer_wav(p: &Path) -> Result<(Vec<f32>, u32), String> {
    let mut r = hound::WavReader::open(p).map_err(|e| format!("wav de piper ilegible: {e}"))?;
    let spec = r.spec();
    let canales = spec.channels.max(1) as usize;
    let datos: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            let esc = (1i64 << (spec.bits_per_sample - 1)) as f32;
            r.samples::<i32>().filter_map(|s| s.ok()).map(|s| s as f32 / esc).collect()
        }
        hound::SampleFormat::Float => r.samples::<f32>().filter_map(|s| s.ok()).collect(),
    };
    // a mono
    let mono = datos.chunks(canales).map(|c| c.iter().sum::<f32>() / canales as f32).collect();
    Ok((mono, spec.sample_rate))
}

/// Remuestreo lineal (las voces de Piper ya vienen a 22050 Hz; esto es por si alguna no).
fn remuestrear(x: &[f32], de: u32, a: u32) -> Vec<f32> {
    if de == a || x.is_empty() {
        return x.to_vec();
    }
    let n = (x.len() as u64 * a as u64 / de as u64) as usize;
    (0..n)
        .map(|i| {
            let pos = i as f64 * de as f64 / a as f64;
            let j = pos.floor() as usize;
            let f = (pos - j as f64) as f32;
            let a0 = x[j.min(x.len() - 1)];
            let a1 = x[(j + 1).min(x.len() - 1)];
            a0 + (a1 - a0) * f
        })
        .collect()
}

/// highpass=f=160, acompressor=threshold=0.1:ratio=4:attack=3:release=60, volume=13dB, alimiter=limit=0.88
fn arreglar(x: &[f32]) -> Vec<f32> {
    let sr = SR as f32;
    // pasa-altos de 2º orden (Butterworth) a 160 Hz
    let w0 = std::f32::consts::TAU * 160.0 / sr;
    let alfa = w0.sin() / (2.0 * std::f32::consts::FRAC_1_SQRT_2);
    let c = w0.cos();
    let a0 = 1.0 + alfa;
    let (b0, b1, b2) = ((1.0 + c) / 2.0 / a0, -(1.0 + c) / a0, (1.0 + c) / 2.0 / a0);
    let (a1, a2) = (-2.0 * c / a0, (1.0 - alfa) / a0);
    let (mut x1, mut x2, mut y1, mut y2) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    // compresor (seguidor de envolvente) + ganancia + limitador
    let ataque = (-1.0 / (0.003 * sr)).exp();
    let suelta = (-1.0 / (0.060 * sr)).exp();
    let (umbral, razon, ganancia, limite) = (0.1f32, 4.0f32, 10f32.powf(13.0 / 20.0), 0.88f32);
    let mut env = 0.0f32;
    let mut lim = 1.0f32;
    let lim_suelta = (-1.0 / (0.025 * sr)).exp();
    x.iter()
        .map(|&s| {
            let y = b0 * s + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
            (x2, x1, y2, y1) = (x1, s, y1, y);
            let nivel = y.abs();
            let k = if nivel > env { ataque } else { suelta };
            env = k * env + (1.0 - k) * nivel;
            let g = if env > umbral { (umbral * (env / umbral).powf(1.0 / razon)) / env } else { 1.0 };
            let v = y * g * ganancia;
            // limitador: baja al tiro, se recupera de a poco
            let necesita = if v.abs() > limite { limite / v.abs() } else { 1.0 };
            lim = if necesita < lim { necesita } else { lim_suelta * lim + (1.0 - lim_suelta) * 1.0 };
            (v * lim).clamp(-limite, limite)
        })
        .collect()
}

fn escribir_wav(p: &Path, x: &[f32]) -> Result<(), String> {
    let spec = hound::WavSpec { channels: 1, sample_rate: SR, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
    let tmp = p.with_extension("tmp");
    let mut w = hound::WavWriter::create(&tmp, spec).map_err(|e| format!("no se pudo escribir {}: {e}", p.display()))?;
    for &s in x {
        w.write_sample((s * 32767.0) as i16).map_err(|e| e.to_string())?;
    }
    w.finalize().map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, p).map_err(|e| e.to_string())
}
