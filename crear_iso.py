#!/usr/bin/env python3
"""Crear la ISO de Nidito con una ventana (tkinter), sin escribir comandos.

Hace lo mismo que los scripts, en orden:
  1. ./personalizar.sh   (perfil con el nombre del niño o niña; se omite en la versión pública)
  2. ./preparar.sh       (compila, genera las voces y arma el perfil de la ISO)
  3. pkexec ./construir_iso.sh   (crea la ISO; pide la clave de administrador del computador)

Necesita Arch Linux (o derivado) con: archiso, rust, clang, ffmpeg y uv.
Uso: python3 crear_iso.py
"""

import os
import queue
import re
import shutil
import subprocess
import threading
import tkinter as tk
import unicodedata
from tkinter import messagebox, ttk

RAIZ = os.path.dirname(os.path.abspath(__file__))
PAQUETES = {
    "mkarchiso": "archiso",
    "cargo": "rust",
    "clang": "clang",
    "ffmpeg": "ffmpeg",
    "uv": "uv",
    "pkexec": "polkit",
}


def a_usuario(nombre: str) -> str:
    """'José Ñúñez' -> 'josenunez' (igual que personalizar.sh)."""
    s = unicodedata.normalize("NFKD", nombre).encode("ascii", "ignore").decode().lower()
    s = re.sub(r"[^a-z0-9]", "", s)
    s = re.sub(r"^[0-9]+", "", s)
    return s[:16]


