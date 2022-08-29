extern crate bcrypt;
extern crate chrono;
extern crate rand;
extern crate rand_hc;

use self::{
    bcrypt::{hash, verify, DEFAULT_COST},
    chrono::NaiveDateTime,
    rand::{FromEntropy, Rng},
    rand_hc::Hc128Rng,
};
use super::{
    Review, ReviewDetails, SigninDetails, SignupDetails, UpdateProfileDetails, UserDetails,
};
use postgres::Client;
use rocket::{
    http::{Cookie, Cookies},
    request::Form,
};
use std::env;

// Function to check if the name is available
pub fn name_available(conn: &mut Client, name: &str) -> bool {
    if name.len() == 0 {
        // The name has length 0
        return false;
    }
    // Return whether there are no rows with the given name
    conn.query(&format!("SELECT * FROM users WHERE name = '{}'", name), &[])
        .unwrap()
        .is_empty()
}

/*
CREATE TABLE users (
id serial PRIMARY KEY,
name VARCHAR (50) UNIQUE NOT NULL,
class VARCHAR (4) NOT NULL,
password VARCHAR (62) NOT NULL,
salt BIGINT NOT NULL,
email VARCHAR (200),
profile_pic VARCHAR NOT NULL,
private BOOL NOT NULL
)
*/

// Function to create a user with the given details if they're valid
pub fn create_user(conn: &mut Client, user_details: Form<SignupDetails>) -> String {
    if user_details.name.len() > 3 {
        // Proceed if name is valid
        if user_details.class.len() == 0 {
            return String::from("Error: No class specified");
        }
        if name_available(conn, &user_details.name) == false {
            return String::from("Error: Name already registered");
        }
        // Generate salt using a CSPRNG
        let salt = Hc128Rng::from_entropy().gen::<u32>();
        let password_hash = hash(
                &format!(
                    "{}{}",
                    salt,
                    env::var("DEFAULT_PASSWORD").expect("Env var DEFAULT_PASSWORD not found")
                ),
                DEFAULT_COST
            ).unwrap();
        println!("{}", password_hash);
        if let Err(e) = conn.query(&format!("INSERT INTO users VALUES(
            DEFAULT, $1, $2, $3, {}, $4,
            'https://cdn.discordapp.com/attachments/528262411392909343/548507683037118465/IMG_20190222_194150.jpg',
            false
        )", salt), &[&user_details.name, &user_details.class, &password_hash, &user_details.email]) {
            println!("Error: {}", e.to_string());
            return String::from(e.to_string());
        };
        return String::from("Success");
    }
    String::from("Error: Name has to be more than 3 characters long")
}

// Function to check if specified credentials match
pub fn signin_user(
    conn: &mut Client,
    user_details: Form<SigninDetails>,
    mut cookies: Cookies,
) -> String {
    if user_details.name.len() > 0 && user_details.password.len() > 0 {
        // Proceed if name and password have been provided
        let rows = conn
            .query(
                "SELECT password, salt FROM users WHERE name = $1",
                &[&user_details.name],
            )
            .unwrap();
        if rows.is_empty() == false {
            // Compare passwords if name exists
            let password_hash: String = rows.get(0).unwrap().get(0);
            let salt: i64 = rows.get(0).unwrap().get(1);
            if verify(
                &format!("{}{}", salt, user_details.password),
                &password_hash.trim(),
            )
            .unwrap()
            {
                // If the credentials are correct, create the cookies to sign in the user
                cookies.add_private(Cookie::new("name", (&user_details.name).clone()));
                cookies.add_private(Cookie::new("hash", String::from(password_hash.trim())));
                return String::from("Success");
            }
        } else {
            // Hash something even if name doesn't exist, so that response times are similar
            hash("foo", DEFAULT_COST).unwrap();
        }
    }
    String::from("Wrong credentials")
}

