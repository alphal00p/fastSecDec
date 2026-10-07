//! Topology selection delegates circuit enumeration to Linnet.

use std::collections::BTreeSet;

use feynkit_generator::VertexSelector;
use feynkit_graph::FeynmanDiagram;
use feynkit_model::Model;
use serde::Serialize;

use super::Result;

pub fn interactions(model: &Model) -> Result<Vec<VertexSelector>> {
    let mut allowed = Vec::new();
    let mut signatures = BTreeSet::new();
    for rule in model.vertex_rules() {
        let mut pdgs = rule
            .particles
            .iter()
            .map(|id| model.particle_by_id(*id).map(|p| p.pdg_code))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        pdgs.sort_unstable();
        if pdgs == [-6, 6, 21] || pdgs == [-6, 6, 25] {
            signatures.insert(pdgs);
            allowed.push(VertexSelector::from(rule.name.clone()));
        }
    }
    if signatures.len() != 2 {
        return Err("model must contain both ttg and ttH interactions".into());
    }
    Ok(allowed)
}

#[derive(Debug, Serialize)]
pub struct Topology {
    pub top_edges: Vec<usize>,
    pub gluon_edge: usize,
    pub circuit_edge_counts: Vec<usize>,
    pub hexagon_edges: Vec<usize>,
    pub box_external_pdgs: Vec<Vec<i64>>,
}

pub fn double_box(diagram: &FeynmanDiagram) -> Result<Option<Topology>> {
    if diagram.vertices().count() != 6 || diagram.loop_momentum_basis().loop_edges.len() != 2 {
        return Ok(None);
    }
    let mut top = Vec::new();
    let mut gluon = Vec::new();
    let mut externals = Vec::new();
    for (id, ends, edge) in diagram.edges() {
        let pdg = diagram.model().particle_by_id(edge.particle)?.pdg_code;
        if edge.is_dummy {
            return Ok(None);
        }
        if edge.external.is_some() {
            externals.push((
                ends.source
                    .or(ends.target)
                    .ok_or("external without a vertex")?
                    .0,
                pdg,
            ));
        } else {
            match pdg.abs() {
                6 => top.push(id.0),
                21 => gluon.push(id.0),
                _ => return Ok(None),
            }
        }
    }
    let mut external_pdgs = externals.iter().map(|(_, pdg)| *pdg).collect::<Vec<_>>();
    external_pdgs.sort_unstable();
    if top.len() != 6 || gluon.len() != 1 || external_pdgs != [21, 21, 25, 25] {
        return Ok(None);
    }
    let graph = diagram.underlying();
    let internal = diagram.internal_subgraph();
    if !graph.is_connected(&internal) {
        return Ok(None);
    }
    let circuits = graph.all_cycles_of(&internal, 2)?;
    let mut counts = circuits
        .iter()
        .map(|cycle| graph.iter_edges_of(&cycle.filter).count())
        .collect::<Vec<_>>();
    counts.sort_unstable();
    if counts != [4, 4, 6] {
        return Ok(None);
    }
    let mut hexagon = Vec::new();
    let mut boxes = Vec::new();
    for cycle in circuits {
        let mut edges = graph
            .iter_edges_of(&cycle.filter)
            .map(|(_, id, _)| id.0)
            .collect::<Vec<_>>();
        edges.sort_unstable();
        if edges.len() == 6 {
            hexagon = edges;
        } else {
            let nodes = graph
                .iter_nodes_of(&cycle.filter)
                .map(|(id, _, _)| id.0)
                .collect::<BTreeSet<_>>();
            let mut legs = externals
                .iter()
                .filter(|(v, _)| nodes.contains(v))
                .map(|(_, pdg)| *pdg)
                .collect::<Vec<_>>();
            legs.sort_unstable();
            boxes.push(legs);
        }
    }
    top.sort_unstable();
    boxes.sort();
    // The s-channel has both incoming gluons on one box and both Higgs legs
    // on the other; canonical membership separately fixes the supplied D05.
    if hexagon != top || boxes != [vec![21, 21], vec![25, 25]] {
        return Ok(None);
    }
    Ok(Some(Topology {
        top_edges: top,
        gluon_edge: gluon[0],
        circuit_edge_counts: counts,
        hexagon_edges: hexagon,
        box_external_pdgs: boxes,
    }))
}
