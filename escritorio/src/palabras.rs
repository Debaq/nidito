//! Vocabulario pregrabado para leer lo que el niño escribe (Letras, Escribir) con la voz buena.
//! Si todas las palabras de un texto están grabadas, se leen una tras otra; si no, se usa espeak.
//! Se compara sin tildes ni mayúsculas ("mama" -> "mamá"), porque los niños casi no ponen tildes.

use crate::apps::{base, Frases};
use crate::cx::Cx;

pub const COMUNES: &[&str] = &[
    // familia y personas
    "mamá", "papá", "abuela", "abuelo", "tía", "tío", "hermana", "hermano", "bebé", "prima", "primo",
    "amiga", "amigo", "familia", "profesora", "profesor", "niña", "niño", "yo", "tú", "nosotros",
    // saludos y palabras chicas
    "hola", "chao", "sí", "no", "gracias", "por", "favor", "el", "la", "los", "las", "un", "una", "y", "o",
    "de", "del", "en", "con", "para", "mi", "mis", "tu", "tus", "su", "es", "son", "me", "te", "a", "al",
    "que", "muy", "más", "mucho", "mucha", "muchos", "poco", "todo", "todos", "hoy", "mañana", "ayer",
    // sentimientos
    "quiero", "te", "amo", "gusta", "amor", "beso", "besos", "abrazo", "feliz", "contenta", "contento",
    "triste", "lindo", "linda", "bonito", "bonita", "bueno", "buena", "rico", "rica", "grande", "chico",
    "chica", "pequeño", "pequeña",
    // cosas
    "casa", "sol", "luna", "estrella", "estrellas", "flor", "flores", "árbol", "agua", "auto", "pelota",
    "muñeca", "juguete", "libro", "colegio", "jardín", "cama", "mesa", "silla", "plaza", "playa", "mar",
    "nube", "lluvia", "nieve", "cielo", "arcoíris", "corazón", "regalo", "cumpleaños", "dibujo", "computador",
    // comida
    "pan", "leche", "manzana", "plátano", "helado", "torta", "queso", "chocolate", "galleta", "sopa",
    "arroz", "fruta", "dulce", "dulces", "jugo", "huevo", "frutilla", "uva", "naranja", "tomate",
    // colores
    "rojo", "azul", "verde", "amarillo", "naranjo", "rosado", "morado", "negro", "blanco", "café", "gris",
    // acciones
    "jugar", "comer", "dormir", "pintar", "correr", "saltar", "cantar", "bailar", "leer", "escribir",
    "ver", "ir", "tengo", "tiene", "soy", "estoy", "está", "vamos", "mira",
    // animales y cuerpo (los de la enciclopedia ya están grabados con mayúscula; aquí en minúscula)
    "perro", "gato", "caballo", "vaca", "oveja", "gallina", "conejo", "león", "elefante", "jirafa", "mono",
    "oso", "tigre", "cebra", "pingüino", "ballena", "delfín", "tortuga", "cóndor", "pudú", "pájaro", "pez",
    "mano", "manos", "pie", "pies", "ojo", "ojos", "boca", "nariz", "pelo", "cabeza",
];

/// Clave de comparación: minúsculas y sin tildes (la ñ se mantiene).
fn clave(p: &str) -> String {
    p.chars().filter(|c| c.is_alphanumeric()).map(base).collect()
}

/// Palabras del nombre y apodo (para grabarlas sueltas).
fn del_nombre(nombre: &str, apodo: &str) -> Vec<String> {
    nombre.split_whitespace().chain(apodo.split_whitespace()).map(|p| p.to_lowercase()).collect()
}

pub fn frases(f: &mut Frases, nombre: &str, apodo: &str) {
    for p in COMUNES {
        f.es(*p);
    }
    for p in del_nombre(nombre, apodo) {
        f.es(p);
    }
}

/// Si todas las palabras de `texto` están grabadas, devuelve cómo decirlas (con tildes).
pub fn pregrabado(texto: &str, nombre: &str, apodo: &str) -> Option<Vec<String>> {
    let propias = del_nombre(nombre, apodo);
    let mut salida = Vec::new();
    for w in texto.split_whitespace() {
        let k = clave(w);
        if k.is_empty() {
            continue;
        }
        let hallada = COMUNES.iter().map(|s| s.to_string()).chain(propias.iter().cloned()).find(|c| clave(c) == k)?;
        salida.push(hallada);
    }
    (!salida.is_empty()).then_some(salida)
}

/// Lee un texto libre: con la voz grabada si se puede, si no con espeak.
pub fn leer(cx: &mut Cx, texto: &str) {
    match pregrabado(texto, &cx.nombre.clone(), &cx.apodo.clone()) {
        Some(ps) => cx.decir_es_v(&ps),
        None => cx.decir(texto),
    }
}
