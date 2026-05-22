use ant_simulation::{
    tick::{Simulation, Speed},
    grid::Grid,
    snapshot::Snapshot,
};

fn main() {
    println!("=== Ant Colony Analysis ===\n");

    let mut sim = Simulation::from_grid(Grid::generate_initial_world(128, 96));
    sim.spawn_initial_ants(5);
    sim.set_speed(Speed::Normal);

    // Add food patches on the surface for ants to find
    let sy = sim.grid.surface_y();
    if sy > 0 {
        // Food patch left
        for dx in 0i32..8 { sim.grid.set_material(ant_simulation::grid::GridPos::new((20 + dx) as u16, sy - 1), ant_simulation::grid::Material::Food); }
        // Food patch right
        for dx in 0i32..8 { sim.grid.set_material(ant_simulation::grid::GridPos::new((100 + dx) as u16, sy - 1), ant_simulation::grid::Material::Food); }
        // Some food near queen
        let qx = sim.grid.queen_position().x;
        for dx in -3i32..=3 { sim.grid.set_material(ant_simulation::grid::GridPos::new((qx as i32 + dx) as u16, sy - 1), ant_simulation::grid::Material::Food); }
        // Also put food on the surface itself (dirt cells become food)
        for dx in -2i32..=2 { sim.grid.set_material(ant_simulation::grid::GridPos::new((qx as i32 + dx) as u16, sy), ant_simulation::grid::Material::Food); }
    }

    let total_ticks = 5000;
    let sample_every = 100;

    // Metrics storage
    let mut history: Vec<Metrics> = Vec::new();
    let mut all_positions: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut pheromone_heatmap: Vec<f32> = vec![0.0; (sim.grid.width * sim.grid.height) as usize];
    let mut action_counts: [usize; 11] = [0; 11];

    println!("Running {} ticks", total_ticks);
    let start = std::time::Instant::now();

    // Disable ecology for speed
    let fast_mode = true;

    for tick in 0..total_ticks {
        if fast_mode {
            sim.tick += 1;
            sim.grid.evaporate_pheromones();
            sim.queen_events = sim.queen.tick(&mut sim.grid);
            let ant_events = sim.tick_ants();
            for event in &ant_events {
                if let ant_simulation::ant::AntEvent::DeliveredFood { pos } = event {
                    let dx = pos.x as i32 - sim.queen.pos.x as i32;
                    let dy = pos.y as i32 - sim.queen.pos.y as i32;
                    if dx.abs() <= 2 && dy.abs() <= 2 { sim.queen.deliver_food(); }
                }
            }
            sim.ant_events = ant_events;
        } else {
            sim.tick();
        }

        if tick % sample_every == 0 {
            let snap = Snapshot::from_simulation(&sim);

            // Collect ant positions
            let mut positions: Vec<(f32, f32)> = Vec::new();
            for ant in &snap.ants {
                positions.push((ant.pos.x as f32 / sim.grid.width as f32, ant.pos.y as f32 / sim.grid.height as f32));
                action_counts[action_index(ant.action)] += 1;
            }
            all_positions.push(positions);

            // Accumulate pheromone data
            for (i, cell) in snap.cells.iter().enumerate() {
                pheromone_heatmap[i] += cell.phero_strength() as f32;
            }

            // Compute per-sample metrics
            let avg_hunger: f32 = sim.ants.brains.iter().map(|b| b.hunger).sum::<f32>() / sim.ants.brains.len().max(1) as f32;
            let avg_stress: f32 = sim.ants.brains.iter().map(|b| b.stress).sum::<f32>() / sim.ants.brains.len().max(1) as f32;
            let exploring: usize = sim.ants.bodies.iter().filter(|b| matches!(b.current_action, ant_simulation::ant::Action::Move(_))).count();
            let digging: usize = sim.ants.bodies.iter().filter(|b| matches!(b.current_action, ant_simulation::ant::Action::Dig(_))).count();

            history.push(Metrics {
                tick,
                ant_count: sim.ants.bodies.len(),
                queen_hunger: sim.queen.hunger,
                queen_stress: sim.queen.stress,
                queen_food: sim.queen.food_reserve,
                eggs: sim.life_stages.iter().filter(|s| matches!(s.stage, ant_simulation::queen::Stage::Egg)).count(),
                larvae: sim.life_stages.iter().filter(|s| matches!(s.stage, ant_simulation::queen::Stage::Larva)).count(),
                pupae: sim.life_stages.iter().filter(|s| matches!(s.stage, ant_simulation::queen::Stage::Pupa)).count(),
                avg_hunger,
                avg_stress,
                exploring,
                digging,
                food_pheromone: snap.cells.iter().filter(|c| c.phero_type() == 0 && c.phero_strength() > 50).count(),
                danger_pheromone: snap.cells.iter().filter(|c| c.phero_type() == 2 && c.phero_strength() > 50).count(),
            });
        }
    }

    let elapsed = start.elapsed();
    println!("Simulated {} ticks in {:.1}ms ({:.0} ticks/sec)\n", total_ticks, elapsed.as_secs_f32() * 1000.0, total_ticks as f32 / elapsed.as_secs_f32());

    // ── Print ASCII Charts ──

    let h = &history;

    println!("══════════════════════════════════════════════");
    println!("  Population & Queen Status");
    println!("══════════════════════════════════════════════");
    sparkline(h, |m| m.ant_count as f32, "Ants  ");
    sparkline(h, |m| m.queen_food as f32 * 5.0, "Food  ");
    sparkline(h, |m| m.queen_hunger * 100.0, "Q.Hung");
    sparkline(h, |m| m.queen_stress * 100.0, "Q.Str ");

    println!("\n══════════════════════════════════════════════");
    println!("  Ant Needs (colony average)");
    println!("══════════════════════════════════════════════");
    sparkline(h, |m| m.avg_hunger * 100.0, "Hunger");
    sparkline(h, |m| m.avg_stress * 100.0, "Stress");

    println!("\n══════════════════════════════════════════════");
    println!("  Behavior Distribution");
    println!("══════════════════════════════════════════════");
    sparkline(h, |m| m.exploring as f32, "Explore");
    sparkline(h, |m| m.digging as f32, "Digging");

    println!("\n══════════════════════════════════════════════");
    println!("  Pheromone Activity");
    println!("══════════════════════════════════════════════");
    sparkline(h, |m| m.food_pheromone as f32, "FoodPh");
    sparkline(h, |m| m.danger_pheromone as f32, "Danger");

    println!("\n══════════════════════════════════════════════");
    println!("  Life Stages");
    println!("══════════════════════════════════════════════");
    sparkline(h, |m| m.eggs as f32, "Eggs  ");
    sparkline(h, |m| m.larvae as f32, "Larvae");
    sparkline(h, |m| m.pupae as f32, "Pupae ");

    println!("\n══════════════════════════════════════════════");
    println!("  Action Distribution (total)");
    println!("══════════════════════════════════════════════");
    let actions = ["Idle", "Move", "Dig", "CarryDirt", "Collect", "CarryFood", "Eat", "Rest", "Groom", "Flee", "Share"];
    let max_a = *action_counts.iter().max().unwrap_or(&1) as f32;
    for (i, name) in actions.iter().enumerate() {
        if action_counts[i] > 0 {
            let bar_w = (action_counts[i] as f32 / max_a * 40.0) as usize;
            println!("  {:>10}: {:>5} {}", name, action_counts[i], "█".repeat(bar_w));
        }
    }

    // ── Pheromone coverage ──
    println!("\n══════════════════════════════════════════════");
    println!("  Pheromone Heatmap (normalized)");
    println!("══════════════════════════════════════════════");
    let max_p = pheromone_heatmap.iter().cloned().fold(0.0f32, f32::max).max(1.0);
    let w = sim.grid.width as usize;
    for y in 0..sim.grid.height as usize {
        let mut line = String::with_capacity(w);
        for x in 0..w {
            let v = pheromone_heatmap[y * w + x] / max_p;
            line.push(match v {
                v if v > 0.8 => '█',
                v if v > 0.6 => '▓',
                v if v > 0.4 => '▒',
                v if v > 0.2 => '░',
                _ => ' ',
            });
        }
        // Only print every 4th row
        if y % 4 == 0 {
            println!("  {}", line);
        }
    }

    // ── Generate HTML report ──
    generate_html(&history, &action_counts, &actions, total_ticks, &pheromone_heatmap, sim.grid.width as usize, sim.grid.height as usize);
}

