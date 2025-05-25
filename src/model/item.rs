use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
}
