// project to better learn how to code an api with rust and the most common crates like actix_web
// its not the first project i made in that scope but the first i will publish

/*

based out of https://blog.logrocket.com/create-backend-api-with-rust-postgres/#running-rust-api-demo-app
https://docs.rs/chrono/latest/chrono/
https://diesel.rs/guides/getting-started
https://github.com/actix/actix-web-httpauth/blob/master/examples/middleware.rs

*/

use actix_web::{web, App, HttpResponse, HttpServer, HttpRequest, dev::ServiceRequest, HttpMessage, FromRequest, dev::Payload};
use serde::{Serialize, Deserialize};
use serde_json::json;
use chrono::{Utc, Duration};
use models::{NewPost, Post, User, NewUser};
use date::format_time;
use diesel::result::Error as DieselError;
use diesel::r2d2;
use diesel::result::DatabaseErrorKind;
use validator::Validate;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};
use futures::future::{ready, Ready};


mod models;
mod schema;
mod date;

type DbPool = r2d2::Pool<r2d2::ConnectionManager<diesel::PgConnection>>;

type DbConn = r2d2::PooledConnection<r2d2::ConnectionManager<diesel::PgConnection>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: i32,
    name: String,
    exp: usize,
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
    name: String,
    email: String,
}

struct Authenticated(Claims);

fn map_diesel_error(e: DieselError) -> actix_web::Error {
    match e {
        DieselError::NotFound => actix_web::error::ErrorNotFound(e.to_string()),
        DieselError::DatabaseError(kind, _) => {
            match kind {
                DatabaseErrorKind::UniqueViolation => 
                actix_web::error::ErrorBadRequest("Username already exists"),
                _ => actix_web::error::ErrorInternalServerError(e.to_string()),
            }
        }
        _ => actix_web::error::ErrorInternalServerError(e),
    }
}


impl FromRequest for Authenticated {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let claims = req.extensions()
            .get::<Claims>()
            .cloned()
            .ok_or(actix_web::error::ErrorUnauthorized("Invalid token"));
        
        ready(claims.map(Authenticated))
    }
}

fn get_db_conn(pool: &web::Data<DbPool>) -> Result<DbConn, actix_web::Error> {
    pool.get()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))
}

#[derive(Debug, Validate, Deserialize)]
struct PostRequest {
    #[validate(length(min = 1, message = "Field cannot be empty"))]
    title: String,
    #[validate(length(min = 1, message = "Field cannot be empty"))]
    text: String,
}

fn validate_password(password: &str) -> Result<(), &'static str> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters");
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err("Password must contain at least one digit");
    }
    Ok(())
}

async fn login(
    pool: web::Data<DbPool>,
    data: web::Json<AuthRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut conn = get_db_conn(&pool)?;

    let user = User::find_by_username(&mut conn, &data.username)
        .map_err(map_diesel_error)?;

    if !user.verify_password(&data.password) {
        return Err(actix_web::error::ErrorUnauthorized("Invalid credentials"));
    }

    let expiration = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    let claims = Claims {
        sub: user.id,
        name: user.name.clone(),
        exp: expiration,
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(json!({ "token": token })))
}

async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let token = credentials.token();
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
    
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ) {
        Ok(token_data) => {
            req.extensions_mut().insert(token_data.claims);
            Ok(req)
        }
        Err(_) => {
            let error = actix_web::error::ErrorUnauthorized("Invalid token"); 
            Err((error, req))
        }
    }
}

async fn register(
    pool: web::Data<DbPool>,
    data: web::Json<RegisterRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut conn = get_db_conn(&pool)?;

    validate_password(&data.password).map_err(|e| actix_web::error::ErrorBadRequest(e))?;
    
    let new_user = NewUser::new(
        data.username.clone(),
        data.password.clone(),
        data.name.clone(),
        data.email.clone(),
        format_time(),
    ).map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    User::create_user(&mut conn, new_user)
    .map(|user| HttpResponse::Created().json(json!({
            "id": user.id,
            "name": user.name,
            "username": user.username,
    })))
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))
}

async fn create_post(
    pool: web::Data<DbPool>,
    post_data: web::Json<PostRequest>,
    auth: Authenticated,
) -> Result<HttpResponse, actix_web::Error> {

    /* For some reason, using just

    let claims = req.extensions().get::<Claims>()

    provided a bug because of memory ownership, which I fixed by doing this. */
    
    post_data.validate().map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let mut conn = get_db_conn(&pool)?;

    let new_post = NewPost {
        author: auth.0.name.clone(),
        title: post_data.title.clone(),
        text: post_data.text.clone(),
        created_at: format_time(),
        edited: false,
        edited_at: String::new(),
    };

    Post::new_post(&mut conn, new_post).map(|post| HttpResponse::Created().json(post)).map_err(map_diesel_error)
}

async fn update_post(
    pool: web::Data<DbPool>,
    post_id: web::Path<i32>,
    updated_post: web::Json<PostRequest>,
    auth: Authenticated,
) -> Result<HttpResponse, actix_web::Error> {

    let mut conn = get_db_conn(&pool)?;

    let post = Post::get_post(&mut conn, *post_id).map_err(map_diesel_error)?;

    updated_post.validate().map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    if post.author != auth.0.name {
        return Err(actix_web::error::ErrorForbidden("You can only edit your own posts dummy"));
    }

    let updated = Post {
        id: *post_id,
        author: post.author,
        title: updated_post.title.clone(),
        text: updated_post.text.clone(),
        created_at: post.created_at,
        edited: true,
        edited_at: format_time(),
    };

    Post::update_post(&mut conn, *post_id, updated).map(|post| HttpResponse::Ok().json(post))
        .map_err(map_diesel_error)
}

async fn get_all_posts(pool: web::Data<DbPool>) -> Result<HttpResponse, actix_web::Error> {
    let mut conn = get_db_conn(&pool)?;
    
    Post::get_posts(&mut conn).map(|posts| HttpResponse::Ok().json(posts)).map_err(map_diesel_error)
}

async fn delete_post(
    pool: web::Data<DbPool>,
    post_id: web::Path<i32>,
    auth: Authenticated,
) -> Result<HttpResponse, actix_web::Error> {
        
    let mut conn = get_db_conn(&pool)?;

    let post = Post::get_post(&mut conn, *post_id).map_err(map_diesel_error)?;

    if post.author != auth.0.name {
        return Err(actix_web::error::ErrorForbidden("You can only delete your own posts dummy. no censorship here"));
    }

    Post::delete_post(&mut conn, *post_id).map(|_| HttpResponse::NoContent().finish()).map_err(map_diesel_error)
}

async fn get_post(
    pool: web::Data<DbPool>,
    post_id: web::Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut conn = get_db_conn(&pool)?;

    Post::get_post(&mut conn, *post_id).map(|post| HttpResponse::Ok().json(post)).map_err(map_diesel_error)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    std::env::var("JWT_SECRET").expect("go set your secret in .env");
    let database_url = std::env::var("DATABASE_URL").expect("you forgot to set database url you dummy (go to .env)");

    let manager = r2d2::ConnectionManager::<diesel::PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder().build(manager).expect("Database pool creation failed. If this bug actually happends idk what to do");
    
    let auth = HttpAuthentication::bearer(validator);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(register))
                    .route("/login", web::post().to(login)),
            )
            .service(
                web::scope("/posts")
                    .wrap(auth.clone())
                    .route("", web::get().to(get_all_posts))
                    .route("", web::post().to(create_post))
                    .route("/{id}", web::put().to(update_post))
                    .route("/{id}", web::delete().to(delete_post))
                    .route("/{id}", web::get().to(get_post)),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}