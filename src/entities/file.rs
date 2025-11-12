use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "files")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub file_key: Uuid,
    pub user_id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub content_type: String,
    pub content_size: i64,
    pub version: String,
    pub is_latest: bool,
    pub added_at: DateTime,
    pub deletion_mark_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm (
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::UserId",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn new(
        user_id: Uuid,
        file_name: String,
        file_path: String,
        content_type: String,
        content_size: i64,
        version: String,
    ) -> Self {
        Self {
            file_key: Set(Uuid::now_v7()),
            user_id: Set(user_id),
            file_name: Set(file_name),
            file_path: Set(file_path),
            content_type: Set(content_type),
            content_size: Set(content_size),
            version: Set(version),
            is_latest: Set(true),
            added_at: Set(chrono::Utc::now().naive_utc()),
            deletion_mark_at: Set(None),
        }
    }
}