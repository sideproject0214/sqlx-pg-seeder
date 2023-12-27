# sqlx-pg-seeder

This crate facilitates easy data input using Sqlx and Postgres.

## 0. Changes [2023-12-27 14:42:59]
- 0.1.3.version : The issue where the settings for JSONB and array field names were in string format in version 0.1.2. has been fixed. <br> Starting from version 0.1.3., users can now specify the JSONB and array field names within 'pg-seeder.toml' for customization

## 1. Workflow

This crate primarily operates with API servers (e.g., axum, actix). When you place seed files in the designated folder (seeder/task) in the desired JSON format, upon successful completion of the seed task, the file moves from seeder/task to seeder/success.

Upon restarting the server, if no JSON files exist in seeder/task, the crate skips that folder. Therefore, if there's data that needs seeding, placing files in JSON format within that folder (seeder/task) will automatically initiate the seed task. (The location of seed-related folders can be modified in the pg-seeder.toml file.)


## 2. Installation and Usage

### (1) Dependencies(Please install using one of the following options)

```toml
[dependencies]
sqlx-pg-seeder = "0.1.3"
```
```bash
cargo add sqlx-pg-seeder
```

### (2) pg-seeder.toml (Configuration for Seed Folder Location, etc.)

```toml
task_folder = "src/seeders/task"
success_folder = "src/seeders/success"
created_at_name = "created_at"
updated_at_name = "updated_at"
jsonb_name = "size"
array_string_name = "thumbnail_src"
```

**task_folder**: Location for placing seed files.<br><br>
**success_folder**: Destination for files after successful seeding.<br><br>
**created_at_name and updated_at_name**: Names of fields in the JSON file, allowing customization for timestamp fields. <br>
- here is no limit to the number of JSON files. Even if there are multiple JSON files, they will all move together after the seed is completed. At this time, types other than the following string are directly bound to their pure values read by serde_json. Since the timestamp type in Postgres is not supported by chrono, a separate trait must be created and used. Therefore, based on the field names of the JSON files, a method of separating the respective values is used. As a convention, you can set a separate name for the field in case 'created_at (document creation date),' which is commonly used, is not used.



### (3) Migration Setup
```sql
create extension if not exists "uuid-ossp";

create table if not exists users (
    "id"  serial primary key,
    "uuid" uuid default uuid_generate_v4() not null unique,
    "name" varchar(50) not null,
    "email" varchar(100) not null unique,
    "password" varchar(100) not null,
    "is_admin" boolean not null default false,
    "google_id" varchar(100) unique,
    "naver_id" varchar(100) unique,
    "kakao_id" varchar(100) unique,
    "email_token" text,
    "is_verified" boolean not null default false,
    "pw_email_address" varchar(100),
    "created_at" timestamp not null default now(),
    "updated_at" timestamp not null default now()
);

create table if not exists posts (
    "id"  serial primary key,
    "uuid" uuid default uuid_generate_v4() not null,
    "user_id" uuid not null,
    "title" varchar(100),
    "image_src" text not null,
    "thumbnail_src" text[],
    "description" text not null,
    "brand" varchar(100) not null,
    "category" varchar(100) not null,
    "size"  jsonb not null,
    "price" bigint not null default 0,
    "count_in_stock" bigint not null default 0,
    "rating" double precision not null default 0,
    "num_reviews" bigint not null default 0,
    "sale" bigint not null default 0,
    "free_shipping" bool not null default false,
    "delivery_fee" bigint not null default 0,
    "created_at" timestamp not null default now(),
    "updated_at" timestamp not null default now(),

    constraint fk_user foreign key ("user_id") references "users" ("uuid") on delete cascade
);
```
>[ Caution ]<br> Use types: bigint (integer), double precision (). Rust currently supports type coercion as as_i64, as_f64 by default, so it cannot be defined smaller than these types.

### (4) Server Setup
#### (4-1) How to make a pool and migrations
```rust
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

  let my_pool = sqlx::postgres::PgPool::connect(&pg_url).await.unwrap();

  let migrate = migrate!("./src/migrations").run(&my_pool).await;

  match migrate {
    Ok(()) => println!("sqlx migration success"),
    Err(e) => println!("sqlx migration error : {:?}", e),
  }

  my_pool
}
```

<br>

 #### (4-2) How to connect a seeder

```rust
use sqlx_pg_seeder::seeder; 

... 

#[tokio::main]
async fn main() {
  
  ...

  let pool = get_db_conn(&my_env_value).await;

  seeder(&pool).await;

}

```

  

<br>

**The key point** here is to place a pool connecting PostgreSQL and sqlx within the seeder.<br><br> This is essential for using sqlx in frameworks like axum, actix, etc. The pool (which creates a pool connecting to the database in advance, retrieves the connection whenever a connection request comes in, utilizes it, and returns it afterward, efficiently managing and reusing database connections) is crucial.<br><br> ※ The general method of creating a pool has been outlined, but specific details may vary slightly for each server framework. Here, we're focusing on axum.

### (5) Seed Files
The seed file name corresponds to the table's name and matches the field names in the JSON. By default, the initial seed file is located in the 'src/seeder/task' folder according to the default settings.