// Function to get user's details if user is signed in
pub fn get_user_details(conn: &mut Client, cookies: &mut Cookies) -> String {
    if let Some(name) = cookies.get_private("name") {
        // If the name cookie exists
        if let Some(password_hash) = cookies.get_private("hash") {
            // If the password hash cookie exists as well
            let rows = conn
                .query("SELECT * FROM users WHERE name = $1", &[&name.value()])
                .unwrap();
            // Read all the values to variables
            let id: i32 = rows.get(0).unwrap().get(0);
            let name: String = rows.get(0).unwrap().get(1);
            let class: String = rows.get(0).unwrap().get(2);
            let password: String = rows.get(0).unwrap().get(3);
            let email: String = rows.get(0).unwrap().get(5);
            let profile_pic: String = rows.get(0).unwrap().get(6);
            let private: bool = rows.get(0).unwrap().get(7);

            if password_hash.value() == password.trim() {
                // Return the details separated with '|'
                return format!(
                    "{}|{}|{}|{}|{}|{}",
                    id, name, class, email, profile_pic, private
                );
            }
        }
    }
    String::from("x|x|x|x|x|x")
}

pub fn check_registered_status(
    conn: &mut Client,
    user_details: Form<UserDetails>,
    cookies: &mut Cookies,
) -> String {
    if let Some(name) = cookies.get_private("name") {
        // If the name cookie exists
        if let Some(password_hash) = cookies.get_private("hash") {
            // If the password hash cookie exists as well
            let rows = conn
                .query("SELECT * FROM users WHERE name = $1", &[&name.value()])
                .unwrap();
            let password: String = rows.get(0).unwrap().get(3);

            if password_hash.value() == password.trim() {
                // The user is signed in
                if &user_details.name == name.value() {
                    // The user is checking their own registered status
                    return String::from("Self");
                }
                let complying_rows = conn
                    .query(
                        "SELECT * FROM users WHERE name = $1 AND class = $2",
                        &[&user_details.name, &user_details.class],
                    )
                    .unwrap();
                if complying_rows.is_empty() {
                    // No user has the specified details
                    return String::from("Not registered");
                }
                // A user has the specified details
                return String::from("Registered");
            }
        }
    }
    String::from("Signed out")
}

/*
CREATE TABLE reviews (
id serial PRIMARY KEY,
about VARCHAR (50) NOT NULL,
about_class VARCHAR (4) NOT NULL,
time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
content VARCHAR (4000) NOT NULL,
by_user VARCHAR (50) NOT NULL
)
*/

pub fn post_review(
    conn: &mut Client,
    review_details: Form<ReviewDetails>,
    mut cookies: Cookies,
) -> String {
    if let Some(name) = cookies.get_private("name") {
        // If the name cookie exists
        if let Some(password_hash) = cookies.get_private("hash") {
            // If the password hash cookie exists as well
            let rows = conn
                .query("SELECT * FROM users WHERE name = $1", &[&name.value()])
                .unwrap();
            let password: String = rows.get(0).unwrap().get(3);

            if password_hash.value() == password.trim() {
                // The user is signed in
                let by_user = if review_details.anonymous {
                    "no"
                } else {
                    &name.value()
                };
                if let Err(e) = conn.query(
                    "INSERT INTO reviews VALUES(
                    DEFAULT, $1, $2, DEFAULT, $3, $4
                )",
                    &[
                        &review_details.about,
                        &review_details.about_class,
                        &review_details.content,
                        &by_user,
                    ],
                ) {
                    println!("Error: {}", e.to_string());
                    return String::from(e.to_string());
                };
                return String::from("Success");
            }
        }
    }
    String::from("Signed out")
}

