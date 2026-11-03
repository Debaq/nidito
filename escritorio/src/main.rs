//! Nidito: escritorio infantil. Dibuja todo por CPU (tiny-skia) en una ventana X11 a pantalla completa.
#![cfg_attr(target_arch = "wasm32", allow(dead_code, unused_imports))]

mod almacen;
mod apps;
mod captura;
mod config;
mod gatitos;
mod palabras;
mod cx;
mod dibujos;
mod efectos;
mod escritorio;
mod gfx;
mod padres;
mod sonidos;
mod tiempo;
mod voz;
#[cfg(not(target_arch = "wasm32"))]
mod vozgen;
#[cfg(not(target_arch = "wasm32"))]
mod instalador;
#[cfg(not(target_arch = "wasm32"))]
mod arranque;

use apps::{Ev, T};
use escritorio::{Accion, Escritorio};
use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Duration;
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, TouchPhase, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowId};

pub const W: u32 = 1024;
pub const H: u32 = 600;
/// Versión + fecha de compilación (la pone build.rs): se ve en el menú de adultos y en `--version`.
pub const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("NIDITO_COMPILADO"), ")");

struct Vent {
    sup: softbuffer::Surface<Rc<Window>, Rc<Window>>,
    _ctx: softbuffer::Context<Rc<Window>>,
    win: Rc<Window>,
    tam: (u32, u32),
}

#[derive(Clone, Copy)]
struct Mapeo {
    esc: f32,
    ox: f32,
    oy: f32,
}

struct Programa {
    vent: Option<Vent>,
    lienzo: gfx::Lienzo,
    cx: cx::Cx,
    esc: Escritorio,
    ultimo: Instant,
    proximo: Instant,
    mods: ModifiersState,
    mapeo: Mapeo,
    codigo_salida: i32,
    desarrollo: bool,
    tabla_x: Vec<i32>,
}

impl Programa {
    fn nuevo() -> Programa {
        let cfg = config::cargar();
        let mut lienzo = gfx::Lienzo::nuevo(W, H);
        let esc = Escritorio::nuevo(&mut lienzo, &cfg);
        Programa {
            vent: None,
            lienzo,
            cx: cx::Cx::nuevo(&cfg, true),
            esc,
            ultimo: Instant::now(),
            proximo: Instant::now(),
            mods: ModifiersState::empty(),
            mapeo: Mapeo { esc: 1.0, ox: 0.0, oy: 0.0 },
            codigo_salida: 0,
            desarrollo: std::env::var_os("NIDITO_VENTANA").is_some(),
            tabla_x: vec![],
        }
    }

    fn a_lienzo(&self, x: f64, y: f64) -> (f32, f32) {
        let m = self.mapeo;
        (((x as f32 - m.ox) / m.esc).clamp(0.0, W as f32 - 1.0), ((y as f32 - m.oy) / m.esc).clamp(0.0, H as f32 - 1.0))
    }

