#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use crate::db_operations::*;
use dotenv::dotenv;
use postgres::{Client, NoTls};
use rocket::{http::Cookies, request::Form, Config, State};
use rocket_contrib::{serve::StaticFiles, templates::Template};
use std::{env, sync::Mutex};

mod db_operations;

#[derive(Serialize)]
pub struct Review {
    about: String,
    about_class: String,
    time: String,
    content: String,
    by_user: [String; 5],
}

#[derive(Serialize)]
struct TemplateContext {
    value: Option<String>,
    user_details: Option<Vec<String>>,
    reviews: Option<Vec<Review>>,
}

#[derive(FromForm, Debug)]
pub struct SigninDetails {
    name: String,
    password: String,
}

#[derive(FromForm, Debug)]
pub struct SignupDetails {
    class: String,
    name: String,
    email: String,
}

#[derive(FromForm, Debug)]
pub struct UserDetails {
    name: String,
    class: String,
}

#[derive(FromForm, Debug)]
pub struct ReviewDetails {
    about: String,
    about_class: String,
    content: String,
    anonymous: bool,
}

#[derive(FromForm, Debug)]
pub struct UpdateProfileDetails {
    class: String,
    old_password: String,
    new_password: String,
    email: String,
    profile_pic: String,
    private: bool,
}

#[get("/")]
fn index_route(conn: State<Mutex<Client>>, mut cookies: Cookies) -> Template {
    let user_details = get_user_details(&mut conn.lock().unwrap(), &mut cookies);
    let context = TemplateContext {
        value: None,
        user_details: Some(user_details.clone().split("|").map(str::to_owned).collect()),
        reviews: None,
    };
    if user_details == String::from("x|x|x|x|x|x") {
        return Template::render("signin", &context);
    }
    Template::render("index", &context)
}

#[get("/reviews")]
fn reviews_route(conn: State<Mutex<Client>>, mut cookies: Cookies) -> Template {
    let user_details = get_user_details(&mut conn.lock().unwrap(), &mut cookies);
    let context = TemplateContext {
        value: if user_details == String::from("x|x|x|x|x|x") {
            Some(String::from("<script>window.location.href='/'</script>"))
        } else {
            None
        },
        user_details: Some(user_details.split("|").map(str::to_owned).collect()),
        reviews: get_reviews(&mut conn.lock().unwrap(), &mut cookies),
    };
    Template::render("reviews", &context)
}

#[get("/signup")]
fn signup_route() -> Template {
    let context = TemplateContext {
        value: None,
        user_details: None,
        reviews: None,
    };
    Template::render("signup", &context)
}

#[get("/profile")]
fn profile_route(conn: State<Mutex<Client>>, mut cookies: Cookies) -> Template {
    let user_details = get_user_details(&mut conn.lock().unwrap(), &mut cookies);
    let context = TemplateContext {
        value: if user_details == String::from("x|x|x|x|x|x") {
            Some(String::from("<script>window.location.href='/'</script>"))
        } else {
            None
        },
        user_details: Some(user_details.split("|").map(str::to_owned).collect()),
        reviews: None,
    };
    Template::render("profile", &context)
}

#[post("/signin", data = "<signin_details>")]
fn signin_user_route(
    conn: State<Mutex<Client>>,
    signin_details: Form<SigninDetails>,
    cookies: Cookies,
) -> String {
    db_operations::signin_user(&mut conn.lock().unwrap(), signin_details, cookies)
}

#[post("/signup", data = "<signup_details>")]
fn signup_user_route(
    conn: State<Mutex<Client>>,
    signup_details: Form<SignupDetails>,
) -> String {
    db_operations::create_user(&mut conn.lock().unwrap(), signup_details)
}

#[post("/registeredStatus", data = "<user_details>")]
fn registered_status_route(
    conn: State<Mutex<Client>>,
    user_details: Form<UserDetails>,
    mut cookies: Cookies,
) -> String {
    db_operations::check_registered_status(&mut conn.lock().unwrap(), user_details, &mut cookies)
}

#[post("/postReview", data = "<review_details>")]
fn post_review_route(
    conn: State<Mutex<Client>>,
    review_details: Form<ReviewDetails>,
    cookies: Cookies,
) -> String {
    db_operations::post_review(&mut conn.lock().unwrap(), review_details, cookies)
}

#[post("/updateProfile", data = "<update_profile_details>")]
fn update_profile_route(
    conn: State<Mutex<Client>>,
    update_profile_details: Form<UpdateProfileDetails>,
    mut cookies: Cookies,
) -> String {
    db_operations::update_profile(&mut conn.lock().unwrap(), update_profile_details, &mut cookies)
}

#[post("/signout")]
fn signout_route(mut cookies: Cookies) -> String {
    db_operations::signout_user(&mut cookies)
}

fn configure() -> Config {
    // Configure Rocket to serve on the port requested by Heroku.
    let mut config = Config::active().expect("could not load configuration");
    config
        .set_secret_key(env::var("SECRET_KEY").expect("Env var SECRET_KEY not found"))
        .unwrap();
    if let Ok(port_str) = env::var("PORT") {
        let port = port_str.parse().expect("could not parse PORT");
        config.set_port(port);
    }
    config
}

fn rocket() -> rocket::Rocket {
    rocket::custom(configure())
        .mount(
            "/",
            routes![
                index_route,
                reviews_route,
                signup_route,
                profile_route,
                signin_user_route,
                signup_user_route,
                registered_status_route,
                post_review_route,
                update_profile_route,
                signout_route,
            ],
        )
        .mount("/styles", StaticFiles::from("static/styles"))
        .mount("/scripts", StaticFiles::from("static/scripts"))
        .mount("/fonts", StaticFiles::from("static/fonts"))
        .mount("/images", StaticFiles::from("static/images"))
        .attach(Template::fairing())
}

fn main() {
    dotenv().ok();
    let client = Client::connect(&env::var("DATABASE_URL").expect("Env var DATABASE_URL not found"), NoTls).unwrap();
    rocket().manage(Mutex::new(client)).launch();
}