class App(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("Crear la ISO de Nidito")
        self.minsize(720, 640)
        self.cola: queue.Queue = queue.Queue()
        self.trabajando = False

        marco = ttk.Frame(self, padding=14)
        marco.pack(fill="both", expand=True)

        ttk.Label(marco, text="Nidito: crear la ISO", font=("TkDefaultFont", 16, "bold")).grid(row=0, column=0, columnspan=4, sticky="w")
        ttk.Label(marco, text="Llena los datos y presiona «Crear ISO». Tarda entre 10 y 30 minutos la primera vez.").grid(
            row=1, column=0, columnspan=4, sticky="w", pady=(0, 10))

        self.publica = tk.BooleanVar(value=False)
        ttk.Checkbutton(marco, text="Versión pública (sin nombre: saluda con «¡Hola!»)", variable=self.publica,
                        command=self.cambiar_publica).grid(row=2, column=0, columnspan=4, sticky="w", pady=(0, 6))

        self.nombre = tk.StringVar()
        self.apodo = tk.StringVar()
        self.usuario = tk.StringVar()
        self.juegos = tk.StringVar(value="20")
        self.pausa = tk.StringVar(value="20")
        self.cumple = tk.StringVar()
        self.nombre.trace_add("write", lambda *_: self.usuario.set(a_usuario(self.nombre.get()) or "nino"))

        campos = [
            ("Nombre del niño o niña:", self.nombre, "Así lo saluda: «¡Hola, Sofía!»"),
            ("Apodo (opcional):", self.apodo, "Si hay apodo, saluda con el apodo"),
            ("Usuario del sistema:", self.usuario, "Solo a-z y 0-9 (se sugiere solo)"),
            ("Minutos de juegos por día:", self.juegos, "0 = sin tope"),
            ("Pausa para los ojos cada (min):", self.pausa, "0 = nunca"),
            ("Cumpleaños (DD-MM, opcional):", self.cumple, "Ese día hay globos y «Cumpleaños feliz»"),
        ]
        self.entradas = []
        for i, (etq, var, ayuda) in enumerate(campos):
            ttk.Label(marco, text=etq).grid(row=3 + i, column=0, sticky="w", pady=2)
            e = ttk.Entry(marco, textvariable=var, width=28)
            e.grid(row=3 + i, column=1, sticky="w", pady=2)
            ttk.Label(marco, text=ayuda, foreground="#666").grid(row=3 + i, column=2, columnspan=2, sticky="w", padx=8)
            self.entradas.append(e)

        voces = ttk.LabelFrame(marco, text="Voces", padding=8)
        voces.grid(row=9, column=0, columnspan=4, sticky="we", pady=10)
        self.motor = tk.StringVar(value="piper")
        ttk.Radiobutton(voces, text="Piper: voces libres, sin internet. Se pueden compartir.", value="piper",
                        variable=self.motor).pack(anchor="w")
        ttk.Radiobutton(voces, text="edge-tts: más naturales, necesita internet. Solo para uso personal (no publicar).",
                        value="edge", variable=self.motor).pack(anchor="w")

        botones = ttk.Frame(marco)
        botones.grid(row=10, column=0, columnspan=4, sticky="we")
        ttk.Button(botones, text="Revisar requisitos", command=self.revisar).pack(side="left")
        self.b_crear = ttk.Button(botones, text="Crear ISO", command=self.crear)
        self.b_crear.pack(side="left", padx=8)
        ttk.Button(botones, text="Abrir carpeta de la ISO", command=self.abrir_salida).pack(side="left")

        self.registro = tk.Text(marco, height=14, wrap="word", font=("TkFixedFont", 9), state="disabled")
        self.registro.grid(row=11, column=0, columnspan=4, sticky="nsew", pady=(10, 0))
        barra = ttk.Scrollbar(marco, command=self.registro.yview)
        barra.grid(row=11, column=4, sticky="ns", pady=(10, 0))
        self.registro["yscrollcommand"] = barra.set
        marco.rowconfigure(11, weight=1)
        marco.columnconfigure(3, weight=1)

        self.after(100, self.leer_cola)
        self.revisar(silencioso=True)

    # --- utilidades ---
    def escribir(self, texto: str):
        self.registro["state"] = "normal"
        self.registro.insert("end", texto)
        self.registro.see("end")
        self.registro["state"] = "disabled"

    def leer_cola(self):
        try:
            while True:
                tipo, dato = self.cola.get_nowait()
                if tipo == "texto":
                    self.escribir(dato)
                elif tipo == "fin":
                    self.terminar(*dato)
        except queue.Empty:
            pass
        self.after(100, self.leer_cola)

    def cambiar_publica(self):
        estado = "disabled" if self.publica.get() else "normal"
        for e in self.entradas:
            e["state"] = estado

    def faltan(self):
        return [(p, PAQUETES[p]) for p in PAQUETES if shutil.which(p) is None]

    def revisar(self, silencioso=False):
        faltan = self.faltan()
        if not faltan:
            if not silencioso:
                self.escribir("Requisitos: todo instalado.\n")
            return True
        pkgs = " ".join(sorted({pk for _, pk in faltan}))
        self.escribir(f"Faltan programas: {', '.join(p for p, _ in faltan)}\n"
                      f"Instálalos en una terminal con:\n  sudo pacman -S --needed {pkgs}\n\n")
        return False

    def abrir_salida(self):
        subprocess.Popen(["xdg-open", os.path.join(RAIZ, "salida")])

    # --- validar y crear ---
    def validar(self):
        if self.publica.get():
            return None
        nombre = self.nombre.get().strip()
        if not nombre:
            return "Falta el nombre."
        if not re.fullmatch(r"[a-z][a-z0-9]{0,15}", self.usuario.get()):
            return "Usuario inválido: solo a-z y 0-9, empieza con letra, máximo 16."
        if self.usuario.get() in ("root", "bin", "daemon", "nobody", "mail", "ftp", "http"):
            return "Ese usuario está reservado."
        for v, n in ((self.juegos, "minutos de juegos"), (self.pausa, "pausa para los ojos")):
            if not v.get().strip().isdigit():
                return f"Los {n} deben ser un número."
        c = self.cumple.get().strip()
        if c and not re.fullmatch(r"\d{1,2}[-/]\d{1,2}", c):
            return "El cumpleaños va como DD-MM, por ejemplo 15-03."
        return None

    def crear(self):
        if self.trabajando:
            return
        if not self.revisar():
            messagebox.showwarning("Faltan programas", "Instala los programas que aparecen abajo y vuelve a intentar.")
            return
        error = self.validar()
        if error:
            messagebox.showerror("Revisa los datos", error)
            return
        if self.publica.get() and self.motor.get() == "edge":
            if not messagebox.askyesno("Voces edge-tts", "Las voces edge-tts son solo para uso personal.\n"
                                       "¿Seguro que quieres la versión pública con ellas? (no la compartas)"):
                return
        perfil = os.path.join(RAIZ, "perfiles", f"{self.usuario.get()}.conf")
        if not self.publica.get() and os.path.exists(perfil):
            if not messagebox.askyesno("Perfil existente", f"Ya existe el perfil «{self.usuario.get()}».\n"
                                       "¿Reemplazarlo con estos datos?"):
                return
        pasos = []
        if self.publica.get():
            pasos.append(["./preparar.sh", "publico"])
        else:
            pasos.append(["./personalizar.sh", self.nombre.get().strip(), self.apodo.get().strip(), self.usuario.get(),
                          self.juegos.get().strip(), self.cumple.get().strip(), self.pausa.get().strip()])
            pasos.append(["./preparar.sh", self.usuario.get()])
        pasos.append(["pkexec", os.path.join(RAIZ, "construir_iso.sh")])
        self.trabajando = True
        self.b_crear["state"] = "disabled"
        self.escribir("\n=== Creando la ISO ===\n")
        threading.Thread(target=self.ejecutar, args=(pasos,), daemon=True).start()

    def ejecutar(self, pasos):
        env = dict(os.environ, NIDITO_MOTOR=self.motor.get())
        for cmd in pasos:
            self.cola.put(("texto", f"\n$ {' '.join(cmd)}\n"))
            try:
                p = subprocess.Popen(cmd, cwd=RAIZ, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                     stdin=subprocess.DEVNULL, text=True, bufsize=1)
            except OSError as e:
                self.cola.put(("fin", (False, f"No se pudo ejecutar {cmd[0]}: {e}")))
                return
            for linea in p.stdout:
                self.cola.put(("texto", linea))
            if p.wait() != 0:
                if cmd[0] == "pkexec":
                    msg = ("No se pudo crear la ISO como administrador (¿se canceló la clave?).\n"
                           "Todo lo demás quedó listo: en una terminal, dentro de la carpeta de Nidito, ejecuta:\n"
                           "  sudo ./construir_iso.sh")
                else:
                    msg = f"Falló: {' '.join(cmd)}. Revisa el registro."
                self.cola.put(("fin", (False, msg)))
                return
        self.cola.put(("fin", (True, "")))

    def terminar(self, ok, msg):
        self.trabajando = False
        self.b_crear["state"] = "normal"
        if ok:
            salida = os.path.join(RAIZ, "salida")
            isos = sorted((f for f in os.listdir(salida) if f.endswith(".iso")),
                          key=lambda f: os.path.getmtime(os.path.join(salida, f)))
            iso = isos[-1] if isos else "?"
            self.escribir(f"\n=== Lista: salida/{iso} ===\n")
            messagebox.showinfo("¡Lista!", f"La ISO quedó en:\nsalida/{iso}\n\n"
                                "Grábala en un pendrive (por ejemplo con Ventoy, balenaEtcher o dd) "
                                "y arranca el computador desde él.")
        else:
            self.escribir(f"\n{msg}\n")
            messagebox.showerror("No se pudo terminar", msg)


if __name__ == "__main__":
    App().mainloop()
