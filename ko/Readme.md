# sqlx-pg-seeder

이 크레이트는 Sqlx와 Postgres를 사용하여 데이터 입력을 간편하게 도와줍니다.

## 1. Workflow (작동 관련 큰 흐름)

이 크레이트는 기본적으로 API 서버(axum, actix 등)와 함께 작동합니다. 지정된 폴더(seeder/task)에 seed를 원하는 JSON 형식의 파일을 넣으면, seed 작업이 성공적으로 완료되면 해당 파일은 seeder/task 폴더에서 seeder/success 폴더로 이동합니다.

다음 서버를 다시 시작할 때, seeder/task 폴더에 JSON 파일이 없으면 크레이트는 해당 폴더를 건너뛰게 됩니다. 따라서 seed를 수행해야 하는 데이터가 있으면, 해당 폴더(seeder/task)에 JSON 형태로 파일을 넣어두면 자동으로 seed 작업이 이루어집니다. (seed 관련 폴더 위치는 pg-seeder.toml 파일에서 수정할 수 있습니다)

## 2. 설치 및 사용 방법

### (1) Dependencies : 아래 둘중 하나의 방법으로 설치

```toml
[dependencies]
sqlx-pg-seeder = "0.1.0"
```
```bash
cargo add sqlx-pg-seeder
```
### (2) pg-seeder.toml (Seed 폴더 위치 등 설정)

```toml
task_folder = "src/seeders/task"
success_folder = "src/seeders/success"
created_at_name = "created_at"
updated_at_name = "updated_at"
```

**task_folder**: seed 파일을 놓는 위치입니다.
- JSON 파일의 개수에는 제한이 없습니다.

**success_folder**: seed 완료 후 파일이 이동하는 폴더입니다.
- JSON 파일이 여러 개더라도 모두 한꺼번에 seed 완료 후 이동합니다.

**created_at_name**: pg-sqlx-seeder는 기본적으로 JSON 파일을 열어서 String으로 변환합니다. <br>
- 이때 다음 String 외의 타입들은 serde_json으로 읽은 순수 값을 직접 해당 bind 합니다. 이때 postgres의 timestamp 타입은 chrono에서 지원하지 않아 별도의 트레이트를 만들어서 사용해야 합니다. 따라서 json 파일의 **필드 이름**을 기준으로 해당 값을 분리하는 방식을 사용하고 있어, 관례적으로 많이 사용하는 'created_at(문서 생성일)' 이라는 필드를 사용하지 않을 경우를 대비해 해당 필드의 이름을 별도로 설정할 수 있습니다.

**updated_at_name**: 값 수정일로 'created_at_name'과 동일한 이유로 별도 이름을 설정할 수 있습니다.

### (3) migration 설정
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
>**[ 주의사항 ]**<br> 
> 숫자 : bigint(정수), double precision() 타입을 사용한다. 이는 러스트는 현재 기본적으로 타입화인시 as_i64, as_f64만 지원하기에 이보다 작게 정의할 수 없다. 

### (4) 서버 세팅
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
  seeder(&pool).await;

  // setting
}

```

```rust 
  let pool = get_db_conn(&my_env_value).await;
  seeder(&pool).await;
```
<br>
여기서 핵심은 다음과 같이 seeder 안에 postgres와 sqlx를 연결하는 pool를 넣어야 한다는 것이다.<br><br>
이는 axum, actix 등 모두 sqlx를 사용하기 위해서 pool(사전에 DB와 연결하는 pool를 만들어 연결요청이 올때마다 해당 연결을 가져와 사용한 후 끝이나면 반납하는식으로 DB 연결을 효율적으로 관리하고 재사용한다) <br><br>
※ pool 만드는 법은 개략적으로 적었고 세부사항은 각 서버 프레임워크 마다 조금씩 다를 수 있다. 여기서는 axum을 기준으로 한다. 

### (5) 시드 파일
시드 파일 이름은 테이블의 이름으로 적고 json의 필드 이름과 일치시킨다. 기본 설정에 따르면 처음에 시드 파일은 'src/seeder/task' 폴더에 위치한다

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
이미지는 파일코인 ipfs를 이용하고 있는 web3.storage를 사용했다

## 3. 기타
sqlx-pg-seeder는 기본적으로 postgres에서 많이(저를 기준으로 ^^;) 사용하는 타입 위주로 지원하고 있다. sqlx가 지원하는 DB는 기본적으로 이 크레이트가 작동하는 방식으로 모두 적용이 가능할 것이라고 생각한다. 

## 4. 기여 방법
- Fork 해주세요<br>
- 새로운 branch를 생성해주세요 (git checkout -b feature)<br>
- 변경 사항을 커밋해주세요 (git commit -am 'Add new feature')<br>
- 변경 사항을 푸시해주세요 (git push origin feature)<br>
- Pull Request를 생성해주세요<br>

## 5. 라이센스 : MIT
이 크레이트는 [MIT 라이센스] 라이센스 하에 배포됩니다.
> 모든 사람이 해당 소프트웨어를 사용, 수정, 복제, 배포할 수 있도록 허용합니다. 다만 저작권 고지 및 면책 조항을 포함시켜야 합니다.