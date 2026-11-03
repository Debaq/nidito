//! Contexto compartido: audio, voz, azar, tiempo, idioma.

use crate::dibujos::Obj;
use crate::sonidos::{Sonidos, SR};
use crate::voz::{Parte, Voz, L};
use rodio::buffer::SamplesBuffer;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::io::Cursor;

pub struct Rng(u64);

impl Rng {
    pub fn nuevo(semilla: u64) -> Rng {
        Rng(semilla.wrapping_mul(0x9E3779B97F4A7C15) | 1)
    }
    pub fn u(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    pub fn f(&mut self) -> f32 {
        (self.u() >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn rango(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
    pub fn n(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.u() % n as u64) as usize
        }
    }
    pub fn elegir<'a, T>(&mut self, v: &'a [T]) -> &'a T {
        &v[self.n(v.len())]
    }
    pub fn barajar<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.n(i + 1);
            v.swap(i, j);
        }
    }
}

#[derive(Clone, Copy)]
pub enum Son {
    Ding,
    Fanfarria,
    Boop,
    Pop,
    Whoosh,
    Clic,
    Sello,
    Aplauso,
    Tic,
    Nota(usize),
    Xilo(usize),
    Cumpleanos,
}

pub struct Audio {
    voz_sink: Option<Sink>,
    clip_sink: Option<Sink>,
    handle: Option<OutputStreamHandle>,
    _stream: Option<OutputStream>,
    sonidos: Sonidos,
    voz: Voz,
}

impl Audio {
    pub fn nuevo(activo: bool, voz_es: &str, voz_en: &str) -> Audio {
        let (stream, handle) = if activo {
            match OutputStream::try_default() {
                Ok((s, h)) => (Some(s), Some(h)),
                Err(e) => {
                    eprintln!("nidito: sin audio: {e}");
                    (None, None)
                }
            }
        } else {
            (None, None)
        };
        Audio { voz_sink: None, clip_sink: None, handle, _stream: stream, sonidos: Sonidos::generar(), voz: Voz::nueva(activo, voz_es, voz_en) }
    }

    pub fn sonar(&self, s: Son) {
        let Some(h) = &self.handle else { return };
        let so = &self.sonidos;
        let datos = match s {
            Son::Ding => &so.ding,
            Son::Fanfarria => &so.fanfarria,
            Son::Boop => &so.boop,
            Son::Pop => &so.pop,
            Son::Whoosh => &so.whoosh,
            Son::Clic => &so.clic,
            Son::Sello => &so.sello,
            Son::Aplauso => &so.aplauso,
            Son::Tic => &so.tic,
            Son::Nota(i) => &so.notas[i % so.notas.len()],
            Son::Xilo(i) => &so.xilo[i % so.xilo.len()],
            Son::Cumpleanos => &so.cumple,
        };
        let _ = h.play_raw(SamplesBuffer::new(1, SR, datos.clone()).amplify(0.9));
    }

    pub fn decir(&mut self, partes: Vec<Parte>) {
        self.voz.pedir(partes);
    }

    pub fn callar(&mut self) {
        self.voz.callar();
        self.voz_sink = None;
        self.clip_sink = None;
    }

    /// Reproduce un audio (ogg/wav) embebido, p. ej. el sonido de un animal. Calla la voz.
    pub fn reproducir(&mut self, bytes: &'static [u8]) {
        self.callar();
        let Some(h) = &self.handle else { return };
        let Ok(sink) = Sink::try_new(h) else { return };
        if let Ok(dec) = Decoder::new(Cursor::new(bytes)) {
            sink.append(dec);
            self.clip_sink = Some(sink);
        }
    }

    pub fn actualizar(&mut self) {
        if let Some(audios) = self.voz.listo() {
            let Some(h) = &self.handle else { return };
            let Ok(sink) = Sink::try_new(h) else { return };
            for bytes in audios {
                if let Ok(dec) = Decoder::new(Cursor::new(bytes)) {
                    sink.append(dec);
                }
            }
            self.voz_sink = Some(sink);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Idioma {
    Es,
    En,
    Ambos,
}

pub struct Cx {
    pub audio: Audio,
    pub rng: Rng,
    pub t: f32,
    pub nombre: String,
    pub apodo: String,
    pub idioma: Idioma,
    /// ratón en coordenadas de pantalla
    pub raton: (f32, f32),
    /// ratón en coordenadas locales de la app abierta
    pub rl: (f32, f32),
    pub apretado: bool,
    /// registro de frases pedidas (para revisar el catálogo en pruebas)
    pub registro: Option<Vec<Parte>>,
}

impl Cx {
    pub fn nuevo(cfg: &crate::config::Config, audio: bool) -> Cx {
        let semilla = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(7);
        Cx {
            audio: Audio::nuevo(audio, &cfg.voz, &cfg.voz_en),
            rng: Rng::nuevo(semilla),
            t: 0.0,
            nombre: cfg.nombre.clone(),
            apodo: cfg.apodo.clone(),
            idioma: Idioma::Es,
            raton: (-100.0, -100.0),
            rl: (-100.0, -100.0),
            apretado: false,
            registro: None,
        }
    }
    pub fn sonar(&self, s: Son) {
        self.audio.sonar(s)
    }
    fn hablar(&mut self, partes: Vec<Parte>) {
        if let Some(r) = &mut self.registro {
            r.extend(partes.iter().cloned());
        }
        self.audio.decir(partes);
    }
    /// Siempre en español.
    pub fn decir(&mut self, t: &str) {
        self.hablar(vec![(L::Es, t.to_string())]);
    }
    /// Varias partes, siempre en español.
    pub fn decir_es_v<A: AsRef<str>>(&mut self, partes: &[A]) {
        self.hablar(partes.iter().map(|t| (L::Es, t.as_ref().to_string())).collect());
    }
    /// Siempre en inglés.
    pub fn decir_en(&mut self, t: &str) {
        self.hablar(vec![(L::En, t.to_string())]);
    }
    /// Varias partes en inglés, siempre.
    pub fn decir_en_v(&mut self, partes: &[String]) {
        self.hablar(partes.iter().map(|t| (L::En, t.clone())).collect());
    }
    /// Español y luego inglés, siempre.
    pub fn decir_es_en(&mut self, es: &str, en: &str) {
        self.hablar(vec![(L::Es, es.to_string()), (L::En, en.to_string())]);
    }
    /// Según el idioma elegido en el escritorio.
    pub fn decir_bil(&mut self, es: &str, en: &str) {
        self.decir_v(&[es], &[en]);
    }
    /// Varias partes por idioma (así se combinan frases pregrabadas).
    pub fn decir_v<A: AsRef<str>, B: AsRef<str>>(&mut self, es: &[A], en: &[B]) {
        let mut p: Vec<Parte> = Vec::new();
        if self.idioma != Idioma::En {
            p.extend(es.iter().map(|t| (L::Es, t.as_ref().to_string())));
        }
        if self.idioma != Idioma::Es {
            p.extend(en.iter().map(|t| (L::En, t.as_ref().to_string())));
        }
        self.hablar(p);
    }
    pub fn decir_num(&mut self, n: usize) {
        self.decir_bil(crate::apps::num(n), crate::apps::num_en(n));
    }
    pub fn decir_obj(&mut self, o: Obj) {
        self.decir_bil(o.singular(), o.ingles());
    }
}
