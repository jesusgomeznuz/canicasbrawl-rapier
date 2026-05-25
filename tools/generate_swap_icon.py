#!/usr/bin/env python3
"""
Genera assets/effects/swap.glb — dos flechas en arco bicolor estilo tabler:refresh.

Uso:
    /Users/jesus/miniconda3/bin/python tools/generate_swap_icon.py

Salida:
    assets/effects/swap.glb

El modelo es low-poly cartoon: arco grueso + cono tangencial al final.
Dos arcos a 180° con colores distintos. Pensado para girar en eje Z.
"""

import numpy as np
import trimesh
from shapely.geometry import Polygon
from pathlib import Path


def build_one_arrow(r_arc=0.18, r_tube=0.022, start_deg=20.0, end_deg=170.0,
                    arc_segments=24, tube_segments=8,
                    tip_length=0.07, tip_flare=2.4):
    start = np.radians(start_deg)
    end = np.radians(end_deg)
    angles = np.linspace(start, end, arc_segments + 1)
    path = np.column_stack([
        r_arc * np.cos(angles),
        r_arc * np.sin(angles),
        np.zeros_like(angles),
    ])

    cross_angles = np.linspace(0, 2 * np.pi, tube_segments, endpoint=False)
    cross_pts = np.column_stack([
        r_tube * np.cos(cross_angles),
        r_tube * np.sin(cross_angles),
    ])
    cross_polygon = Polygon(cross_pts)

    arc_mesh = trimesh.creation.sweep_polygon(cross_polygon, path, cap_ends=True)

    end_pos = path[-1]
    tangent = np.array([-np.sin(end), np.cos(end), 0.0])
    tangent /= np.linalg.norm(tangent)

    cone = trimesh.creation.cone(
        radius=r_tube * tip_flare,
        height=tip_length,
        sections=tube_segments,
    )
    align = trimesh.geometry.align_vectors([0, 0, 1], tangent)
    cone.apply_transform(align)
    cone.apply_translation(end_pos)

    return trimesh.util.concatenate([arc_mesh, cone])


def main():
    # Colores oficiales JoyCon Nintendo Switch
    joycon_red = np.array([255, 60, 40, 255], dtype=np.uint8)
    joycon_blue = np.array([10, 185, 230, 255], dtype=np.uint8)

    arrow_a = build_one_arrow()
    arrow_a.visual.face_colors = joycon_red

    arrow_b = build_one_arrow()
    flip = trimesh.transformations.rotation_matrix(np.pi, [0, 0, 1])
    arrow_b.apply_transform(flip)
    arrow_b.visual.face_colors = joycon_blue

    scene = trimesh.Scene([arrow_a, arrow_b])

    out = Path(__file__).resolve().parents[1] / "assets" / "effects" / "swap.glb"
    out.parent.mkdir(parents=True, exist_ok=True)
    scene.export(str(out))
    print(f"escrito {out} ({out.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
