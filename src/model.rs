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
