use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Relationship {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub relationship_type: RelationshipType,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationshipType {
    OneToOne,
    OneToMany,
    ManyToMany,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFeatures {
    pub auth: AuthFeatures,
    pub database: DatabaseFeatures,
    pub email: EmailFeatures,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthFeatures {
    #[serde(rename = "type")]
    pub auth_type: AuthType,
    pub magic_link: bool,
    pub password: bool,
    pub oauth: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthType {
    Jwt,
    Session,
}

impl Default for AuthFeatures {
    fn default() -> Self {
        Self {
            auth_type: AuthType::Jwt,
            magic_link: false,
            password: false,
            oauth: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseFeatures {
    pub service: DatabaseService,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DatabaseService {
    MongoDB,
    Postgres,
    Supabase,
    Firebase,
}

impl Default for DatabaseFeatures {
    fn default() -> Self {
        Self {
            service: DatabaseService::MongoDB,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EmailFeatures {
    pub service: EmailService,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmailService {
    Resend,
    Sendgrid,
    Mailgun,
}

impl Default for EmailFeatures {
    fn default() -> Self {
        Self {
            service: EmailService::Resend,
        }
    }
}

impl Default for ProjectFeatures {
    fn default() -> Self {
        Self {
            auth: AuthFeatures::default(),
            database: DatabaseFeatures::default(),
            email: EmailFeatures::default(),
        }
    }
}

impl Default for Requirements {
    fn default() -> Self {
        Self {
            entities: vec![],
            relationships: vec![],
            project_features: ProjectFeatures::default(),
        }
    }
}
