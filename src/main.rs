use env_logger::fmt::style::AnsiColor::Cyan;
use log::{error, info};
use std::net::{TcpListener, TcpStream};

use crate::{
    constants::server,
    http::{
        Context::HTTPContext,
        Error::{Flow, HTTPError},
        Router::HTTPRouter,
    },
    utile::files::is_in_public,
};

mod constants;
mod http;
mod utile;

// pub type Middleware = fn(&mut HTTPContext<'_>) -> Flow;

fn logging_middleware(ctx: &mut HTTPContext<'_>) {
    let peer = ctx.get_peer();
    match peer {
        Ok(peer) => {
            info!("the client ip is {}", peer.ip());
            ctx.Next();
        }
        Err(_err) => {
            info!("unknown client {}", _err);
            ctx.Error(_err);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_line_number(true)
        .format_module_path(true)
        .format_file(true)
        .format_timestamp_millis()
        .init();

    let mut router = HTTPRouter::new();
    router.add_middlewares(&[logging_middleware]);
    router.GET("/health", health, &[]);
    router.GET("/hello/:name", hello, &[]);
    router.GET("/users/:id", user, &[]);
    router.POST("/echo", echo, &[]);
    router.compile();

    // leak it once so every thread can hold a &'static HTTPRouter
    let router: &'static HTTPRouter = Box::leak(Box::new(router));

    let listener = TcpListener::bind(server::ADDRESS)?;
    info!("listening on http://{}", server::ADDRESS);

    for stream in listener.incoming() {
        let stream = stream?;
        std::thread::spawn(move || {
            if let Err(err) = handle(stream, router) {
                error!("request failed: {}", err);
            }
        });
    }

    Ok(())
}

fn handle(stream: TcpStream, router: &'static HTTPRouter) -> Result<(), HTTPError> {
    // `new` already writes a 400 response if the request can't be parsed
    let mut ctx = HTTPContext::new(stream)?;
    info!("{} {}", ctx.get_method()?, ctx.get_url()?);

    if ctx.route(router) {
        // `run` writes the response (200, or 404/500)
        return ctx.run();
    } else {
        ctx.Error(http_err!(NotFound, "the route do not exist"));
    }
    Ok(())
}

fn health(ctx: &mut HTTPContext<'_>) -> Result<(), HTTPError> {
    ctx.json(&serde_json::json!({ "status": "ok" }))
}

fn hello(ctx: &mut HTTPContext<'_>) -> Result<(), HTTPError> {
    let name = ctx
        .get_param("name")
        .cloned()
        .unwrap_or_else(|| "stranger".to_string());
    ctx.text(&format!("Hello, {name}!"));
    Ok(())
}

fn user(ctx: &mut HTTPContext<'_>) -> Result<(), HTTPError> {
    let id = ctx
        .get_param("id")
        .cloned()
        .unwrap_or_else(|| "0".to_string());
    ctx.json(&serde_json::json!({ "id": id }))
}

fn echo(ctx: &mut HTTPContext<'_>) -> Result<(), HTTPError> {
    let body = String::from_utf8_lossy(ctx.get_body()).to_string();
    ctx.text(&body);
    Ok(())
}

fn index(ctx: &mut HTTPContext<'_>) -> Result<(), HTTPError> {
    match is_in_public("index.html") {
        Err(err) => ctx.Error(err),
        Ok(Exist) => match Exist {
            Some(path) => ctx.send_file(
                path.to_str()
                    .ok_or_else(|| http_err!(InternalServerError, "path is not valid UTF-8"))?,
            )?,
            None => ctx.Error(http_err!(NotFound, "file not found")),
        },
    }
    Ok(())
}
