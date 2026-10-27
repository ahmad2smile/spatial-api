use std::path::Path;

use sqlx::{PgPool, migrate::Migrator};
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt, core::IntoContainerPort, core::WaitFor,
    runners::AsyncRunner,
};

/// The container is returned so the caller keeps it alive. Dropping it stops the database.
pub async fn create_db_pool() -> (ContainerAsync<GenericImage>, PgPool) {
    let container = GenericImage::new("postgis/postgis", "17-3.5")
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_PASSWORD", "password")
        .start()
        .await
        .unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();

    let pool = PgPool::connect(&format!("postgres://postgres:password@127.0.0.1:{port}"))
        .await
        .unwrap();
    Migrator::new(Path::new("./migrations"))
        .await
        .unwrap()
        .run(&pool)
        .await
        .unwrap();

    (container, pool)
}
