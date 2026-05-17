# Ecology — Design Spec

**Date**: 2026-05-17
**Status**: Approved
**Part of**: Ant Terrarium Simulator (Sub-project 5 of 6)
**Depends on**: Sub-project 1-4

## Overview

Implement dynamic humidity, water flow, fungus growth, organic decomposition, and the closed food cycle. The terrarium becomes a self-sustaining ecosystem where organic matter decomposes into fungus, which ants can eat. Water flows, evaporates, and affects tunnel stability.

## Water & Humidity

### Water flow (simplified)
- Water cells spread humidity to adjacent cells (4 cardinal directions)
- A Water cell gives +5 humidity/tick to each adjacent non-Water, non-Stone cell
- Water cells adjacent to Air slowly evaporate: Water → Air after 600 ticks with <50% humidity in adjacent cells
- Water added by user (right-click on Dirt → WetDirt, as existing)

### Humidity spread
- Every tick, each cell with humidity > neighbors' humidity transfers 1 point to 4 cardinal neighbors
- WetDirt cells generate +2 humidity/tick to adjacent cells
- Air cells in deep tunnels are drier: −1 humidity/tick (ventilation)
- Surface Air cells exposed to "outside": humidity decays to ambient 30%

### Humidity effects
| Range | Effect |
|-------|--------|
| >80% | Fungus growth enabled. LooseDirt collapses 2x faster. |
| 40-80% | Normal. Larvae develop at 1x speed. |
| 20-40% | Dry. Larvae develop at 0.5x speed. Food cells dry out slowly. |
| <20% | Very dry. Larvae die. Food cells become OrganicWaste. Ant stress +0.001/tick. |

## Fungus

### Growth conditions
- Fungus spawns on cells with: humidity > 60% AND organic matter > 0 AND adjacent to existing Fungus OR OrganicWaste
- Fungus grows: OrganicWaste → Fungus (after 1200 ticks of humidity > 60%)
- Fungus spreads: Fungus cell spreads to adjacent Dirt/WetDirt cell with >60% humidity (5% chance per tick)

### Fungus as food
- Ants can `CollectFood` from Fungus cells (same behavior as Food)
- Fungus regrows slowly: after being eaten (→ Air), if humidity remains >60% and organic matter >0, Fungus respawns in 600 ticks
- Fungus provides 0.2 hunger reduction (vs 0.3 for Food)

### Fungus danger
- If Fungus grows too dense (>5 connected fungus cells), it becomes toxic
- Toxic fungus emits DANGER pheromone at rate 3/tick
- Ants avoid dense fungus patches

## Decomposition

### Organic matter
- Each cell already has `organic_matter: u8` (0-255)
- Sources of organic matter:
  - `OrganicWaste` cells decompose into organic_matter in adjacent cells (+5/tick to 4 neighbors)
  - Dead ants become OrganicWaste (already implemented)
  - Food cells left uneaten for 4800 ticks become OrganicWaste
- Organic matter decays very slowly: −1 every 100 ticks

### Cycle
```
Food/Fungus → (eaten by ants) → Ant waste/Death → OrganicWaste
OrganicWaste → (decomposes) → organic_matter + humidity
organic_matter + humidity → Fungus
Fungus → (eaten by ants) → Food source
```

## Out of Scope
- Temperature dynamics (stays at ambient 22°C)
- Advanced ventilation modeling
- Disease or infection spread