pub fn get_reviews(conn: &mut Client, cookies: &mut Cookies) -> Option<Vec<Review>> {
    if let Some(name) = cookies.get_private("name") {
        // If the name cookie exists
        if let Some(password_hash) = cookies.get_private("hash") {
            // If the password hash cookie exists as well
            let rows = conn
                .query("SELECT * FROM users WHERE name = $1", &[&name.value()])
                .unwrap();
            let password: String = rows.get(0)?.get(3);

            if password_hash.value() == password.trim() {
                // The user is signed in
                // Get all the reviews
                let mut reviews = vec![];
                for row in &conn
                    .query("SELECT * FROM reviews ORDER BY id DESC", &[])
                    .unwrap()
                {
                    let about: String = row.get(1);
                    if &about != &name.value() {
                        // Check if the review is about a person with a private profile or not
                        let about_user = conn
                            .query(
                                "SELECT * FROM users WHERE name = $1 AND class = $2",
                                &[&about, &row.get::<_, String>(2)],
                            )
                            .unwrap();
                        if !about_user.is_empty() {
                            if about_user.get(0)?.get(7) {
                                // The review is about a person with a private profile
                                continue;
                            }
                        }
                    }
                    let by_user = conn
                        .query(
                            "SELECT * FROM users WHERE name = $1",
                            &[&row.get::<_, String>(5)],
                        )
                        .unwrap();
                    let review = Review {
                        about,
                        about_class: row.get(2),
                        time: row.get::<_, NaiveDateTime>(3).to_string(),
                        content: row.get(4),
                        by_user: if by_user.is_empty() {
                            // This could happen if it is an anonymous review
                            [
                                String::new(),
                                String::from("no"),
                                String::new(),
                                String::new(),
                                String::new(),
                            ]
                        } else {
                            let by_user = by_user.get(0)?;
                            [
                                by_user.get::<_, i32>(0).to_string(),
                                by_user.get(1),
                                by_user.get(2),
                                by_user.get(5),
                                by_user.get(6),
                            ]
                        },
                    };
                    reviews.push(review);
                }
                return Some(reviews);
            }
        }
    }
    None
}

pub fn update_profile(
    conn: &mut Client,
    update_profile_details: Form<UpdateProfileDetails>,
    cookies: &mut Cookies,
) -> String {
    if let Some(name) = cookies.get_private("name") {
        // If the name cookie exists
        if let Some(password_hash) = cookies.get_private("hash") {
            // If the password hash cookie exists as well
            let rows = conn
                .query("SELECT * FROM users WHERE name = $1", &[&name.value()])
                .unwrap();
            let password: String = rows.get(0).unwrap().get(3);

            if password_hash.value() == password.trim() {
                // The user is signed in
                let password_hash: String = if update_profile_details.new_password == "" {
                    rows.get(0).unwrap().get(3)
                } else {
                    let salt: i64 = rows.get(0).unwrap().get(4);
                    if !verify(
                        &format!("{}{}", salt, update_profile_details.old_password),
                        password.trim(),
                    )
                    .unwrap()
                    {
                        // The user has provided their old password wrongly
                        return String::from("Error: Wrong old password");
                    }
                    if update_profile_details.new_password.len() <= 4 {
                        return String::from("Error: Password is too short");
                    }
                    let new_password_hash = hash(
                        &format!("{}{}", salt, update_profile_details.new_password),
                        DEFAULT_COST,
                    )
                    .unwrap();
                    println!("{}", new_password_hash);
                    new_password_hash
                };
                match conn.query(
                    "UPDATE users SET class = $1,\
                     password = $2, email = $3, profile_pic = $4, private = $5 WHERE id = $6",
                    &[
                        &update_profile_details.class,
                        &password_hash,
                        &update_profile_details.email,
                        &update_profile_details.profile_pic,
                        &update_profile_details.private,
                        &rows.get(0).unwrap().get::<_, i32>(0),
                    ],
                ) {
                    Ok(_) => {
                        if update_profile_details.new_password != "" {
                            // The user has changed their password, so sign them in
                            cookies.add_private(Cookie::new("hash", password_hash));
                        }
                        return String::from("Success");
                    }
                    Err(e) => {
                        return e.to_string();
                    }
                }
            }
        }
    }
    String::from("Signed out")
}

pub fn signout_user(cookies: &mut Cookies) -> String {
    cookies.remove_private(Cookie::named("name"));
    cookies.remove_private(Cookie::named("hash"));
    String::from("Success")
}
