use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(Customer::Table)
                    .if_not_exists()
                    .col(pk_auto(Customer::Id))
                    .col(integer(Customer::Number))
                    .col(string(Customer::Name))
                    .col(date(Customer::Date))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UnableDate::Table)
                    .if_not_exists()
                    .col(pk_auto(UnableDate::Id))
                    .col(date(UnableDate::Date))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Counter::Table)
                    .if_not_exists()
                    .col(pk_auto(Counter::Id))
                    .col(date(Counter::Date))
                    .col(integer(Counter::Number))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop()
                .table(Customer::Table)
                .table(UnableDate::Table)
                .table(Counter::Table)
                .to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Customer {
    Table,
    Id,
    Number,
    Name,
    Date,
}

#[derive(DeriveIden)]
enum UnableDate {
    Table,
    Id,
    Date,
}

#[derive(DeriveIden)]
enum Counter {
    Table,
    Id,
    Date,
    Number,
}