fn sparkline(data: &[Metrics], f: fn(&Metrics) -> f32, label: &str) {
    let values: Vec<f32> = data.iter().map(f).collect();
    let max = values.iter().cloned().fold(0.0f32, f32::max).max(1.0);
    let w = 60;
    let step = (values.len() as f32 / w as f32).max(1.0);
    let mut line = String::with_capacity(w);
    for i in 0..w {
        let idx = (i as f32 * step) as usize;
        if idx < values.len() {
            let v = values[idx] / max;
            line.push(match v {
                v if v > 0.8 => '█',
                v if v > 0.6 => '▓',
                v if v > 0.4 => '▒',
                v if v > 0.2 => '░',
                _ => '▁',
            });
        }
    }
    let last = values.last().copied().unwrap_or(0.0);
    println!("  {:>7} [{:.1}] {}", label, last, line);
}

fn action_index(action: ant_simulation::ant::Action) -> usize {
    use ant_simulation::ant::Action;
    match action {
        Action::Idle => 0, Action::Move(_) => 1, Action::Dig(_) => 2,
        Action::CarryDirt { .. } => 3, Action::CollectFood => 4,
        Action::CarryFood { .. } => 5, Action::Eat => 6, Action::Rest => 7,
        Action::Groom => 8, Action::Flee { .. } => 9,
        Action::Trophallaxis { .. } => 10,
    }
}

