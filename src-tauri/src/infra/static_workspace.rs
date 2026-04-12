use crate::domain::workspace::WorkspaceBootstrapSnapshot;

pub fn load_snapshot() -> WorkspaceBootstrapSnapshot {
    serde_json::from_str(include_str!("../../../src/data/workspace-bootstrap.json"))
        .expect("工作台预览快照必须是合法 JSON")
}
