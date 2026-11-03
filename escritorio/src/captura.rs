//! Modo de prueba sin ventana: simula uso y guarda capturas PNG + tiempos de dibujo.
//! `nidito --captura <dir>`  y  `nidito --frases [nombre]`

use crate::apps::{Ev, T};
use crate::config::Config;
use crate::cx::Cx;
use crate::escritorio::{self, Escritorio};
use crate::gfx::Lienzo;
use crate::{H, W};
use std::time::Instant;

fn paso(esc: &mut Escritorio, cx: &mut Cx, seg: f32) {
    let dt = 1.0 / 30.0;
    let mut t = 0.0;
    while t < seg {
        cx.t += dt;
        esc.actualizar(dt, cx);
        t += dt;
    }
}

fn ev(esc: &mut Escritorio, cx: &mut Cx, e: Ev) {
    if let Ev::Mueve(x, y) | Ev::Clic(x, y) | Ev::Suelta(x, y) = e {
        cx.raton = (x, y);
    }
    esc.evento(e, cx);
}

fn clic(esc: &mut Escritorio, cx: &mut Cx, x: f32, y: f32) {
    ev(esc, cx, Ev::Mueve(x, y));
    ev(esc, cx, Ev::Clic(x, y));
    ev(esc, cx, Ev::Suelta(x, y));
}

fn texto(esc: &mut Escritorio, cx: &mut Cx, s: &str) {
    for c in s.chars() {
        if c == ' ' {
            ev(esc, cx, Ev::Tecla(T::Espacio));
        } else {
            ev(esc, cx, Ev::Texto(c.to_string()));
        }
    }
}

fn idx(t: &str) -> usize {
    escritorio::lista().iter().position(|i| i.titulo == t).expect("app")
}

fn abrir_icono(esc: &mut Escritorio, cx: &mut Cx, t: &str) {
    let r = escritorio::icono_r(idx(t));
    clic(esc, cx, r.cx(), r.y + 48.0);
}

fn abrir_menu(esc: &mut Escritorio, cx: &mut Cx, t: &str) {
    clic(esc, cx, 60.0, 580.0);
    let r = escritorio::menu_item(idx(t));
    clic(esc, cx, r.cx(), r.cy());
}

fn foto(esc: &mut Escritorio, cx: &mut Cx, l: &mut Lienzo, dir: &str, nombre: &str) {
    let t0 = Instant::now();
    esc.dibujar(l, cx);
    esc.dibujar_cursor(l, cx);
    let ms = t0.elapsed().as_secs_f32() * 1000.0;
    // segundo cuadro: el costo estable (glifos ya en caché)
    let t1 = Instant::now();
    esc.dibujar(l, cx);
    let ms2 = t1.elapsed().as_secs_f32() * 1000.0;
    println!("{nombre:<22} primer cuadro {ms:6.1} ms   cuadro normal {ms2:6.1} ms");
    let _ = l.pix.save_png(format!("{dir}/{nombre}.png"));
}

