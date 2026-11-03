//! Cámara: se ve en espejo, cuenta 3-2-1 y saca la foto. Las fotos se guardan en ~/Dibujos
//! (aparecen en "Mis dibujos"). La app solo aparece si hay una cámara (V4L2) conectada.

use super::*;
use crate::cx::Son;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use tiny_skia::{FilterQuality, IntSize, Pixmap, PixmapPaint, Transform};
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::prelude::*;
use v4l::video::Capture;
use v4l::FourCC;

const ANCHO: u32 = 640;
const ALTO: u32 = 480;

/// Primera cámara que sirve (captura de video en YUYV o MJPG), buscada una sola vez.
pub fn camara() -> Option<PathBuf> {
    static CAM: OnceLock<Option<PathBuf>> = OnceLock::new();
    CAM.get_or_init(|| {
        (0..10).map(|i| PathBuf::from(format!("/dev/video{i}"))).filter(|p| p.exists()).find(|p| {
            let Ok(dev) = Device::with_path(p) else { return false };
            let captura = dev.query_caps().map(|c| c.capabilities.contains(v4l::capability::Flags::VIDEO_CAPTURE)).unwrap_or(false);
            let formatos = dev.enum_formats().unwrap_or_default();
            captura && formatos.iter().any(|f| f.fourcc == FourCC::new(b"YUYV") || f.fourcc == FourCC::new(b"MJPG"))
        })
    })
    .clone()
}

pub fn hay_camara() -> bool {
    camara().is_some()
}

pub fn frases(f: &mut Frases) {
    f.bil("Cámara", "Camera");
    f.bil("¡Sonríe!", "Smile!");
    f.bil("¡Qué linda foto!", "What a nice photo!");
    f.bil("La foto se guardó en Mis dibujos.", "The photo was saved in My drawings.");
}

/// Último cuadro de la cámara (RGBA, ya en espejo).
type Cuadro = Arc<Mutex<Option<(u32, u32, Vec<u8>)>>>;

fn yuyv_a_rgba(src: &[u8], w: u32, h: u32) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let mut out = vec![255u8; w * h * 4];
    let clamp = |v: i32| v.clamp(0, 255) as u8;
    for y in 0..h {
        let fila = &src[y * w * 2..(y + 1) * w * 2];
        for x2 in 0..w / 2 {
            let i = x2 * 4;
            let (y0, u, y1, v) = (fila[i] as i32, fila[i + 1] as i32 - 128, fila[i + 2] as i32, fila[i + 3] as i32 - 128);
            let (dr, dg, db) = ((359 * v) >> 8, (88 * u + 183 * v) >> 8, (454 * u) >> 8);
            for (k, yy) in [(0, y0), (1, y1)] {
                let x = w - 1 - (x2 * 2 + k); // espejo
                let o = (y * w + x) * 4;
                out[o] = clamp(yy + dr);
                out[o + 1] = clamp(yy - dg);
                out[o + 2] = clamp(yy + db);
            }
        }
    }
    out
}

fn mjpg_a_rgba(src: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    use zune_jpeg::zune_core::{colorspace::ColorSpace, options::DecoderOptions};
    let mut d = zune_jpeg::JpegDecoder::new_with_options(src, DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA));
    let mut px = d.decode().ok()?;
    let (w, h) = d.dimensions()?;
    for fila in px.chunks_mut(w * 4) {
        for x in 0..w / 2 {
            for c in 0..4 {
                fila.swap(x * 4 + c, (w - 1 - x) * 4 + c);
            }
        }
    }
    Some((w as u32, h as u32, px))
}

/// Hilo que lee la cámara hasta que `parar` se active.
fn capturar(ruta: PathBuf, cuadro: Cuadro, parar: Arc<AtomicBool>, error: Arc<AtomicBool>) {
    let r = (|| -> std::io::Result<()> {
        let dev = Device::with_path(&ruta)?;
        let mut fmt = dev.format()?;
        fmt.width = ANCHO;
        fmt.height = ALTO;
        fmt.fourcc = FourCC::new(b"YUYV");
        let mut fmt = dev.set_format(&fmt)?;
        if fmt.fourcc != FourCC::new(b"YUYV") {
            fmt.fourcc = FourCC::new(b"MJPG");
            fmt = dev.set_format(&fmt)?;
        }
        let yuyv = fmt.fourcc == FourCC::new(b"YUYV");
        let mut stream = MmapStream::with_buffers(&dev, Type::VideoCapture, 4)?;
        while !parar.load(Ordering::Relaxed) {
            let (buf, meta) = stream.next()?;
            let datos = &buf[..(meta.bytesused as usize).min(buf.len())];
            let img = if yuyv {
                (datos.len() >= (fmt.width * fmt.height * 2) as usize).then(|| (fmt.width, fmt.height, yuyv_a_rgba(datos, fmt.width, fmt.height)))
            } else {
                mjpg_a_rgba(datos)
            };
            if let Some(img) = img {
                *cuadro.lock().unwrap() = Some(img);
            }
        }
        Ok(())
    })();
    if let Err(e) = r {
        eprintln!("nidito: cámara: {e}");
        error.store(true, Ordering::Relaxed);
    }
}

