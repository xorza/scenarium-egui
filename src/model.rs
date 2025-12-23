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
