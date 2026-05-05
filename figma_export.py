#!/usr/bin/env python3
"""
figma_export.py — Exporta Level_01 de Figma → assets/levels/level_01.json

Uso:
  python3 figma_export.py          (o: cargo run -- --process-figma)

Requiere FIGMA_TOKEN en el entorno (~/.zshrc):
  export FIGMA_TOKEN="figd_..."

Convención de nombres de capas en Figma:
  platform          plataforma estática, forma detectada automáticamente
  platform|r45      rotación inicial explícita en grados
  platform|w1.5     velocidad angular en rad/s (+ = anti-horario)
  platform|r45|w1.5 ambos

Detección automática:
  Rectángulo cuadrado (w ≈ h, lado > 25 px) → rombo (rot = 45°)
  Cualquier otro rectángulo → barra horizontal/vertical según dimensiones
"""

import json
import math
import os
import re
import sys
import urllib.request
import urllib.error

FILE_KEY   = "q6JrX6IyL2hMdFLITiQZZD"
FRAME_ID   = "4:2"
ARENA_W_PX = 110.0
ARENA_H_PX = 960.0
OUTPUT     = "assets/levels/level_01.json"


def figma_get(endpoint: str) -> dict:
    token = os.environ.get("FIGMA_TOKEN")
    if not token:
        sys.exit("ERROR: FIGMA_TOKEN no está en el entorno. Agrégalo a ~/.zshrc")
    url = f"https://api.figma.com/v1/{endpoint}"
    req = urllib.request.Request(url, headers={"X-Figma-Token": token})
    try:
        with urllib.request.urlopen(req) as r:
            return json.loads(r.read())
    except urllib.error.HTTPError as e:
        sys.exit(f"ERROR Figma API {e.code}: {e.read().decode()}")


def parse_name(name: str) -> tuple[float | None, float]:
    """Devuelve (rot_deg_override, angvel_z) desde el nombre del nodo."""
    rot_deg = None
    angvel_z = 0.0
    for part in name.split("|")[1:]:
        m = re.match(r"r(-?[\d.]+)", part)
        if m:
            rot_deg = float(m.group(1))
        m = re.match(r"w(-?[\d.]+)", part)
        if m:
            angvel_z = float(m.group(1))
    return rot_deg, angvel_z


def convert_node(node: dict, frame_x: float, frame_y: float) -> dict:
    bb   = node["absoluteBoundingBox"]
    rx   = bb["x"] - frame_x
    ry   = bb["y"] - frame_y
    w, h = bb["width"], bb["height"]

    cx_px = rx + w / 2.0
    cy_px = ry + h / 2.0
    game_x = (cx_px - ARENA_W_PX / 2.0) * 0.01
    game_y = (ARENA_H_PX - cy_px) * 0.01

    rot_deg_override, angvel_z = parse_name(node.get("name", "platform"))

    # La REST API devuelve absoluteBoundingBox (post-rotación).
    # Para un cuadrado de lado s rotado 45°, el bounding box mide s*√2.
    # Detectamos rombos por w≈h y corregimos dividiendo por √2.
    is_auto_diamond = rot_deg_override is None and abs(w - h) < 3.0 and w > 25.0
    if rot_deg_override is not None:
        rot_rad = math.radians(rot_deg_override)
        hx = w / 2.0 * 0.01
        hy = h / 2.0 * 0.01
    elif is_auto_diamond:
        rot_rad = math.pi / 4.0
        side = w / math.sqrt(2)  # lado real del cuadrado antes de rotar
        hx = side / 2.0 * 0.01
        hy = side / 2.0 * 0.01
    else:
        rot_rad = 0.0
        hx = w / 2.0 * 0.01
        hy = h / 2.0 * 0.01

    return {
        "x":       round(game_x, 4),
        "y":       round(game_y, 4),
        "hx":      round(hx, 4),
        "hy":      round(hy, 4),
        "rot":     round(rot_rad, 6),
        "angvel_z": angvel_z,
    }


def node_center(node: dict, frame_x: float, frame_y: float) -> tuple[float, float, float, float]:
    """Devuelve (game_x, game_y, game_w, game_h) del centro del nodo."""
    bb  = node["absoluteBoundingBox"]
    rx  = bb["x"] - frame_x
    ry  = bb["y"] - frame_y
    w, h = bb["width"], bb["height"]
    cx_px = rx + w / 2.0
    cy_px = ry + h / 2.0
    return (
        round((cx_px - ARENA_W_PX / 2.0) * 0.01, 4),
        round((ARENA_H_PX - cy_px) * 0.01, 4),
        round(w * 0.01, 4),
        round(h * 0.01, 4),
    )


def main():
    print(f"Leyendo frame {FRAME_ID} del archivo {FILE_KEY}...")
    frame_id_encoded = FRAME_ID.replace(":", "%3A")
    data = figma_get(f"files/{FILE_KEY}/nodes?ids={frame_id_encoded}")

    frame_doc = data["nodes"][FRAME_ID]["document"]
    frame_bb  = frame_doc["absoluteBoundingBox"]
    frame_x, frame_y = frame_bb["x"], frame_bb["y"]

    platforms = []
    floor_y   = None
    spawn     = None
    skipped   = 0

    for node in frame_doc.get("children", []):
        name     = node.get("name", "")
        nodetype = node.get("type", "")

        if name == "floor" and nodetype == "RECTANGLE":
            # La superficie del suelo = borde SUPERIOR del rectángulo
            top_px = node["absoluteBoundingBox"]["y"] - frame_y
            floor_y = round((ARENA_H_PX - top_px) * 0.01, 4)
            print(f"  floor detectado → floor_y={floor_y} m")

        elif name == "spawn_area" and nodetype == "RECTANGLE":
            cx, cy, _, _ = node_center(node, frame_x, frame_y)
            spawn = {"cx": cx, "cy": cy}
            print(f"  spawn_area detectado → centro=({cx}, {cy}) m")

        elif nodetype == "RECTANGLE" and name.startswith("platform"):
            platforms.append(convert_node(node, frame_x, frame_y))

        else:
            skipped += 1

    if not platforms:
        sys.exit("ERROR: no se encontraron nodos con nombre 'platform*' en el frame.")

    output: dict = {"platforms": platforms}
    if floor_y is not None:
        output["floor_y"] = floor_y
    if spawn is not None:
        output["spawn"] = spawn

    os.makedirs(os.path.dirname(OUTPUT), exist_ok=True)
    with open(OUTPUT, "w") as f:
        json.dump(output, f, indent=2)

    print(f"✓ {len(platforms)} plataformas exportadas → {OUTPUT}")
    if skipped:
        print(f"  ({skipped} nodos ignorados)")


if __name__ == "__main__":
    main()