enum Estado {
    Viendo,
    Contando(f32),
    Mostrando(f32),
}

pub struct Camara {
    cuadro: Cuadro,
    parar: Arc<AtomicBool>,
    error: Arc<AtomicBool>,
    vista: Option<Pixmap>,
    foto: Option<Pixmap>,
    estado: Estado,
    flash: f32,
    fotos: usize,
}

const PANTALLA: R = R::new(24.0, 19.0, 640.0, 480.0);
const BOTON: (f32, f32) = (836.0, 220.0);
const BOTON_R: f32 = 78.0;

impl Camara {
    pub fn nuevo(cx: &mut Cx) -> Box<dyn App> {
        let cuadro: Cuadro = Arc::new(Mutex::new(None));
        let parar = Arc::new(AtomicBool::new(false));
        let error = Arc::new(AtomicBool::new(false));
        match camara() {
            Some(ruta) => {
                let (c, p, e) = (cuadro.clone(), parar.clone(), error.clone());
                std::thread::spawn(move || capturar(ruta, c, p, e));
            }
            None => error.store(true, Ordering::Relaxed),
        }
        cx.decir_bil("¡Sonríe!", "Smile!");
        Box::new(Camara { cuadro, parar, error, vista: None, foto: None, estado: Estado::Viendo, flash: 0.0, fotos: 0 })
    }

    fn disparar(&mut self, cx: &mut Cx) {
        if matches!(self.estado, Estado::Viendo) && self.vista.is_some() {
            self.estado = Estado::Contando(3.0);
            cx.decir_num(3);
            cx.sonar(Son::Tic);
        }
    }

    fn sacar(&mut self, cx: &mut Cx) {
        let Some(img) = self.vista.clone() else { return };
        let dir = ruta_datos("Dibujos");
        let _ = std::fs::create_dir_all(&dir);
        let seg = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let _ = img.save_png(dir.join(format!("foto-{seg}.png")));
        self.foto = Some(img);
        self.fotos += 1;
        self.flash = 1.0;
        self.estado = Estado::Mostrando(2.5);
        cx.sonar(Son::Clic);
        if self.fotos == 1 {
            cx.decir_v(&["¡Qué linda foto!", "La foto se guardó en Mis dibujos."], &["What a nice photo!", "The photo was saved in My drawings."]);
        } else {
            cx.decir_bil("¡Qué linda foto!", "What a nice photo!");
        }
    }
}

impl Drop for Camara {
    fn drop(&mut self) {
        self.parar.store(true, Ordering::Relaxed);
    }
}

impl App for Camara {
    fn evento(&mut self, ev: &Ev, cx: &mut Cx) {
        match ev {
            Ev::Clic(x, y) if dist((*x, *y), BOTON) < BOTON_R || PANTALLA.dentro((*x, *y)) => self.disparar(cx),
            Ev::Tecla(T::Espacio) | Ev::Tecla(T::Enter) => self.disparar(cx),
            _ => {}
        }
    }

    fn actualizar(&mut self, dt: f32, cx: &mut Cx) {
        if let Some((w, h, px)) = self.cuadro.lock().unwrap().take() {
            if let Some(pm) = IntSize::from_wh(w, h).and_then(|s| Pixmap::from_vec(px, s)) {
                self.vista = Some(pm);
            }
        }
        self.flash = (self.flash - dt * 2.5).max(0.0);
        match &mut self.estado {
            Estado::Contando(t) => {
                let antes = t.ceil();
                *t -= dt;
                if *t <= 0.0 {
                    self.sacar(cx);
                } else if t.ceil() < antes {
                    cx.decir_num(t.ceil() as usize);
                    cx.sonar(Son::Tic);
                }
            }
            Estado::Mostrando(t) => {
                *t -= dt;
                if *t <= 0.0 {
                    self.estado = Estado::Viendo;
                }
            }
            Estado::Viendo => {}
        }
    }

