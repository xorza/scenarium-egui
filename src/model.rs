use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum GraphFormat {
    Toml,
    Yaml,
    Json,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub pan: egui::Vec2,
    pub zoom: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub name: String,
    pub pos: egui::Pos2,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub node_id: Uuid,
    pub output_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub name: String,
    pub connection: Option<Connection>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    pub name: String,
}

impl Default for Node {
    fn default() -> Self {
        let id = Uuid::new_v4();
        let name = format!("Node {}", id);

        Self {
            id,
            name,
            pos: egui::Pos2::ZERO,
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }
}

impl Graph {
    pub fn validate(&self) -> Result<()> {
        if !self.zoom.is_finite() || self.zoom <= 0.0 {
            return Err(anyhow!("graph zoom must be finite and positive"));
        }
        if !self.pan.x.is_finite() || !self.pan.y.is_finite() {
            return Err(anyhow!("graph pan must be finite"));
        }

        let mut output_counts = HashMap::new();

        for node in &self.nodes {
            if !node.pos.x.is_finite() || !node.pos.y.is_finite() {
                return Err(anyhow!("node position must be finite"));
            }
            let prior = output_counts.insert(node.id, node.outputs.len());
            if prior.is_some() {
                return Err(anyhow!("duplicate node id detected"));
            }
        }

        for node in &self.nodes {
            for input in &node.inputs {
                if let Some(connection) = &input.connection {
                    let output_count = output_counts
                        .get(&connection.node_id)
                        .ok_or_else(|| anyhow!("connection references a missing node"))?;
                    if connection.output_index >= *output_count {
                        return Err(anyhow!("connection output index out of range"));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn serialize(&self, format: GraphFormat) -> Result<String> {
        self.validate()?;

        match format {
            GraphFormat::Json => serde_json::to_string_pretty(self).map_err(anyhow::Error::from),
            GraphFormat::Yaml => serde_yml::to_string(self).map_err(anyhow::Error::from),
            GraphFormat::Toml => toml::to_string(self).map_err(anyhow::Error::from),
        }
    }

    pub fn deserialize(format: GraphFormat, input: &str) -> Result<Self> {
        if input.trim().is_empty() {
            bail!("graph input is empty");
        }

        let graph = match format {
            GraphFormat::Json => {
                serde_json::from_str::<Graph>(input).map_err(anyhow::Error::from)?
            }
            GraphFormat::Yaml => {
                serde_yml::from_str::<Graph>(input).map_err(anyhow::Error::from)?
            }
            GraphFormat::Toml => toml::from_str::<Graph>(input).map_err(anyhow::Error::from)?,
        };
        graph.validate()?;

        Ok(graph)
    }

    pub fn test_graph() -> Self {
        let value_a_id = Uuid::new_v4();
        let value_b_id = Uuid::new_v4();
        let sum_id = Uuid::new_v4();
        let divide_id = Uuid::new_v4();
        let output_id = Uuid::new_v4();

        let value_a = Node {
            id: value_a_id,
            name: "value_a".to_string(),
            pos: egui::pos2(80.0, 120.0),
            inputs: Vec::new(),
            outputs: vec![Output {
                name: "value".to_string(),
            }],
        };

        let value_b = Node {
            id: value_b_id,
            name: "value_b".to_string(),
            pos: egui::pos2(80.0, 260.0),
            inputs: Vec::new(),
            outputs: vec![Output {
                name: "value".to_string(),
            }],
        };

        let sum = Node {
            id: sum_id,
            name: "math(sum)".to_string(),
            pos: egui::pos2(320.0, 180.0),
            inputs: vec![
                Input {
                    name: "a".to_string(),
                    connection: Some(Connection {
                        node_id: value_a_id,
                        output_index: 0,
                    }),
                },
                Input {
                    name: "b".to_string(),
                    connection: Some(Connection {
                        node_id: value_b_id,
                        output_index: 0,
                    }),
                },
            ],
            outputs: vec![Output {
                name: "sum".to_string(),
            }],
        };

        let divide = Node {
            id: divide_id,
            name: "math(divide)".to_string(),
            pos: egui::pos2(560.0, 180.0),
            inputs: vec![
                Input {
                    name: "sum".to_string(),
                    connection: Some(Connection {
                        node_id: sum_id,
                        output_index: 0,
                    }),
                },
                Input {
                    name: "b".to_string(),
                    connection: Some(Connection {
                        node_id: value_b_id,
                        output_index: 0,
                    }),
                },
            ],
            outputs: vec![Output {
                name: "divide".to_string(),
            }],
        };

        let output = Node {
            id: output_id,
            name: "output".to_string(),
            pos: egui::pos2(800.0, 180.0),
            inputs: vec![Input {
                name: "value".to_string(),
                connection: Some(Connection {
                    node_id: divide_id,
                    output_index: 0,
                }),
            }],
            outputs: Vec::new(),
        };

        let graph = Self {
            nodes: vec![value_a, value_b, sum, divide, output],
            pan: egui::Vec2::ZERO,
            zoom: 1.0,
        };

        assert!(graph.nodes.len() == 5, "test_graph must contain 5 nodes");

        graph
    }
}

#[test]
fn test_graph() {
    let graph = Graph::test_graph();
    assert!(graph.validate().is_ok());
}

#[test]
fn graph_roundtrip_json() {
    assert_roundtrip(GraphFormat::Json);
}

#[test]
fn graph_roundtrip_yaml() {
    assert_roundtrip(GraphFormat::Yaml);
}

#[test]
fn graph_roundtrip_toml() {
    assert_roundtrip(GraphFormat::Toml);
}

fn assert_roundtrip(format: GraphFormat) {
    let graph = Graph::test_graph();
    let serialized = graph
        .serialize(format)
        .expect("graph serialization should succeed for test graph");
    assert!(
        !serialized.trim().is_empty(),
        "serialized graph should not be empty"
    );
    let deserialized = Graph::deserialize(format, &serialized)
        .expect("graph deserialization should succeed for test payload");
    assert!(deserialized.validate().is_ok());
    assert_eq!(
        graph.nodes.len(),
        deserialized.nodes.len(),
        "node counts should round-trip"
    );
    assert_eq!(
        graph.nodes[0].id, deserialized.nodes[0].id,
        "node ids should round-trip"
    );
    assert_eq!(graph.zoom, deserialized.zoom, "zoom should round-trip");
    assert_eq!(graph.pan, deserialized.pan, "pan should round-trip");
}
