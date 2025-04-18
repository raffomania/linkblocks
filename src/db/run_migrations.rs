use std::collections::{HashMap, HashSet};

use sqlx::{
    Acquire, PgConnection,
    migrate::{AppliedMigration, Migrate, MigrateError, Migrator},
};

fn check_for_missing_applied_migrations(
    applied_versions: &HashSet<i64>,
    applied_migrations: &[AppliedMigration],
    migrator: &Migrator,
) -> Result<(), MigrateError> {
    if migrator.ignore_missing {
        return Ok(());
    }

    for applied_migration in applied_migrations {
        if !applied_versions.contains(&applied_migration.version) {
            return Err(MigrateError::VersionMissing(applied_migration.version));
        }
    }

    Ok(())
}

fn check_for_checksum_mismatches(
    applied_migrations: &[AppliedMigration],
    migrator: &Migrator,
) -> Result<(), MigrateError> {
    let by_versions: HashMap<_, _> = applied_migrations.iter().map(|m| (m.version, m)).collect();

    for migration in migrator.iter() {
        dbg!(migration.version);
        if migration.migration_type.is_down_migration() {
            continue;
        }

        if let Some(applied_migration) = by_versions.get(&migration.version) {
            if migration.checksum != applied_migration.checksum {
                return Err(MigrateError::VersionMismatch(migration.version));
            }
        }
    }

    Ok(())
}

pub(super) async fn run_migrations(
    migrator: &Migrator,
    conn: &mut PgConnection,
) -> Result<(), MigrateError> {
    if migrator.locking {
        conn.lock().await?;
    }

    conn.ensure_migrations_table().await?;

    // Check for partially applied migrations
    // (Supposedly does not happen because we wrap migrations in transactions)
    let version = conn.dirty_version().await?;
    if let Some(version) = version {
        return Err(MigrateError::Dirty(version));
    }

    let applied_migrations = conn.list_applied_migrations().await?;
    let applied_versions: HashSet<_> = migrator.iter().map(|m| m.version).collect();

    check_for_missing_applied_migrations(&applied_versions, &applied_migrations, migrator)?;
    check_for_checksum_mismatches(&applied_migrations, migrator)?;

    let to_apply = migrator.iter().filter(|m| {
        !applied_versions.contains(&m.version) && !m.migration_type.is_down_migration()
    });
    for migration in to_apply {
        // Wrap both SQL and our rust migrations into a transaction
        // to make sure it gets reverted if something in the rust code fails.
        let mut tx = conn.begin().await?;
        tx.apply(migration).await?;
        tx.commit().await?;
    }

    if migrator.locking {
        conn.unlock().await?;
    }

    Ok(())
}
