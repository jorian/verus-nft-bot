# Verusnft

## Setup

First we need Docker.
Then we need to setup postgresql for the database.

`docker pull postgres:alpine`  
`docker run --name postgres-test -e POSTGRES_PASSWORD=password -d -p 5432:5432 postgres:alpine`

At this point we need the DATABASE_URL to be set:  
`DATABASE_URL=<url>`  
`sqlx database create`  
`cargo sqlx migrate run`

Because we're using both a `lib` and a `bin` here, we need to run extra arguments:  
`cargo sqlx prepare -- --bin verusnft`

## Docker (needs work)

This uses Docker. In order to make it run, run `cargo sqlx prepare` to prepare the `query!()` statements into a sqlx-data.json.

Then, start docker:  
`docker build --tag verusnft --file Dockerfile .`  
`docker run --network="host" --name=verusnft verusnft`

Make sure postgres is running in its own container and has port 5432 mapped.

## Postgresql

`docker exec -it postgres-test bash`  
`psql -U postgres`  
`\c test0` connects to database  
`\dt` shows tables in database