struct Metrics {
    tick: u64,
    ant_count: usize,
    queen_hunger: f32,
    queen_stress: f32,
    queen_food: u16,
    eggs: usize,
    larvae: usize,
    pupae: usize,
    avg_hunger: f32,
    avg_stress: f32,
    exploring: usize,
    digging: usize,
    food_pheromone: usize,
    danger_pheromone: usize,
}

fn generate_html(history: &[Metrics], action_counts: &[usize; 11], action_names: &[&str; 11], total_ticks: u64, _heatmap: &[f32], _w: usize, _h: usize) {
    let path = std::path::PathBuf::from("colony_analysis.html");

    // Build JSON data
    let ticks_json: Vec<u64> = history.iter().map(|m| m.tick).collect();
    let ants_json: Vec<usize> = history.iter().map(|m| m.ant_count).collect();
    let hunger_json: Vec<f32> = history.iter().map(|m| m.avg_hunger).collect();
    let stress_json: Vec<f32> = history.iter().map(|m| m.avg_stress).collect();
    let explore_json: Vec<usize> = history.iter().map(|m| m.exploring).collect();
    let dig_json: Vec<usize> = history.iter().map(|m| m.digging).collect();
    let foodph_json: Vec<usize> = history.iter().map(|m| m.food_pheromone).collect();
    let dangerph_json: Vec<usize> = history.iter().map(|m| m.danger_pheromone).collect();
    let eggs_json: Vec<usize> = history.iter().map(|m| m.eggs).collect();
    let larvae_json: Vec<usize> = history.iter().map(|m| m.larvae).collect();
    let action_json: Vec<usize> = action_counts[..11].to_vec();

    let html = format!(r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Ant Colony Analysis</title>
<style>body{{background:#0a0a14;color:#ccc;font-family:monospace;margin:20px}}
h1{{color:#e8c040}}h2{{color:#88aacc;margin-top:30px}}
.chart{{background:#111122;border-radius:8px;padding:16px;margin:10px 0}}
canvas{{width:100%%;max-width:900px}}
.grid{{display:grid;grid-template-columns:repeat(2,1fr);gap:10px}}
@media(max-width:700px){{.grid{{grid-template-columns:1fr}}}}
</style></head><body>
<h1>🐜 Ant Colony Analysis — {ticks} ticks</h1>

<div class="grid">
<div class="chart"><h3>Population</h3><canvas id="ants"></canvas></div>
<div class="chart"><h3>Queen Hunger & Stress</h3><canvas id="queen"></canvas></div>
<div class="chart"><h3>Ant Needs (avg)</h3><canvas id="needs"></canvas></div>
<div class="chart"><h3>Behaviors</h3><canvas id="behaviors"></canvas></div>
<div class="chart"><h3>Pheromone Activity</h3><canvas id="pheromones"></canvas></div>
<div class="chart"><h3>Life Stages</h3><canvas id="life"></canvas></div>
</div>

<div class="chart"><h3>Action Distribution</h3><canvas id="actions" style="height:200px"></canvas></div>

<script>
const ticks = {ticks_json:?};
const ants = {ants_json:?};
const hunger = {hunger_json:?};
const stress = {stress_json:?};
const explore = {explore_json:?};
const dig = {dig_json:?};
const foodph = {foodph_json:?};
const dangerph = {dangerph_json:?};
const eggs = {eggs_json:?};
const larvae = {larvae_json:?};
const actionDist = {action_json:?};
const actionLabels = {action_names:?};

function lineChart(id, datasets) {{
    const canvas = document.getElementById(id);
    const ctx = canvas.getContext('2d');
    const w = canvas.parentElement.clientWidth - 32;
    canvas.width = w; canvas.height = 180;
    const h = 180; const pad = {{t:10,r:10,b:30,l:50}};
    const pw = w - pad.l - pad.r; const ph = h - pad.t - pad.b;

    let allMax = 0;
    for(const ds of datasets) for(const v of ds.data) if(v>allMax) allMax=v;
    if(allMax==0) allMax=1;

    ctx.fillStyle='#111122'; ctx.fillRect(0,0,w,h);
    // Grid
    ctx.strokeStyle='#1a1a33'; ctx.lineWidth=1;
    for(let i=0;i<=4;i++) {{
        const y = pad.t + ph * (1 - i/4);
        ctx.beginPath(); ctx.moveTo(pad.l,y); ctx.lineTo(w-pad.r,y); ctx.stroke();
        ctx.fillStyle='#555'; ctx.fillText(Math.round(allMax*i/4), 2, y+4);
    }}
    // Lines
    for(const ds of datasets) {{
        ctx.strokeStyle=ds.color; ctx.lineWidth=2; ctx.beginPath();
        for(let i=0;i<ds.data.length;i++) {{
            const x = pad.l + (i/ds.data.length)*pw;
            const y = pad.t + ph*(1 - ds.data[i]/allMax);
            if(i==0) ctx.moveTo(x,y); else ctx.lineTo(x,y);
        }}
        ctx.stroke();
    }}
    // Legend
    let lx=pad.l;
    for(const ds of datasets) {{
        ctx.fillStyle=ds.color; ctx.fillRect(lx,h-16,12,10);
        ctx.fillStyle='#ccc'; ctx.fillText(ds.label,lx+16,h-6);
        lx+=ctx.measureText(ds.label).width+40;
    }}
}}

function barChart(id, labels, data, color) {{
    const canvas = document.getElementById(id);
    const ctx = canvas.getContext('2d');
    const w = canvas.parentElement.clientWidth - 32;
    canvas.width = w; canvas.height = 160;
    const h = 160; const pad = {{t:10,r:10,b:40,l:60}};
    const pw = w - pad.l - pad.r; const ph = h - pad.t - pad.b;
    const max = Math.max(...data, 1);
    const bw = pw / data.length * 0.7; const gap = pw / data.length * 0.3;

    ctx.fillStyle='#111122'; ctx.fillRect(0,0,w,h);
    for(let i=0;i<data.length;i++) {{
        const bh = (data[i]/max)*ph;
        ctx.fillStyle=color; ctx.fillRect(pad.l + i*(bw+gap), pad.t+ph-bh, bw, bh);
        ctx.fillStyle='#aaa';
        ctx.save(); ctx.translate(pad.l+i*(bw+gap)+bw/2, h-4); ctx.rotate(-0.5);
        ctx.fillText(labels[i],0,0); ctx.restore();
    }}
}}

lineChart('ants', [{{label:'Worker Ants',data:ants,color:'#e8c040'}}]);
lineChart('queen', [{{label:'Hunger %',data:hunger.map(v=>v*100),color:'#e04040'}},{{label:'Stress %',data:stress.map(v=>v*100),color:'#8844ff'}}]);
lineChart('needs', [{{label:'Avg Hunger %',data:hunger.map(v=>v*100),color:'#ff8844'}},{{label:'Avg Stress %',data:stress.map(v=>v*100),color:'#cc44ff'}}]);
lineChart('behaviors', [{{label:'Exploring',data:explore,color:'#44aaff'}},{{label:'Digging',data:dig,color:'#e8c040'}}]);
lineChart('pheromones', [{{label:'Food Pheromone',data:foodph,color:'#44dd44'}},{{label:'Danger Pheromone',data:dangerph,color:'#ff4444'}}]);
lineChart('life', [{{label:'Eggs',data:eggs,color:'#ffffff'}},{{label:'Larvae',data:larvae,color:'#ffcc88'}}]);
barChart('actions', actionLabels, actionDist, '#88aacc');
</script>
<p style="color:#666;margin-top:20px">Generated by ant_terrarium analyze tool | {samples} samples over {ticks} ticks</p>
</body></html>"#,
        samples = history.len(),
        ticks = total_ticks,
    );

    std::fs::write(&path, html).unwrap();
    println!("\n📊 HTML report saved to: {}", path.display());
    println!("   Open it in your browser to see interactive charts.");
}
