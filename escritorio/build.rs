// Deja la fecha y hora de compilación en NIDITO_COMPILADO, para distinguir versiones
// al actualizar desde un pendrive (menú de adultos → Actualizar).
fn main() {
    let fecha = std::process::Command::new("date")
        .arg("+%Y-%m-%d %H:%M")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "?".into());
    println!("cargo:rustc-env=NIDITO_COMPILADO={fecha}");
    println!("cargo:rerun-if-changed=src");
}
