use crate::models::{Algorithm, StepKind};
use crate::settings::GRID_SNAP_STEP;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

const STEP_Y_DELTA: f32 = 324.0;
const SIBLING_STEP_X: f32 = 396.0;

#[derive(Clone)]
struct BranchEdge {
    parent_id: Uuid,
    branch_idx: usize,
    total_branches: usize,
}

pub fn auto_layout_algorithm(algo: &mut Algorithm, snap_to_grid: bool) {
    if algo.steps.is_empty() {
        return;
    }

    // 1. Сбор прямых рёбер с сохранением строгого порядка выходов
    let mut raw_edges: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    let mut incoming_branches: HashMap<Uuid, Vec<BranchEdge>> = HashMap::new();
    let mut in_degrees: HashMap<Uuid, usize> = HashMap::new();

    for step in &algo.steps {
        raw_edges.entry(step.id).or_default();
        in_degrees.entry(step.id).or_insert(0);
        incoming_branches.entry(step.id).or_default();
    }

    for step in &algo.steps {
        let mut targets = Vec::new();
        match &step.kind {
            StepKind::Standard => {
                for opt in &step.options {
                    if let Some(tid) = opt.next_step_id {
                        targets.push(tid);
                    }
                }
            }
            StepKind::Measurement {
                next_if_normal,
                next_if_abnormal,
                ..
            } => {
                if let Some(nid) = next_if_normal {
                    targets.push(*nid);
                }
                if let Some(aid) = next_if_abnormal {
                    targets.push(*aid);
                }
            }
            StepKind::SafetyWarning { next_step_id, .. } => {
                if let Some(nid) = next_step_id {
                    targets.push(*nid);
                }
            }
            StepKind::Subprocess {
                next_if_success,
                next_if_failure,
                ..
            } => {
                if let Some(sid) = next_if_success {
                    targets.push(*sid);
                }
                if let Some(fid) = next_if_failure {
                    targets.push(*fid);
                }
            }
        }

        let total = targets.len();
        for (idx, tid) in targets.into_iter().enumerate() {
            if raw_edges.contains_key(&tid) {
                raw_edges.entry(step.id).or_default().push(tid);
                incoming_branches.entry(tid).or_default().push(BranchEdge {
                    parent_id: step.id,
                    branch_idx: idx,
                    total_branches: total,
                });
                *in_degrees.entry(tid).or_insert(0) += 1;
            }
        }
    }

    // 2. Определение корня схемы
    let root_id = algo
        .first_step_id
        .filter(|id| raw_edges.contains_key(id))
        .or_else(|| {
            algo.steps
                .iter()
                .find(|s| in_degrees.get(&s.id) == Some(&0))
                .map(|s| s.id)
        })
        .unwrap_or_else(|| algo.steps[0].id);

    // 3. Выделение прямого дерева (Forward DAG) без учета обратных циклов
    let mut state: HashMap<Uuid, u8> = HashMap::new();
    let mut fwd_adj: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    let mut topo_order: Vec<Uuid> = Vec::new();

    for step in &algo.steps {
        fwd_adj.insert(step.id, Vec::new());
    }

    fn dfs_dag(
        u: Uuid,
        raw_edges: &HashMap<Uuid, Vec<Uuid>>,
        state: &mut HashMap<Uuid, u8>,
        fwd_adj: &mut HashMap<Uuid, Vec<Uuid>>,
        topo_order: &mut Vec<Uuid>,
    ) {
        state.insert(u, 1);
        if let Some(neighbors) = raw_edges.get(&u) {
            for &v in neighbors {
                let v_state = state.get(&v).copied().unwrap_or(0);
                if v_state == 1 {
                    continue; // обратная петля наверх
                }
                fwd_adj.entry(u).or_default().push(v);
                if v_state == 0 {
                    dfs_dag(v, raw_edges, state, fwd_adj, topo_order);
                }
            }
        }
        state.insert(u, 2);
        topo_order.push(u);
    }

    dfs_dag(
        root_id,
        &raw_edges,
        &mut state,
        &mut fwd_adj,
        &mut topo_order,
    );

    for step in &algo.steps {
        if state.get(&step.id).copied().unwrap_or(0) == 0 {
            dfs_dag(
                step.id,
                &raw_edges,
                &mut state,
                &mut fwd_adj,
                &mut topo_order,
            );
        }
    }

    topo_order.reverse();

    // 4. Расчет вертикальных уровней Y
    let mut levels: HashMap<Uuid, usize> = HashMap::new();
    for step in &algo.steps {
        levels.insert(step.id, 0);
    }

    for &u in &topo_order {
        let u_lvl = levels[&u];
        if let Some(children) = fwd_adj.get(&u) {
            for &v in children {
                let next_lvl = u_lvl + 1;
                let cur_v = levels.entry(v).or_insert(0);
                if next_lvl > *cur_v {
                    *cur_v = next_lvl;
                }
            }
        }
    }

    let mut layers: BTreeMap<usize, Vec<Uuid>> = BTreeMap::new();
    for &u in &topo_order {
        let lvl = levels[&u];
        layers.entry(lvl).or_default().push(u);
    }

    // 5. Расчет X-координат со строгой привязкой порядка веток
    let mut assigned_x: HashMap<Uuid, f32> = HashMap::new();

    // Размещение корней
    if let Some(roots) = layers.get(&0) {
        let count = roots.len();
        let start_x = -(count as f32 - 1.0_f32) * 0.5_f32 * SIBLING_STEP_X;
        for (i, &id) in roots.iter().enumerate() {
            assigned_x.insert(id, start_x + (i as f32) * SIBLING_STEP_X);
        }
    }

    for (&lvl, node_ids) in &layers {
        if lvl == 0 {
            continue;
        }

        let mut desired_xs: Vec<(Uuid, f32)> = Vec::new();

        for &node_id in node_ids {
            let branches = incoming_branches.get(&node_id).cloned().unwrap_or_default();
            let mut votes = Vec::new();

            for b in branches {
                if levels.get(&b.parent_id).copied().unwrap_or(0) < lvl {
                    if let Some(&px) = assigned_x.get(&b.parent_id) {
                        let offset = if b.total_branches <= 1 {
                            0.0_f32
                        } else {
                            // Опция 0 -> влево, Опция 1 -> вправо
                            (b.branch_idx as f32 - (b.total_branches as f32 - 1.0_f32) * 0.5_f32)
                                * SIBLING_STEP_X
                        };
                        votes.push(px + offset);
                    }
                }
            }

            let desired_x = if !votes.is_empty() {
                votes.iter().sum::<f32>() / votes.len() as f32
            } else {
                0.0_f32
            };

            desired_xs.push((node_id, desired_x));
        }

        // Сортировка слоя слева направо
        desired_xs.sort_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        // Устранение наложений с соблюдением зазора
        let mut xs: Vec<f32> = desired_xs.iter().map(|p| p.1).collect();
        for i in 1..xs.len() {
            if xs[i] < xs[i - 1] + SIBLING_STEP_X {
                xs[i] = xs[i - 1] + SIBLING_STEP_X;
            }
        }

        let orig_mid = (desired_xs.first().unwrap().1 + desired_xs.last().unwrap().1) * 0.5_f32;
        let new_mid = (xs.first().unwrap() + xs.last().unwrap()) * 0.5_f32;
        let shift = new_mid - orig_mid;

        for (i, (node_id, _)) in desired_xs.into_iter().enumerate() {
            assigned_x.insert(node_id, xs[i] - shift);
        }
    }

    // 6. Центрирование графа и привязка к сетке
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    for &x in assigned_x.values() {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
    }
    let global_offset_x = (min_x + max_x) * 0.5_f32;

    for step in &mut algo.steps {
        let raw_x = assigned_x.get(&step.id).copied().unwrap_or(0.0_f32) - global_offset_x;
        let raw_y = (levels[&step.id] as f32) * STEP_Y_DELTA + 100.0_f32;

        let mut final_x = raw_x;
        let mut final_y = raw_y;

        if snap_to_grid {
            final_x = (final_x / GRID_SNAP_STEP).round() * GRID_SNAP_STEP;
            final_y = (final_y / GRID_SNAP_STEP).round() * GRID_SNAP_STEP;
        }

        step.pos = [final_x, final_y];
    }
}
