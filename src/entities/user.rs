use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: Uuid,
    pub total_space_used: i64,
    pub updated_at: DateTime,
    pub last_auto_sync_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::file::Entity")]
    Files,
}

impl Related<super::file::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Files.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn new(
        user_id: Uuid,
        total_space_used: i64,
    ) -> Self {
        Self {
            user_id: Set(user_id),
            total_space_used: Set(total_space_used),
            updated_at: Set(chrono::Utc::now().naive_utc()),
            last_auto_sync_at: Set(None),
        }
    }
}