pub fn ejecutar(dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    let cfg = Config { nombre: "Peque".into(), apodo: "Pequeñín".into(), ..Config::default() };
    let _ = std::fs::remove_dir_all(crate::apps::ruta_datos(".local/share/nidito"));
    let mut cx = Cx::nuevo(&cfg, false);
    cx.registro = Some(vec![]);
    let mut l = Lienzo::nuevo(W, H);
    let mut esc = Escritorio::nuevo(&mut l, &cfg);
    paso(&mut esc, &mut cx, 0.5);
    ev(&mut esc, &mut cx, Ev::Mueve(70.0, 70.0));
    foto(&mut esc, &mut cx, &mut l, dir, "00-escritorio");
    for _ in 0..4 {
        esc.actualizar(31.0, &mut cx); // fuerza que los gatitos se cambien de lugar
    }
    paso(&mut esc, &mut cx, 0.1);
    foto(&mut esc, &mut cx, &mut l, dir, "00b-escritorio-gatos-movidos");

    clic(&mut esc, &mut cx, 60.0, 580.0);
    ev(&mut esc, &mut cx, Ev::Mueve(120.0, 300.0));
    foto(&mut esc, &mut cx, &mut l, dir, "01-menu-inicio");
    clic(&mut esc, &mut cx, 900.0, 300.0); // cierra menú

    // Pintar: trazos y sellos
    abrir_icono(&mut esc, &mut cx, "Pintar");
    paso(&mut esc, &mut cx, 0.5);
    let (ox, oy) = (12.0, 38.0);
    ev(&mut esc, &mut cx, Ev::Mueve(ox + 300.0, oy + 200.0));
    ev(&mut esc, &mut cx, Ev::Clic(ox + 300.0, oy + 200.0));
    for i in 0..60 {
        let a = i as f32 * 0.15;
        ev(&mut esc, &mut cx, Ev::Mueve(ox + 450.0 + a.cos() * 150.0, oy + 250.0 + a.sin() * 120.0));
    }
    ev(&mut esc, &mut cx, Ev::Suelta(0.0, 0.0));
    clic(&mut esc, &mut cx, ox + 942.0 + 26.0, oy + 14.0 + 70.0 * 2.0 + 28.0); // arcoíris
    ev(&mut esc, &mut cx, Ev::Clic(ox + 200.0, oy + 400.0));
    for i in 0..40 {
        ev(&mut esc, &mut cx, Ev::Mueve(ox + 200.0 + i as f32 * 12.0, oy + 400.0 + (i as f32 * 0.4).sin() * 40.0));
    }
    ev(&mut esc, &mut cx, Ev::Suelta(0.0, 0.0));
    clic(&mut esc, &mut cx, ox + 884.0 + 26.0, oy + 14.0 + 70.0 * 3.0 + 28.0); // estrella
    clic(&mut esc, &mut cx, ox + 700.0, oy + 120.0);
    clic(&mut esc, &mut cx, ox + 884.0 + 26.0, oy + 14.0 + 70.0 * 5.0 + 28.0); // gato
    clic(&mut esc, &mut cx, ox + 260.0, oy + 120.0);
    ev(&mut esc, &mut cx, Ev::Mueve(ox + 600.0, oy + 300.0));
    paso(&mut esc, &mut cx, 0.2);
    foto(&mut esc, &mut cx, &mut l, dir, "02-pintar");

    // minimizar Pintar, abrir Letras
    clic(&mut esc, &mut cx, 1024.0 - 6.0 - 3.0 * 50.0 + 4.0 + 23.0, 19.0);
    abrir_icono(&mut esc, &mut cx, "Letras");
    paso(&mut esc, &mut cx, 0.5);
    texto(&mut esc, &mut cx, "SO");
    paso(&mut esc, &mut cx, 0.2);
    foto(&mut esc, &mut cx, &mut l, dir, "03-letras-copiar");
    clic(&mut esc, &mut cx, ox + 100.0, oy + 40.0);
    texto(&mut esc, &mut cx, "HOLA MAMA");
    ev(&mut esc, &mut cx, Ev::Tecla(T::Enter));
    texto(&mut esc, &mut cx, "TE AMO");
    foto(&mut esc, &mut cx, &mut l, dir, "04-letras-libre");

    // restaurar (ventana normal) y abrir Contar al lado
    clic(&mut esc, &mut cx, 1024.0 - 6.0 - 2.0 * 50.0 + 4.0 + 23.0, 19.0);
    paso(&mut esc, &mut cx, 0.1);
    foto(&mut esc, &mut cx, &mut l, dir, "05-ventana-normal");

    let cerrar = |esc: &mut Escritorio, cx: &mut Cx| clic(esc, cx, 1024.0 - 6.0 - 50.0 + 4.0 + 23.0, 19.0);

    let apps: [(&str, &str); 12] = [
        ("Escribir", "05b-escribir"),
        ("Contar", "06-contar"),
        ("Mi clase", "07-clase"),
        ("Cocina", "08-cocina"),
        ("English", "09-english"),
        ("Memorice", "10-memorice"),
        ("Globos", "11-globos"),
        ("Teclas", "12-teclas"),
        ("Mis dibujos", "13-mis-dibujos"),
        ("Enciclopedia", "20-enciclopedia"),
        ("Figuras", "22-figuras"),
        ("Música", "23-musica"),
    ];
    for (titulo, nombre) in apps {
        abrir_menu(&mut esc, &mut cx, titulo);
        paso(&mut esc, &mut cx, 0.5);
        match titulo {
            "Escribir" => {
                texto(&mut esc, &mut cx, "Hola mamá");
                ev(&mut esc, &mut cx, Ev::Tecla(T::Enter));
                clic(&mut esc, &mut cx, ox + 230.0 + 2.0 * 52.0 + 24.0, oy + 37.0);
                clic(&mut esc, &mut cx, ox + 400.0 + 44.0 + 20.0, oy + 37.0);
                clic(&mut esc, &mut cx, ox + 732.0 + 25.0, oy + 37.0);
                texto(&mut esc, &mut cx, "te quiero mucho");
                clic(&mut esc, &mut cx, ox + 870.0 + 28.0, oy + 84.0 + 38.0);
                ev(&mut esc, &mut cx, Ev::Tecla(T::Enter));
                clic(&mut esc, &mut cx, ox + 230.0 + 24.0, oy + 37.0);
                clic(&mut esc, &mut cx, ox + 678.0 + 25.0, oy + 37.0);
                texto(&mut esc, &mut cx, "Sofia tiene tres años y le gusta pintar con muchos colores");
            }
            "Contar" => {
                clic(&mut esc, &mut cx, ox + 500.0 - 250.0, oy + 240.0);
                clic(&mut esc, &mut cx, ox + 500.0 - 125.0, oy + 240.0);
                clic(&mut esc, &mut cx, ox + 500.0 - 125.0, oy + 170.0);
                clic(&mut esc, &mut cx, ox + 500.0 - 250.0, oy + 170.0);
            }
            "Mi clase" => {
                texto(&mut esc, &mut cx, "Los animales");
                ev(&mut esc, &mut cx, Ev::Tecla(T::Enter));
                texto(&mut esc, &mut cx, "el gato");
                clic(&mut esc, &mut cx, ox + 878.0 + 27.0 + 58.0, oy + 86.0 + 84.0 * 2.0 + 38.0); // perro
                clic(&mut esc, &mut cx, ox + 126.0 + 600.0, oy + 86.0 + 300.0);
                clic(&mut esc, &mut cx, ox + 878.0 + 27.0, oy + 86.0 + 84.0 * 2.0 + 38.0); // gato
                clic(&mut esc, &mut cx, ox + 126.0 + 450.0, oy + 86.0 + 300.0);
                clic(&mut esc, &mut cx, ox + 12.0 + 50.0, oy + 86.0 + 66.0 + 28.0); // nueva lámina
                texto(&mut esc, &mut cx, "Los colores");
                paso(&mut esc, &mut cx, 0.2);
                foto(&mut esc, &mut cx, &mut l, dir, "07-clase");
                clic(&mut esc, &mut cx, ox + 736.0 + 90.0, oy + 14.0 + 28.0); // presentar
                paso(&mut esc, &mut cx, 0.5);
                ev(&mut esc, &mut cx, Ev::Mueve(ox + 400.0, oy + 200.0));
                foto(&mut esc, &mut cx, &mut l, dir, "07b-presentar");
                clic(&mut esc, &mut cx, ox + 500.0, oy + 200.0);
                clic(&mut esc, &mut cx, ox + 500.0, oy + 200.0);
                paso(&mut esc, &mut cx, 0.6);
                foto(&mut esc, &mut cx, &mut l, dir, "07c-fin-clase");
                cerrar(&mut esc, &mut cx);
                continue;
            }
            "Cocina" => {
                foto(&mut esc, &mut cx, &mut l, dir, "08-cocina");
                clic(&mut esc, &mut cx, ox + 22.0 + 90.0, oy + 200.0); // panqueques
                paso(&mut esc, &mut cx, 0.3);
                for _ in 0..2 {
                    for k in 0..3 {
                        clic(&mut esc, &mut cx, ox + 24.0 + 85.0, oy + 96.0 + k as f32 * 140.0 + 60.0);
                    }
                }
                paso(&mut esc, &mut cx, 0.25);
                foto(&mut esc, &mut cx, &mut l, dir, "08b-cocina-poner");
                cerrar(&mut esc, &mut cx);
                continue;
            }
            "Enciclopedia" => {
                foto(&mut esc, &mut cx, &mut l, dir, nombre);
                clic(&mut esc, &mut cx, ox + 335.0, oy + 270.0);
                paso(&mut esc, &mut cx, 0.2);
                foto(&mut esc, &mut cx, &mut l, dir, "20a-enciclopedia-animales");
                clic(&mut esc, &mut cx, ox + 110.0, oy + 124.0);
                paso(&mut esc, &mut cx, 0.3);
                foto(&mut esc, &mut cx, &mut l, dir, "20b-enciclopedia-perro");
                for _ in 0..17 {
                    ev(&mut esc, &mut cx, Ev::Tecla(T::Der));
                }
                paso(&mut esc, &mut cx, 0.3);
                foto(&mut esc, &mut cx, &mut l, dir, "20c-enciclopedia-tortuga");
                ev(&mut esc, &mut cx, Ev::Tecla(T::Escape));
                ev(&mut esc, &mut cx, Ev::Tecla(T::Escape));
                clic(&mut esc, &mut cx, ox + 665.0, oy + 270.0);
                paso(&mut esc, &mut cx, 0.2);
                foto(&mut esc, &mut cx, &mut l, dir, "20d-enciclopedia-cuerpo");
                clic(&mut esc, &mut cx, ox + 110.0 + 195.0, oy + 124.0);
                paso(&mut esc, &mut cx, 0.3);
                foto(&mut esc, &mut cx, &mut l, dir, "20e-enciclopedia-corazon");
                for _ in 0..2 {
                    ev(&mut esc, &mut cx, Ev::Tecla(T::Izq));
                }
                paso(&mut esc, &mut cx, 0.3);
                foto(&mut esc, &mut cx, &mut l, dir, "20f-enciclopedia-pies");
                cerrar(&mut esc, &mut cx);
                continue;
            }
            "Figuras" => {
                clic(&mut esc, &mut cx, ox + 30.0 + 240.0 * 2.0 + 110.0, oy + 92.0 + 99.0); // triángulo
                paso(&mut esc, &mut cx, 0.4);
                foto(&mut esc, &mut cx, &mut l, dir, nombre);
                clic(&mut esc, &mut cx, ox + 222.0 + 95.0, oy + 41.0); // jugar
                clic(&mut esc, &mut cx, ox + 80.0 + 130.0, oy + 300.0);
                clic(&mut esc, &mut cx, ox + 80.0 + 290.0 + 130.0, oy + 300.0);
                paso(&mut esc, &mut cx, 0.7);
                ev(&mut esc, &mut cx, Ev::Mueve(ox + 900.0, oy + 480.0));
                foto(&mut esc, &mut cx, &mut l, dir, "22b-figuras-jugar");
                cerrar(&mut esc, &mut cx);
                continue;
            }
            "Música" => {
                texto(&mut esc, &mut cx, "135");
                paso(&mut esc, &mut cx, 0.1);
                foto(&mut esc, &mut cx, &mut l, dir, nombre);
                clic(&mut esc, &mut cx, ox + 222.0 + 100.0, oy + 41.0); // Estrellita
                for b in [0.0, 0.0, 4.0, 4.0] {
                    clic(&mut esc, &mut cx, ox + 58.0 + b * 112.0 + 46.0, oy + 300.0);
                }
                paso(&mut esc, &mut cx, 0.1);
                ev(&mut esc, &mut cx, Ev::Mueve(ox + 900.0, oy + 480.0));
                foto(&mut esc, &mut cx, &mut l, dir, "23b-musica-estrellita");
                cerrar(&mut esc, &mut cx);
                continue;
            }
            "English" => {
                clic(&mut esc, &mut cx, ox + 150.0 + 110.0, oy + 88.0 + 98.0);
                paso(&mut esc, &mut cx, 0.3);
                foto(&mut esc, &mut cx, &mut l, dir, nombre);
                clic(&mut esc, &mut cx, ox + 222.0 + 90.0, oy + 40.0);
                paso(&mut esc, &mut cx, 0.2);
                foto(&mut esc, &mut cx, &mut l, dir, "09b-english-jugar");
                cerrar(&mut esc, &mut cx);
                continue;
            }
            "Memorice" => {
                clic(&mut esc, &mut cx, ox + 300.0, oy + 150.0);
                clic(&mut esc, &mut cx, ox + 500.0, oy + 380.0);
                paso(&mut esc, &mut cx, 0.4);
            }
            "Globos" => paso(&mut esc, &mut cx, 4.0),
            "Teclas" => {
                texto(&mut esc, &mut cx, "sofia!?");
                clic(&mut esc, &mut cx, ox + 700.0, oy + 300.0);
                paso(&mut esc, &mut cx, 0.25);
            }
            _ => {}
        }
        foto(&mut esc, &mut cx, &mut l, dir, nombre);
        cerrar(&mut esc, &mut cx);
    }
    // cámara (solo si hay una): espera cuadros reales, cuenta 3-2-1 y saca la foto
    if crate::apps::camara::hay_camara() {
        abrir_menu(&mut esc, &mut cx, "Cámara");
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            paso(&mut esc, &mut cx, 0.1);
        }
        foto(&mut esc, &mut cx, &mut l, dir, "21-camara");
        ev(&mut esc, &mut cx, Ev::Tecla(T::Espacio));
        paso(&mut esc, &mut cx, 1.2);
        foto(&mut esc, &mut cx, &mut l, dir, "21b-camara-cuenta");
        for _ in 0..20 {
            paso(&mut esc, &mut cx, 0.1);
        }
        foto(&mut esc, &mut cx, &mut l, dir, "21c-camara-foto");
        cerrar(&mut esc, &mut cx);
    }


    // diálogo de apagar
    clic(&mut esc, &mut cx, 60.0, 580.0);
    clic(&mut esc, &mut cx, escritorio::MENU_APAGAR.cx(), escritorio::MENU_APAGAR.cy());
    foto(&mut esc, &mut cx, &mut l, dir, "14-apagar");
    clic(&mut esc, &mut cx, escritorio::DLG_NO.cx(), escritorio::DLG_NO.cy());
    esc.abrir_padres(&mut cx);
    foto(&mut esc, &mut cx, &mut l, dir, "15-padres");
    ev(&mut esc, &mut cx, Ev::Tecla(T::Escape));
    esc.abrir_ajustes(&mut cx);
    ev(&mut esc, &mut cx, Ev::Mueve(0.0, 0.0));
    foto(&mut esc, &mut cx, &mut l, dir, "15b-ajustes");
    ev(&mut esc, &mut cx, Ev::Tecla(T::Escape));

    // pausa para los ojos (cada 1 minuto de uso activo)
    {
        let cfg_p = Config { pausa_ojos: 1, ..cfg.clone() };
        let mut e2 = Escritorio::nuevo(&mut l, &cfg_p);
        for _ in 0..14 {
            ev(&mut e2, &mut cx, Ev::Mueve(500.0, 300.0));
            paso(&mut e2, &mut cx, 5.0);
        }
        paso(&mut e2, &mut cx, 3.0);
        foto(&mut e2, &mut cx, &mut l, dir, "15c-pausa-ojos");
    }
    // cumpleaños hoy
    {
        use chrono::Datelike;
        let hoy = chrono::Local::now();
        let cfg_c = Config { cumpleanos: Some((hoy.day(), hoy.month())), ..cfg.clone() };
        let mut e3 = Escritorio::nuevo(&mut l, &cfg_c);
        paso(&mut e3, &mut cx, 4.0);
        foto(&mut e3, &mut cx, &mut l, dir, "15d-cumpleanos");
    }

    // bloqueo con clave y tiempo de uso
    let cfg2 = Config { sal: "s".into(), clave: crate::tiempo::hash("s", "1234"), juegos_min: 1, ..cfg.clone() };
    {
        let mut e4 = Escritorio::nuevo(&mut l, &cfg2);
        e4.abrir_padres(&mut cx);
        texto(&mut e4, &mut cx, "12");
        foto(&mut e4, &mut cx, &mut l, dir, "15e-padres-clave");
    }
    let mut esc = Escritorio::nuevo(&mut l, &cfg2);
    paso(&mut esc, &mut cx, 0.3);
    texto(&mut esc, &mut cx, "12");
    foto(&mut esc, &mut cx, &mut l, dir, "16-bloqueo");
    texto(&mut esc, &mut cx, "34");
    ev(&mut esc, &mut cx, Ev::Tecla(T::Enter));
    foto(&mut esc, &mut cx, &mut l, dir, "17-elegir-tiempo");
    clic(&mut esc, &mut cx, 122.0 + 158.0 + 70.0, 368.0); // 30 min
    abrir_menu(&mut esc, &mut cx, "Globos");
    for _ in 0..70 {
        paso(&mut esc, &mut cx, 1.0);
        ev(&mut esc, &mut cx, Ev::Mueve(500.0, 300.0));
    }
    ev(&mut esc, &mut cx, Ev::Mueve(900.0, 500.0));
    foto(&mut esc, &mut cx, &mut l, dir, "18-juegos-dormidos");
    for _ in 0..(30 * 60 / 30) {
        paso(&mut esc, &mut cx, 30.0);
        ev(&mut esc, &mut cx, Ev::Mueve(500.0, 300.0));
    }
    paso(&mut esc, &mut cx, 1.0);
    foto(&mut esc, &mut cx, &mut l, dir, "19-fin-tiempo");

    // frases pedidas que no están en el catálogo
    let catalogo: std::collections::HashSet<_> = crate::apps::catalogo("Peque", "Pequeñín").into_iter().collect();
    let pedidas = cx.registro.take().unwrap_or_default();
    let faltan: Vec<_> = pedidas.iter().filter(|p| !catalogo.contains(*p)).collect();
    println!("frases dichas: {}  fuera del catálogo: {}", pedidas.len(), faltan.len());
    for f in faltan {
        println!("  FALTA {:?}", f);
    }
}

/// Imprime el catálogo: archivo \t idioma \t texto
pub fn frases(nombre: &str, apodo: &str) {
    for (l, t) in crate::apps::catalogo(nombre, apodo) {
        println!("{}\t{}\t{}", crate::voz::archivo(l, &t), l.codigo(), t);
    }
}
