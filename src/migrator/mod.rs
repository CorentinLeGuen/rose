use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251112_140000_create_tables::Migration),
            Box::new(m20251114_174900_add_space_used_triggers::Migration),
        ]
    }
}

pub mod m20251112_140000_create_tables;
pub mod m20251114_174900_add_space_used_triggers;