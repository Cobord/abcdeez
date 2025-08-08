use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TopologyType {
    Linear,
    Cyclic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub position: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topology {
    pub topology_type: TopologyType,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub node_map: HashMap<String, usize>,
}

impl Topology {
    pub fn new_linear(items: Vec<String>) -> Self {
        let n = items.len();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map = HashMap::new();

        for (i, label) in items.iter().enumerate() {
            let id = format!("node_{}", i);
            nodes.push(Node {
                id: id.clone(),
                label: label.clone(),
                position: i as f64,
            });
            node_map.insert(id.clone(), i);

            if i < n - 1 {
                edges.push(Edge {
                    from: id.clone(),
                    to: format!("node_{}", i + 1),
                    weight: 1.0,
                });
            }
        }

        Topology {
            topology_type: TopologyType::Linear,
            nodes,
            edges,
            node_map,
        }
    }

    pub fn new_cyclic(items: Vec<String>) -> Self {
        let n = items.len();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map = HashMap::new();

        for (i, label) in items.iter().enumerate() {
            let id = format!("node_{}", i);
            let angle = (2.0 * PI * i as f64) / n as f64;
            nodes.push(Node {
                id: id.clone(),
                label: label.clone(),
                position: angle,
            });
            node_map.insert(id.clone(), i);

            let next_idx = (i + 1) % n;
            edges.push(Edge {
                from: id.clone(),
                to: format!("node_{}", next_idx),
                weight: 1.0,
            });
        }

        Topology {
            topology_type: TopologyType::Cyclic,
            nodes,
            edges,
            node_map,
        }
    }

    pub fn alphabet() -> Self {
        let alphabet: Vec<String> = (b'A'..=b'Z').map(|c| (c as char).to_string()).collect();
        Self::new_linear(alphabet)
    }

    pub fn days_of_week() -> Self {
        let days = vec![
            "Monday".to_string(),
            "Tuesday".to_string(),
            "Wednesday".to_string(),
            "Thursday".to_string(),
            "Friday".to_string(),
            "Saturday".to_string(),
            "Sunday".to_string(),
        ];
        Self::new_cyclic(days)
    }

    pub fn get_successor(&self, node_id: &str) -> Option<String> {
        self.edges
            .iter()
            .find(|e| e.from == node_id)
            .map(|e| e.to.clone())
    }

    pub fn get_predecessor(&self, node_id: &str) -> Option<String> {
        self.edges
            .iter()
            .find(|e| e.to == node_id)
            .map(|e| e.from.clone())
    }

    pub fn get_distance(&self, from: &str, to: &str) -> Option<usize> {
        let from_idx = self.node_map.get(from)?;
        let to_idx = self.node_map.get(to)?;

        match self.topology_type {
            TopologyType::Linear => Some((*to_idx as i32 - *from_idx as i32).abs() as usize),
            TopologyType::Cyclic => {
                let n = self.nodes.len();
                let forward = (*to_idx + n - from_idx) % n;
                let backward = (*from_idx + n - to_idx) % n;
                Some(forward.min(backward))
            }
        }
    }

    pub fn get_k_jump(&self, start: &str, k: i32) -> Option<String> {
        let start_idx = self.node_map.get(start)?;
        let n = self.nodes.len();

        let target_idx = match self.topology_type {
            TopologyType::Linear => {
                let new_idx = *start_idx as i32 + k;
                if new_idx < 0 || new_idx >= n as i32 {
                    return None;
                }
                new_idx as usize
            }
            TopologyType::Cyclic => {
                let new_idx = (*start_idx as i32 + k).rem_euclid(n as i32);
                new_idx as usize
            }
        };

        Some(format!("node_{}", target_idx))
    }

    pub fn get_node_by_label(&self, label: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.label == label)
    }

    pub fn get_node_by_id(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn is_before(&self, a: &str, b: &str) -> Option<bool> {
        let a_node = self.get_node_by_label(a)?;
        let b_node = self.get_node_by_label(b)?;

        match self.topology_type {
            TopologyType::Linear => Some(a_node.position < b_node.position),
            TopologyType::Cyclic => {
                None
            }
        }
    }

    pub fn get_segment(&self, start: &str, count: usize, reverse: bool) -> Vec<String> {
        let mut result = Vec::new();
        let start_node = match self.get_node_by_label(start) {
            Some(n) => n,
            None => return result,
        };

        let mut current_id = start_node.id.clone();
        result.push(start_node.label.clone());

        for _ in 1..count {
            let next_id = if reverse {
                self.get_predecessor(&current_id)
            } else {
                self.get_successor(&current_id)
            };

            match next_id {
                Some(id) => {
                    if let Some(node) = self.get_node_by_id(&id) {
                        result.push(node.label.clone());
                        current_id = id;
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_topology() {
        let topo = Topology::alphabet();
        assert_eq!(topo.nodes.len(), 26);
        assert_eq!(topo.get_node_by_label("A").unwrap().position, 0.0);
        assert_eq!(topo.get_node_by_label("Z").unwrap().position, 25.0);
        assert!(topo.is_before("A", "B").unwrap());
        assert!(!topo.is_before("Z", "A").unwrap());
    }

    #[test]
    fn test_cyclic_topology() {
        let topo = Topology::days_of_week();
        assert_eq!(topo.nodes.len(), 7);
        assert_eq!(topo.get_distance("node_0", "node_3"), Some(3));
        assert_eq!(topo.get_distance("node_6", "node_1"), Some(2));
    }

    #[test]
    fn test_segment() {
        let topo = Topology::alphabet();
        let segment = topo.get_segment("D", 3, false);
        assert_eq!(segment, vec!["D", "E", "F"]);

        let reverse_segment = topo.get_segment("D", 3, true);
        assert_eq!(reverse_segment, vec!["D", "C", "B"]);
    }
}