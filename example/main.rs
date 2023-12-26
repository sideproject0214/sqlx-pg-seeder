use sqlx::{migrate, FromRow, Pool, Postgres};

#[derive(FromRow)]
pub struct EntityUuid {
  pub uuid: Uuid,
}

#[derive(Clone)]
pub struct DbRepo {
  my_pool: Pool<Postgres>,
}
pub trait DbPoolGetter {
  type Output;
  fn get_pool(&self) -> &Self::Output;
}

impl DbRepo {
  pub async fn init(my_env: &EnvValue) -> Self {
    Self {
      my_pool: get_db_conn(&my_env).await,
    }
  }
}

impl DbPoolGetter for DbRepo {
  type Output = Pool<Postgres>;

  fn get_pool(&self) -> &Self::Output {
    &self.my_pool
  }
}

pub async fn get_db_conn(my_env: &EnvValue) -> Pool<Postgres> {
  println!("Get DB Connect Start!");
  let pg_dialect = &my_env.db_dialect;
  let pg_username = &my_env.db_username;
  let pg_password = &my_env.db_password;
  let pg_host = &my_env.db_host;
  let pg_port = &my_env.db_port;
  let pg_database = &my_env.db_database;

  let pg_url = format!(
    "{pg_dialect}://{pg_username}:{pg_password}@{pg_host}:{pg_port}/{pg_database}"
  );
  println!("DB URL : {:?}", &pg_url);
  let my_pool = sqlx::postgres::PgPool::connect(&pg_url).await.unwrap();

  let migrate = migrate!("./src/migrations").run(&my_pool).await;

  match migrate {
    Ok(()) => println!("sqlx migration success"),
    Err(e) => println!("sqlx migration error : {:?}", e),
  }

  my_pool
}

#[tokio::main]
async fn main() {
  // setting
  let pool = get_db_conn(&my_env_value).await;
  let _seeder = seeder(&pool).await;

  // setting
}
