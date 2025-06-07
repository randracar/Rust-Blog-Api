# Rust-Blog-Api
An API for blogs made with actix web, diesel, bcrypt and chronos. Has an authentication system with registering and login.  

## Setting up

### Dependencies

[PostgreSQL](https://www.postgresql.org/download/)
[Rust](https://www.rust-lang.org/tools/install)
[Diesel CLI](https://diesel.rs/guides/getting-started#installing-diesel-cli)

All other dependencies are already included in the project.

### .env

Below is an example of the .env file, which should be in the root folder of the project.

```
DATABASE_URL="postgres://user:password@localhost/blogapi"
JWT_SECRET="vOMR2LCmh2Jhw4EQrxoxtSvAxR90eDcE"
RUST_LOG="info"
```

### Running migrations

Run database migrations with the command below. 

```
diesel migration run
```

### Running the program

Run the project with the command below.

```
cargo run
```