    fn dibujar(&mut self, l: &mut Lienzo, cx: &Cx) {
        l.degradado_v(0.0, 0.0, AW, AH, rgb(52, 58, 78), rgb(30, 33, 46));
        // pantalla
        l.rrect_r(PANTALLA.crecer(6.0), 14.0, rgb(20, 22, 30));
        let mostrar = match self.estado {
            Estado::Mostrando(_) => self.foto.as_ref(),
            _ => self.vista.as_ref(),
        };
        match mostrar {
            Some(img) => {
                let esc = (PANTALLA.w / img.width() as f32).min(PANTALLA.h / img.height() as f32);
                let (w, h) = (img.width() as f32 * esc, img.height() as f32 * esc);
                let (x, y) = (PANTALLA.cx() - w / 2.0, PANTALLA.cy() - h / 2.0);
                let pp = PixmapPaint { quality: if esc == 1.0 { FilterQuality::Nearest } else { FilterQuality::Bilinear }, ..Default::default() };
                l.pix.draw_pixmap(0, 0, img.as_ref(), &pp, Transform::from_row(esc, 0.0, 0.0, esc, x, y), None);
            }
            None if self.error.load(Ordering::Relaxed) => {
                l.texto_c("No se pudo abrir la cámara", PANTALLA.cx(), PANTALLA.cy(), 32.0, BLANCO);
            }
            None => l.texto_c("Encendiendo la cámara…", PANTALLA.cx(), PANTALLA.cy(), 32.0, BLANCO),
        }
        if let Estado::Contando(t) = self.estado {
            let n = t.ceil().max(1.0) as usize;
            let f = 1.0 - (t - t.floor());
            l.circulo(PANTALLA.cx(), PANTALLA.cy(), 90.0 + 20.0 * f, alfa(NEGRO, 0.35));
            l.texto_c_borde(&n.to_string(), PANTALLA.cx(), PANTALLA.cy(), 150.0 + 30.0 * f, BLANCO, TINTA, 4.0);
        }
        if self.flash > 0.0 {
            l.rect(PANTALLA.x, PANTALLA.y, PANTALLA.w, PANTALLA.h, alfa(BLANCO, self.flash));
        }

        // botón de disparo
        let listo = matches!(self.estado, Estado::Viendo) && self.vista.is_some();
        let sobre = dist(cx.rl, BOTON) < BOTON_R && listo;
        let r = if sobre { BOTON_R * 1.06 } else { BOTON_R };
        l.circulo(BOTON.0, BOTON.1 + 5.0, r + 8.0, alfa(NEGRO, 0.4));
        l.circulo(BOTON.0, BOTON.1, r + 8.0, BLANCO);
        l.circulo(BOTON.0, BOTON.1, r, if listo { ROJO } else { GRIS });
        // ícono de cámara
        let u = r / 50.0;
        l.rrect(BOTON.0 - 30.0 * u, BOTON.1 - 18.0 * u, 60.0 * u, 40.0 * u, 8.0 * u, BLANCO);
        l.rrect(BOTON.0 - 12.0 * u, BOTON.1 - 26.0 * u, 24.0 * u, 12.0 * u, 4.0 * u, BLANCO);
        l.circulo(BOTON.0, BOTON.1 + 2.0 * u, 13.0 * u, if listo { ROJO } else { GRIS });
        l.circulo(BOTON.0, BOTON.1 + 2.0 * u, 7.0 * u, BLANCO);
        l.texto_c("¡Foto!", BOTON.0, BOTON.1 + BOTON_R + 40.0, 34.0, BLANCO);

        // última foto
        if let Some(f) = &self.foto {
            let r = R::new(BOTON.0 - 80.0, 390.0, 160.0, 120.0);
            l.rect(r.x - 4.0, r.y - 4.0, r.w + 8.0, r.h + 8.0, BLANCO);
            let esc = (r.w / f.width() as f32).min(r.h / f.height() as f32);
            let pp = PixmapPaint { quality: FilterQuality::Bilinear, ..Default::default() };
            l.pix.draw_pixmap(0, 0, f.as_ref(), &pp, Transform::from_row(esc, 0.0, 0.0, esc, r.x, r.y), None);
        }
    }
}
