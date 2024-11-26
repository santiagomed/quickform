use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub name: String,
    pub description: String,
    pub fields: Vec<Field>,
    pub features: EntityFeatures,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EntityFeatures {
    pub has_auth: bool,
    pub needs_search: bool,
    pub needs_soft_delete: bool,
    pub has_timestamps: bool,
}
