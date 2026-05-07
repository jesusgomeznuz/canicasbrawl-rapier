---
description: Actualiza los flowcharts en docs/flow.md para reflejar el estado actual del código.
---

# update-flow

Refresca `docs/flow.md` con el estado actual del repo.

## Procedimiento

1. **Lee el flow actual**: `docs/flow.md` — revisa los 5 diagramas existentes (main, pipeline editor, pipeline video actual, pipeline video futuro, módulos del crate).

2. **Verifica contra el código**:
   - `src/main.rs` — flags soportados, plugins registrados, sistemas en cada schedule. ¿Cambió algún `add_plugins`, `add_systems`, o el match de `parse_mode()`?
   - `src/world.rs`, `src/level.rs`, `src/marbles.rs`, `src/camera.rs`, `src/race.rs`, `src/process_modules.rs` — responsabilidades por módulo, qué exporta cada uno.
   - `assets/modules/*.json` — qué módulos existen como fuente de verdad.
   - Pipeline de producción: revisa `outputs/`, scripts Python de mezcla de audio si los hay, `voice_tracker.json`.

3. **Detecta drift**:
   - Sistemas/plugins nuevos en `main.rs` que no aparecen en el flowchart de main.
   - Archivos `.rs` nuevos en `src/` que no están en el diagrama de módulos.
   - Cambios en el pipeline (nuevos flags, nuevos artefactos en `outputs/`, cambios en la skill `/export-module`).
   - Diagramas que mencionan archivos/funciones que ya no existen.

4. **Actualiza solo lo que cambió**. No reescribas diagramas que siguen vigentes — eso introduce ruido en el diff. Si todo sigue igual, dilo y termina.

5. **Valida la sintaxis Mermaid**: si el MCP `mermaid` está configurado en `.mcp.json`, llama a su tool `generate` con cada bloque modificado para confirmar que renderiza. Si no está, solo verifica que las llaves `flowchart`, `graph`, `subgraph` y los `-->` estén bien balanceados.

6. **Reporta**: una lista corta de qué diagramas cambiaron y por qué (ej. "main: añadido sistema `X` en Update; módulos: nuevo archivo `audio.rs`"). Sin diff completo — el usuario lo ve en git.

## Notas

- El pipeline futuro (sección 4 del flow.md) es prospectivo — no lo "corrijas" contra el código actual; solo actualízalo si el usuario te pide reflejar un cambio de plan.
- Mantén el estilo conciso: nodos con texto corto, flechas con label solo si el "qué pasa entre A y B" no es obvio.
- Si encuentras un flujo nuevo importante (ej. nueva pipeline de assets), considera proponer un diagrama #6 al usuario antes de añadirlo — no lo agregues sin confirmar.
