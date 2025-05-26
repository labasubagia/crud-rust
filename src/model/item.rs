use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct Item {
    pub id: String,
    pub name: String,
}
