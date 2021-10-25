use std::sync::Mutex;
use tokio::sync::mpsc;

use actix_web::{
    middleware::Logger,
    web::{self, Json},
    App, HttpResponse, HttpServer, Responder,
};
use backend::MediaSource;
use env_logger;
use errors::HomeRadioError;
use log::{error, info};

use crate::{
    backend::{Backend, FileBackend},
    media_service::{MediaService, RemoteMediaService, VLCMediaServiceFactory},
};
mod backend;
mod errors;
mod media_service;

const INDEX_HTML: &str = include_str!("./ui/index.html");
const FORM_HTML: &str = include_str!("./ui/add-media-form.html");
const INDEX_JS: &str = include_str!("./ui/index.js");
const INDEX_CSS: &str = include_str!("./ui/index.css");
const COMMON_JS: &str = include_str!("./ui/common.js");
const FORM_JS: &str = include_str!("./ui/form.js");
const FAVICON: &[u8] = include_bytes!("./ui/favicon.ico");
const ANDROID_FAVICON: &[u8] = include_bytes!("./ui/android-chrome-192x192.png");

#[actix_web::main]
async fn main() -> Result<(), HomeRadioError> {
    let media_file_path = std::env::args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "/var/lib/home-radio".into());
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let fb = FileBackend::new(media_file_path).await?;
    let vol = fb.get_volume().await?;
    // media_service::start_media_thread(rcv, VLCMediaServiceFactory {}, vol);
    let backend = web::Data::new(Mutex::new(Box::new(fb) as Box<dyn Backend + Send>));

    HttpServer::new(move || {
        let srvc = RemoteMediaService::new_with_auth(
            "localhost".into(),
            "8090".into(),
            "".into(),
            "foo".into(),
        );

        App::new()
            //.wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::Data::new(srvc))
            .app_data(backend.clone())
            // ui routes
            .route("/", web::get().to(index_html))
            .route("index.css", web::get().to(index_css))
            .route("index.html", web::get().to(index_html))
            .route("index.js", web::get().to(index_js))
            .route("common.js", web::get().to(common_js))
            .route("form.js", web::get().to(form_js))
            .route("add-media-form.html", web::get().to(media_form_html))
            .route("favicon.ico", web::get().to(favicon))
            .route("android-chrome-192x192.png", web::get().to(android_favicon))
            // resource routes
            .route("/media", web::get().to(get_media_sources))
            .route("/media", web::put().to(add_media_source))
            // media control routes
            .route("/start", web::post().to(start_playback))
            .route("/stop", web::post().to(stop_playback))
            .route("/volume", web::get().to(get_current_volume))
            .route("/volume", web::put().to(set_current_volume))
            .route("/increase_volume", web::post().to(increase_volume))
            .route("/decrease_volume", web::post().to(decrease_volume))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;
    Ok(())
}

async fn add_media_source(
    body: Json<MediaSource>,
    backend: web::Data<Mutex<Box<dyn Backend + Send>>>,
) -> impl Responder {
    let result = { backend.lock().unwrap().add_media_source(body.0).await };
    if let Err(e) = result {
        error!("{}", e);
        return HttpResponse::InternalServerError().into();
    }
    HttpResponse::Ok()
}

async fn get_media_sources(backend: web::Data<Mutex<Box<dyn Backend + Send>>>) -> impl Responder {
    let media_sources = { backend.lock().unwrap().get_media_sources().await };
    let media_sources = if let Err(e) = media_sources {
        error!("{}", e);
        return HttpResponse::InternalServerError().into();
    } else {
        media_sources.unwrap()
    };

    HttpResponse::Ok().json(media_sources)
}

async fn index_css() -> impl Responder {
    HttpResponse::Ok().content_type("text/css").body(INDEX_CSS)
}

async fn favicon() -> impl Responder {
    HttpResponse::Ok().body(FAVICON)
}
async fn android_favicon() -> impl Responder {
    HttpResponse::Ok().body(ANDROID_FAVICON)
}
async fn media_form_html() -> impl Responder {
    HttpResponse::Ok().content_type("text/html").body(FORM_HTML)
}

async fn index_html() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(INDEX_HTML)
}

async fn common_js() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(COMMON_JS)
}
async fn form_js() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(FORM_JS)
}

async fn index_js() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(INDEX_JS)
}

async fn set_current_volume(
    backend: web::Data<Mutex<Box<dyn Backend + Send>>>,
    srvc: web::Data<RemoteMediaService>,
    body: String,
) -> impl Responder {
    let result = body.parse::<u16>();
    let amount = if let Ok(amount) = result {
        amount
    } else {
        return HttpResponse::BadRequest().body("invalid volume");
    };

    let result = { backend.lock().unwrap().set_volume(amount).await };
    info!("default volume set to {}", amount);
    if let Err(e) = result {
        error!("{}", e);
        return HttpResponse::InternalServerError().into();
    }
    let result = srvc.set_volume(amount).await;

    if let Err(e) = result {
        error!("{}", e);
        HttpResponse::InternalServerError().into()
    } else {
        HttpResponse::Ok().into()
    }
}

async fn start_playback(
    backend: web::Data<Mutex<Box<dyn Backend + Send>>>,
    srvc: web::Data<RemoteMediaService>,
    body: String,
) -> impl Responder {
    info!("starting playback of {}", &body);
    let vol = { backend.lock().unwrap().get_volume().await };
    let vol = if let Err(e) = vol {
        error!("error getting current volume: {}", e);
        return HttpResponse::InternalServerError().into();
    } else {
        vol.unwrap()
    };

    let result = srvc.play(&body, vol).await;

    if let Err(e) = result {
        error!("error starting playback of url {}: {}", &body, e);
        HttpResponse::InternalServerError().body(e.to_string())
    } else {
        HttpResponse::Ok().into()
    }
}

async fn stop_playback(srvc: web::Data<RemoteMediaService>) -> impl Responder {
    let result = srvc.stop().await;
    if let Err(e) = result {
        error!("error stopping playback");
        HttpResponse::InternalServerError().body(e.to_string())
    } else {
        HttpResponse::Ok().into()
    }
}

async fn get_current_volume(backend: web::Data<Mutex<Box<dyn Backend + Send>>>) -> impl Responder {
    let result = { backend.lock().unwrap().get_volume().await };
    match result {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(vol) => HttpResponse::Ok()
            .content_type("text/plain")
            .body(vol.to_string()),
    }
}

async fn increase_volume(body: String, srvc: web::Data<Box<dyn MediaService>>) -> impl Responder {
    let result = body.parse::<u16>();
    let amount = if let Ok(amount) = result {
        amount
    } else {
        return HttpResponse::BadRequest().body("invalid amount");
    };
    let result = srvc.increase_volume(amount as i32);

    if let Err(e) = result {
        HttpResponse::InternalServerError().body(e.to_string())
    } else {
        HttpResponse::Ok().into()
    }
}

async fn decrease_volume(body: String, srvc: web::Data<Box<dyn MediaService>>) -> impl Responder {
    let result = body.parse::<u16>();
    let amount = if let Ok(amount) = result {
        amount
    } else {
        return HttpResponse::BadRequest().body("invalid amount");
    };
    let result = srvc.decrease_volume(amount as i32);

    if let Err(e) = result {
        HttpResponse::InternalServerError().body(e.to_string())
    } else {
        HttpResponse::Ok().into()
    }
}
