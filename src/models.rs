use serde::{Serialize, Deserialize};
use diesel::{Queryable, Insertable, AsChangeset, Selectable, RunQueryDsl, QueryDsl};
use crate::schema::{posts, users};
use diesel::SelectableHelper;
use validator::Validate;
use diesel::PgConnection;
use diesel::ExpressionMethods;
use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Queryable, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub name: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser { 
    pub username: String,
    pub password: String,
    pub name: String,
    pub email: String, 
    pub created_at: String,
}

#[derive(Debug, Validate, Serialize, Deserialize, Queryable, Insertable, AsChangeset, Selectable)]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: i32,
    #[validate(length(min = 1, message = "Author cannot be empty"))]
    pub author: String,
    #[validate(length(min = 1, message = "Title cannot be empty"))]
    pub title: String,
    #[validate(length(min = 1, message = "Text cannot be empty"))]
    pub text: String,
    #[validate(length(min = 1, message = "Date cannot be empty"))]
    pub created_at: String,
    pub edited: bool,
    pub edited_at: String,
}


#[derive(Debug, Validate, Serialize, Deserialize, Queryable, Insertable, AsChangeset, Selectable)]
#[diesel(table_name = posts)]
pub struct NewPost {
    #[validate(length(min = 1, message = "Author cannot be empty"))]
    pub author: String,
    #[validate(length(min = 1, message = "Title cannot be empty"))]
    pub title: String,
    #[validate(length(min = 1, message = "Text cannot be empty"))]
    pub text: String,
    pub created_at: String,
    pub edited: bool,
    pub edited_at: String,
}

impl NewUser {
    pub fn new(username: String, password: String, name: String, email: String, created_at: String) -> Result<Self, bcrypt::BcryptError> {
        
        let hashed_password = hash(password, DEFAULT_COST)?;

        Ok(Self {
            username,
            password: hashed_password,
            name,
            email,
            created_at,
        })
    }
}

impl User {
    pub fn verify_password(&self, password: &str) -> bool {
        verify(password, &self.password).unwrap_or(false)
    }

    pub fn create_user(conn: &mut PgConnection, new_user: NewUser) -> Result<User, diesel::result::Error> {
        use crate::schema::users::dsl::*;
        diesel::insert_into(users).values(&new_user).get_result(conn)
    }

    pub fn find_by_username(conn: &mut PgConnection, uname: &str) -> Result<User, diesel::result::Error> {
        use crate::schema::users::dsl::*;
        users.filter(username.eq(uname)).first(conn)
    }
}

impl Post {
    pub fn get_posts(conn: &mut PgConnection) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::schema::posts::dsl::*;
        posts.load::<Post>(conn)
    }

    pub fn new_post(conn: &mut PgConnection, new_post: NewPost) -> Result<Self, diesel::result::Error> {
        use crate::schema::posts::dsl::*;
        diesel::insert_into(posts)
            .values(&new_post)
            .returning(Post::as_returning())
            .get_result(conn)
    }

    pub fn update_post(conn: &mut PgConnection, post_id: i32, updated_post: Post) -> Result<Self, diesel::result::Error> {
        use crate::schema::posts::dsl::*;
        diesel::update(posts.find(post_id)).set(&updated_post).returning(Post::as_returning()).get_result(conn)
    }

    pub fn get_post(conn: &mut PgConnection, post_id: i32) -> Result<Self, diesel::result::Error> {
        use crate::schema::posts::dsl::*;
        posts.find(post_id).get_result(conn)
    }

    pub fn delete_post(conn: &mut PgConnection, post_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::schema::posts::dsl::*;
        diesel::delete(posts.find(post_id)).execute(conn)
    }

}