- **users.json**
```json
{
  "users": [
    {
      "uuid": "2f806f04-949b-4c28-a091-08a0905ea3ab",
      "name": "Admin User",
      "email": "admin@example.com",
      "password": "$2a$10$L/YmXVQY1JGYzJ2/XQULQOgNznOZ21z4.MWmq0TSoskHX25oBXHOa",
      "is_admin": true
    },
    {
      "uuid": "2f806f04-949b-4c28-a091-08a0905ea2ab",
      "name": "IU",
      "email": "iu@example.com",
      "password": "$2a$10$L/YmXVQY1JGYzJ2/XQULQOgNznOZ21z4.MWmq0TSoskHX25oBXHOa",
      "is_admin": false
    },
    {
      "uuid": "2f806f04-949b-4c28-a091-08a0905ea3bb",
      "name": "SSaple",
      "email": "ssaple@example.com",
      "password": "$2a$10$L/YmXVQY1JGYzJ2/XQULQOgNznOZ21z4.MWmq0TSoskHX25oBXHOa",
      "is_admin": true
    }]
}
```

- **posts.json** 

  
```json
{
  "posts": [
    {
      "user_id": "2f806f04-949b-4c28-a091-08a0905ea3ab",
      "title": "Set of 4 Business Shirts",
      "image_src": "https://w3s.link/ipfs/bafybeidi7o3o3wiosdqidzrwbiwrgpm3jjltbw22caztuq2xdgtectoj6i/man-shirts-1.jpg",
      "thumbnail_src": [
        "https://w3s.link/ipfs/bafybeidi7o3o3wiosdqidzrwbiwrgpm3jjltbw22caztuq2xdgtectoj6i/man-shirts-1.jpg",
        "https://w3s.link/ipfs/bafybeidujgajt4dby7ws6ii7mg5gkwwjddqoqyvsk4lwisufeupti4fg7i/man-shirts-2.png",
        "https://w3s.link/ipfs/bafybeicbjhyhmokbodeocaxfxahdl4gsxuqqkyeoqsvm46wzqmby7ehcba/footwear-1.png",
        "https://w3s.link/ipfs/bafybeicfvkf34c4rrckfo7h2aipkbgbcnm442bxuot5irbpwoqg7shokfu/footwear-2.png"
      ],
      "description": "A must-have item for your wardrobe! The Polo Boys Oxford Shirt, perfect for matching with both casual and formal looks.",
      "brand": "Polo",
      "category": "Mans",
      "size": { "95": 3, "100": 10, "105": 10, "110": 7 },
      "price": 120000,
      "count_in_stock": 30,
      "rating": 5,
      "num_reviews": 1,
      "sale": 30,
      "free_shipping": true,
      "delivery_fee": 0,
      "created_at": "2022-01-01T00:01:02Z",
      "updated_at": "2022-01-02T00:01:02Z"
    },
    {
      "user_id": "2f806f04-949b-4c28-a091-08a0905ea3ab",
      "title": "Set of 2 Business Shirts",
      "image_src": "https://w3s.link/ipfs/bafybeidujgajt4dby7ws6ii7mg5gkwwjddqoqyvsk4lwisufeupti4fg7i/man-shirts-2.png",
      "thumbnail_src": [
        "https://w3s.link/ipfs/bafybeidujgajt4dby7ws6ii7mg5gkwwjddqoqyvsk4lwisufeupti4fg7i/man-shirts-2.png",
        "https://w3s.link/ipfs/bafybeidi7o3o3wiosdqidzrwbiwrgpm3jjltbw22caztuq2xdgtectoj6i/man-shirts-1.jpg",
        "https://w3s.link/ipfs/bafybeicbjhyhmokbodeocaxfxahdl4gsxuqqkyeoqsvm46wzqmby7ehcba/footwear-1.png",
        "https://w3s.link/ipfs/bafybeicfvkf34c4rrckfo7h2aipkbgbcnm442bxuot5irbpwoqg7shokfu/footwear-2.png"
      ],
      "description": "A must-have item for the transitional season! Get your hands on affordable business shirts!",
      "brand": "Ralph Lauren",
      "category": "Mans",
      "size": { "95": 3, "100": 9, "105": 10, "110": 7 },
      "price": 90000,
      "count_in_stock": 29,
      "rating": 3,
      "num_reviews": 2,
      "sale": 10,
      "free_shipping": false,
      "delivery_fee": 3500,
      "created_at": "2022-01-03T00:01:02Z",
      "updated_at": "2022-01-04T00:01:02Z"
    }]
}
```
<br>

> I used web3.storage, which utilizes Filecoin IPFS, for storing the images.

## 3. Miscellaneous
The sqlx-pg-seeder supports commonly used types in Postgres and is adaptable to databases supported by sqlx.

## 4. Contributing
- Fork the repository.
- Create a new branch (git checkout -b feature).
- Commit your changes (git commit -am 'Add new feature').
- Push to the branch (git push origin feature).
- Create a Pull Request.

## 5. License: MIT
This crate is distributed under the [MIT License], allowing anyone to use, modify, replicate, and distribute the software, provided proper copyright notices.