use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::{options::ClientOptions, Client};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct BlogPost {
    id: Option<String>,
    title: String,
    content: String,
}

async fn get_all_posts(db: web::Data<Client>) -> impl Responder {
    let db = db.database("CouponWorld");

    let collection = db.collection::<BlogPost>("coupon");

    if let Ok(mut cursor) = collection.find(None, None).await {
        let mut posts = Vec::new();
        while let Some(post) = cursor.with_type(){
            if let Ok(post) = post {
                posts.push(post);
            }
        }
        return HttpResponse::Ok().json(posts);
    }

    HttpResponse::InternalServerError().finish()

}

async fn get_post_by_id(db: web::Data<Client>, path: web::Path<String>) -> impl Responder {
    let db = db.database("blog");
    let collection = db.collection::<BlogPost>("posts");

    if let Ok(post) = collection.find_one(doc! { "_id": ObjectId::with_string(&path).unwrap() }, None).await {
        if let Some(post) = post {
            return HttpResponse::Ok().json(post);
        }
    }

    HttpResponse::NotFound().finish()
}

async fn create_post(db: web::Data<Client>, post: web::Json<BlogPost>) -> impl Responder {
    let db = db.database("blog");
    let collection = db.collection::<BlogPost>("posts");

    if let Ok(result) = collection.insert_one(post.into_inner(), None).await {
        if let Some(id) = result.inserted_id.as_object_id() {
            let response = doc! { "id": id.to_hex() };
            return HttpResponse::Ok().json(response);
        }
    }

    HttpResponse::InternalServerError().finish()
}

async fn update_post(db: web::Data<Client>, path: web::Path<String>, post: web::Json<BlogPost>) -> impl Responder {
    let db = db.database("blog");
    let collection = db.collection::<BlogPost>("posts");

    if let Ok(_) = collection
        .update_one(doc! { "_id": ObjectId::with_string(&path).unwrap() }, doc! { "$set": post.into_inner() }, None)
        .await
    {
        return HttpResponse::Ok().finish();
    }

    HttpResponse::NotFound().finish()
}

async fn delete_post(db: web::Data<Client>, path: web::Path<String>) -> impl Responder {
    let db = db.database("blog");
    let collection = db.collection::<BlogPost>("posts");

    if let Ok(_result) = collection.delete_one(doc! { "_id": ObjectId::with_string(&path).unwrap() }, None).await {
        return HttpResponse::Ok().finish();
    }

    HttpResponse::NotFound().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up MongoDB client
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
    let client = Client::with_options(client_options).unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(client.clone())
            .route("/posts", web::get().to(get_all_posts))
            .route("/posts/{id}", web::get().to(get_post_by_id))
            .route("/posts", web::post().to(create_post))
            .route("/posts/{id}", web::put().to(update_post))
            .route("/posts/{id}", web::delete().to(delete_post))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
