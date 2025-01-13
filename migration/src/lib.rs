pub use sea_orm_migration::prelude::*;

mod m20240318_000001_create_base_schema;
mod m20240319_000001_create_users_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240318_000001_create_base_schema::Migration),
            Box::new(m20240319_000001_create_users_table::Migration),
        ]
    }
}
