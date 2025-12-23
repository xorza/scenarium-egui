use anyhow::{Result, anyhow};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
}

#[derive(Debug)]
pub struct Node {
    pub id: Uuid,
    pub name: String,
    pub pos: egui::Pos2,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
}

#[derive(Debug)]
pub struct Connection {
    pub node_id: Uuid,
    pub output_index: usize,
}

#[derive(Debug)]
pub struct Input {
    pub name: String,
    pub connection: Option<Connection>,
}

#[derive(Debug)]
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
        let mut output_counts = HashMap::new();

        for node in &self.nodes {
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