    fn accion(&mut self, a: Accion, el: &ActiveEventLoop) {
        match a {
            Accion::Salir(c) => {
                self.esc.cerrar_todo(&mut self.cx);
                self.codigo_salida = c;
                el.exit();
            }
            #[cfg(target_arch = "wasm32")]
            Accion::Apagar | Accion::Reiniciar => {
                // en el navegador "apagar" vuelve a empezar
                if let Some(w) = web_sys::window() {
                    let _ = w.location().reload();
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            Accion::Apagar => {
                // deja que suene la despedida (atendiendo el audio, si no la voz nunca empieza)
                let fin = Instant::now() + Duration::from_millis(1800);
                while Instant::now() < fin {
                    self.cx.audio.actualizar();
                    std::thread::sleep(Duration::from_millis(30));
                }
                if self.desarrollo {
                    el.exit();
                } else {
                    let _ = std::process::Command::new("systemctl").arg("poweroff").spawn();
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            Accion::Reiniciar => {
                if self.desarrollo {
                    el.exit();
                } else {
                    let _ = std::process::Command::new("systemctl").arg("reboot").spawn();
                }
            }
        }
    }

    fn enviar(&mut self, e: Ev, el: &ActiveEventLoop) {
        if let Some(a) = self.esc.evento(e, &mut self.cx) {
            self.accion(a, el);
        }
    }

    fn cuadro(&mut self) {
        let ahora = Instant::now();
        let dt = (ahora - self.ultimo).as_secs_f32().min(0.1);
        self.ultimo = ahora;
        self.cx.t += dt;
        self.cx.audio.actualizar();
        self.esc.actualizar(dt, &mut self.cx);
        if self.esc.necesita_cuadro(dt) {
            self.esc.dibujar(&mut self.lienzo, &self.cx);
            self.esc.dibujar_cursor(&mut self.lienzo, &self.cx);
            self.presentar();
        }
    }

    fn presentar(&mut self) {
        let Some(v) = self.vent.as_mut() else { return };
        let s = v.win.inner_size();
        let (ww, wh) = (s.width.max(1), s.height.max(1));
        if v.tam != (ww, wh) {
            let _ = v.sup.resize(NonZeroU32::new(ww).unwrap(), NonZeroU32::new(wh).unwrap());
            v.tam = (ww, wh);
            let esc = (ww as f32 / W as f32).min(wh as f32 / H as f32);
            let (dw, dh) = (W as f32 * esc, H as f32 * esc);
            self.mapeo = Mapeo { esc, ox: ((ww as f32 - dw) / 2.0).floor(), oy: ((wh as f32 - dh) / 2.0).floor() };
            self.tabla_x = (0..ww)
                .map(|x| {
                    let sx = ((x as f32 - self.mapeo.ox) / esc).floor() as i32;
                    if sx >= 0 && sx < W as i32 {
                        sx
                    } else {
                        -1
                    }
                })
                .collect();
        }
        let Ok(mut buf) = v.sup.buffer_mut() else { return };
        let src = self.lienzo.pix.data();
        if ww == W && wh == H {
            for (px, s) in buf.iter_mut().zip(src.chunks_exact(4)) {
                *px = (s[0] as u32) << 16 | (s[1] as u32) << 8 | s[2] as u32;
            }
        } else {
            let m = self.mapeo;
            for y in 0..wh {
                let sy = ((y as f32 - m.oy) / m.esc).floor() as i32;
                let fila = &mut buf[(y * ww) as usize..((y + 1) * ww) as usize];
                if sy < 0 || sy >= H as i32 {
                    fila.fill(0);
                    continue;
                }
                let base = (sy as usize) * W as usize * 4;
                for (px, &sx) in fila.iter_mut().zip(self.tabla_x.iter()) {
                    *px = if sx < 0 {
                        0
                    } else {
                        let i = base + sx as usize * 4;
                        (src[i] as u32) << 16 | (src[i + 1] as u32) << 8 | src[i + 2] as u32
                    };
                }
            }
        }
        let _ = buf.present();
    }
}

impl ApplicationHandler for Programa {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.vent.is_some() {
            return;
        }
        let mut attrs = Window::default_attributes().with_title("Nidito");
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;
            let canvas = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("nidito"))
                .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok());
            attrs = attrs.with_canvas(canvas).with_focusable(true).with_prevent_default(true);
        }
        #[cfg(not(target_arch = "wasm32"))]
        if self.desarrollo {
            attrs = attrs.with_inner_size(PhysicalSize::new(W, H));
        } else {
            let (mw, mh) = el
                .primary_monitor()
                .or_else(|| el.available_monitors().next())
                .map(|m| (m.size().width, m.size().height))
                .unwrap_or((W, H));
            attrs = attrs
                .with_decorations(false)
                .with_inner_size(PhysicalSize::new(mw, mh))
                .with_position(PhysicalPosition::new(0, 0))
                .with_fullscreen(Some(Fullscreen::Borderless(None)));
        }
        let win = Rc::new(el.create_window(attrs).expect("no se pudo crear la ventana"));
        #[cfg(target_arch = "wasm32")]
        win.focus_window();
        win.set_cursor_visible(false);
        win.focus_window();
        let ctx = softbuffer::Context::new(win.clone()).expect("softbuffer");
        let sup = softbuffer::Surface::new(&ctx, win.clone()).expect("superficie");
        self.vent = Some(Vent { sup, _ctx: ctx, win, tam: (0, 0) });
        self.ultimo = Instant::now();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                if self.desarrollo {
                    self.esc.cerrar_todo(&mut self.cx);
                    el.exit();
                }
            }
            WindowEvent::RedrawRequested => self.cuadro(),
            WindowEvent::CursorMoved { position, .. } => {
                let p = self.a_lienzo(position.x, position.y);
                self.cx.raton = p;
                self.enviar(Ev::Mueve(p.0, p.1), el);
            }
            WindowEvent::MouseInput { state, .. } => {
                // cualquier botón cuenta como clic (panel táctil, botón derecho, etc.)
                let p = self.cx.raton;
                match state {
                    ElementState::Pressed => {
                        self.cx.apretado = true;
                        self.enviar(Ev::Clic(p.0, p.1), el);
                    }
                    ElementState::Released => {
                        self.cx.apretado = false;
                        self.enviar(Ev::Suelta(p.0, p.1), el);
                    }
                }
            }
            // pantallas táctiles (tablets, celulares en la versión web): un dedo = el mouse
            WindowEvent::Touch(t) => {
                let p = self.a_lienzo(t.location.x, t.location.y);
                self.cx.raton = p;
                match t.phase {
                    TouchPhase::Started => {
                        self.enviar(Ev::Mueve(p.0, p.1), el);
                        self.cx.apretado = true;
                        self.enviar(Ev::Clic(p.0, p.1), el);
                    }
                    TouchPhase::Moved => self.enviar(Ev::Mueve(p.0, p.1), el),
                    TouchPhase::Ended | TouchPhase::Cancelled => {
                        self.cx.apretado = false;
                        self.enviar(Ev::Suelta(p.0, p.1), el);
                    }
                }
            }
            WindowEvent::ModifiersChanged(m) => self.mods = m.state(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }
                if !cfg!(target_arch = "wasm32") && self.mods.control_key() && self.mods.alt_key() && event.physical_key == PhysicalKey::Code(KeyCode::KeyP) {
                    self.esc.abrir_padres(&mut self.cx);
                    return;
                }
                if event.repeat {
                    return;
                }
                // botón de encendido (llega como tecla "Power")
                if event.logical_key == Key::Named(NamedKey::Power) || event.physical_key == PhysicalKey::Code(KeyCode::Power) {
                    if let Some(a) = self.esc.boton_apagar(&mut self.cx) {
                        self.accion(a, el);
                    }
                    return;
                }
                let e = match &event.logical_key {
                    Key::Named(n) => Ev::Tecla(match n {
                        NamedKey::Enter => T::Enter,
                        NamedKey::Backspace => T::Borrar,
                        NamedKey::Space => T::Espacio,
                        NamedKey::Escape => T::Escape,
                        NamedKey::ArrowLeft => T::Izq,
                        NamedKey::ArrowRight => T::Der,
                        _ => T::Otra,
                    }),
                    _ => match &event.text {
                        Some(t) if t.chars().any(|c| !c.is_control()) => Ev::Texto(t.to_string()),
                        _ => return,
                    },
                };
                self.enviar(e, el);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        let ahora = Instant::now();
        if ahora >= self.proximo {
            if let Some(v) = &self.vent {
                v.win.request_redraw();
            }
            self.proximo = ahora + Duration::from_millis(33);
        }
        el.set_control_flow(ControlFlow::WaitUntil(self.proximo));
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use winit::platform::web::EventLoopExtWebSys;
    console_error_panic_hook::set_once();
    let el = EventLoop::new().expect("no se pudo iniciar");
    el.spawn_app(Programa::nuevo());
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("--captura") => {
            captura::ejecutar(args.get(2).map(|s| s.as_str()).unwrap_or("capturas"));
            return;
        }
        Some("--frases") => {
            // nidito --frases [archivo.conf]
            let mut cfg = config::Config::default();
            match args.get(2) {
                Some(p) => config::leer_en(&mut cfg, &std::fs::read_to_string(p).unwrap_or_default()),
                None => cfg = config::cargar(),
            }
            captura::frases(&cfg.nombre, &cfg.apodo);
            return;
        }
        Some("--hash-clave") => {
            // nidito --hash-clave <clave>  → líneas sal= y clave= para /etc/nidito.conf
            let clave = args.get(2).cloned().unwrap_or_default();
            let sal = format!("{:016x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(1) as u64 ^ 0x5bd1e995);
            println!("sal={sal}");
            println!("clave={}", tiempo::hash(&sal, &clave));
            return;
        }
        Some("--arranque") => {
            // nidito --arranque <carpeta>   (imágenes de la pantalla de carga)
            if let Err(e) = arranque::generar(args.get(2).map(|s| s.as_str()).unwrap_or("arranque")) {
                eprintln!("no se pudo: {e}");
                std::process::exit(1);
            }
            return;
        }
        Some("--instalador") => {
            instalador::ejecutar(args.get(2).map(|s| s.as_str()) == Some("--simular"));
            return;
        }
        Some("--instalador-capturas") => {
            instalador::capturas(args.get(2).map(|s| s.as_str()).unwrap_or("capturas-instalador"));
            return;
        }
        Some("--voces-nombre") => {
            // nidito --voces-nombre <nidito.conf> <carpeta-voces> <carpeta-piper> <modelo-es.onnx> <modelo-en.onnx>
            let a: Vec<std::path::PathBuf> = args[2..].iter().map(Into::into).collect();
            if a.len() != 5 {
                eprintln!("uso: nidito --voces-nombre <nidito.conf> <carpeta-voces> <carpeta-piper> <modelo-es.onnx> <modelo-en.onnx>");
                std::process::exit(2);
            }
            match vozgen::generar(&a[0], &a[1], &a[2], &a[3], &a[4]) {
                Ok(n) => println!("LISTO {n}"),
                Err(e) => {
                    eprintln!("ERROR {e}");
                    std::process::exit(1);
                }
            }
            return;
        }
        Some("--empaquetar-voces") => {
            // nidito --empaquetar-voces <carpeta-con-ogg> <voces.bin>   (para la versión web)
            let (Some(dir), Some(salida)) = (args.get(2), args.get(3)) else {
                eprintln!("uso: nidito --empaquetar-voces <carpeta> <voces.bin>");
                std::process::exit(2);
            };
            match voz::empaquetar(std::path::Path::new(dir), std::path::Path::new(salida)) {
                Ok(n) => println!("{n} audios en {salida}"),
                Err(e) => {
                    eprintln!("no se pudo empaquetar: {e}");
                    std::process::exit(1);
                }
            }
            return;
        }
        Some("--version") => {
            println!("nidito {VERSION}");
            return;
        }
        _ => {}
    }
    let el = EventLoop::new().expect("no hay servidor gráfico (DISPLAY)");
    let mut prog = Programa::nuevo();
    let _ = el.run_app(&mut prog);
    std::process::exit(prog.codigo_salida);
}
