use std::{path::Path, sync::Mutex, process::Child};

use actix_web::{
    web::{self, Json},
    App, HttpResponse, HttpServer, Responder,
};
use backend::MediaSource;
use env_logger;
use errors::HomeRadioError;
use log::{error, info};
use media_service::RemoteMediaService;

use crate::{
    backend::FileBackend,
};
mod backend;
mod cli;
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
    let app = cli::build_app();
    let matches = app.get_matches();

    match matches.subcommand() {
        ("serve", Some(args)) => {
            let dir = args.value_of("dir").unwrap();
            let autoplay = args.is_present("autoplay");
            serve(dir, autoplay).await?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

struct ProcessCleaner {
    inner: Child
}

impl Drop for ProcessCleaner {
    fn drop(&mut self) {
        let _err = self.inner.kill();
    }
}

async fn serve<A: AsRef<Path>>(dir: A, autoplay: bool) -> Result<(), HomeRadioError> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let fb = FileBackend::new(dir.as_ref()).await?;

    let vlc_process = std::process::Command::new("/usr/bin/vlc")
        .arg("-I")
        .arg("http")
        .arg("--http-port")
        .arg("8090")
        .arg("--http-password")
        .arg("foo")
        .spawn()?;
    
    // kill the vlc process when this goes out of scope
    let _cleaner = ProcessCleaner{inner: vlc_process};
    let srvc = RemoteMediaService::new_with_auth("localhost".into(), "8090".into(), "foo".into());
    if autoplay {
        let current_src = fb.get_current_media_source().await?;
        srvc.wait_for_healthy(20, 200).await?;
        if let Some(current) = current_src {
            let vol = fb.get_volume().await?;

            srvc.play(&current, vol).await?;
        } else {
            let sources = fb.get_media_sources().await?;
            let default_source = sources.iter().find(|src| src.default_source);
            if let Some(src) = default_source {
                let vol = fb.get_volume().await?;
                srvc.play(&src.link, vol).await?;
            }
        }
    }

    let backend = web::Data::new(Mutex::new(fb));

    HttpServer::new(move || {
        let srvc =
            RemoteMediaService::new_with_auth("localhost".into(), "8090".into(), "foo".into());

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
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;
    Ok(())
}

async fn add_media_source(
    body: Json<MediaSource>,
    backend: web::Data<Mutex<FileBackend>>,
) -> impl Responder {
    let result = { backend.lock().unwrap().add_media_source(body.0).await };
    if let Err(e) = result {
        error!("{}", e);
        return HttpResponse::InternalServerError().into();
    }
    HttpResponse::Ok()
}

async fn get_media_sources(backend: web::Data<Mutex<FileBackend>>) -> impl Responder {
    let result = {
        let backend = backend.lock().unwrap();
        (
            backend.get_media_sources().await,
            backend.get_current_media_source().await,
        )
    };

    let (mut media_sources, current_source) = match result {
        (_, Err(e)) | (Err(e), _) => {
            error!("{}", e);
            return HttpResponse::InternalServerError().into();
        }
        (Ok(media_sources), Ok(current_source)) => (media_sources, current_source),
    };

    if let Some(current_source) = current_source {
        for src in media_sources.iter_mut() {
            if current_source == src.link {
                src.currently_playing = Some(true)
            } else {
                src.currently_playing = None;
            }
        }
    }

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
    backend: web::Data<Mutex<FileBackend>>,
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
    backend: web::Data<Mutex<FileBackend>>,
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
        let result = {
            backend
                .lock()
                .unwrap()
                .set_current_media_source(&body)
                .await
        };
        if let Err(e) = result {
            error!("{}", e);
        }
        HttpResponse::Ok().into()
    }
}

async fn stop_playback(
    srvc: web::Data<RemoteMediaService>,
    backend: web::Data<Mutex<FileBackend>>,
) -> impl Responder {
    let result = { backend.lock().unwrap().remove_current_media_source().await };
    if let Err(e) = result {
        error!("error removing current playback source: {}", e);
        return HttpResponse::InternalServerError();
    }

    let result = srvc.stop().await;
    if let Err(e) = result {
        error!("error stopping playback: {}", e);
        HttpResponse::InternalServerError()
    } else {
        HttpResponse::Ok()
    }
}

async fn get_current_volume(backend: web::Data<Mutex<FileBackend>>) -> impl Responder {
    let result = { backend.lock().unwrap().get_volume().await };
    match result {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(vol) => HttpResponse::Ok()
            .content_type("text/plain")
            .body(vol.to_string()),
    }
}