use std::sync::Arc;

use mailfred::{
    self, logger,
    response::{Response, ResponseResult},
    service::Request,
    transports::{Imap, Smtp},
};
use tokio::sync::Mutex;

#[derive(Default)]
struct MyState {
    // User data shared accross all calls
}

type State = Arc<Mutex<MyState>>;

async fn echo(req: Request, _state: State) -> ResponseResult {
    Response::ok(req.header, req.body)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::configure(log::LevelFilter::Trace);

    let imap = Imap {
        domain: "imap.gmail.com".into(),
        port: 993,
        user: "user@gmail.com".into(),
        password: "1234".into(),
        folder: "inbox".into(),
    };

    let smtp = Smtp {
        domain: "smtp.gmail.com".into(),
        port: 587,
        user: "user@gmail.com".into(),
        password: "1234".into(),
    };

    mailfred::serve((imap, smtp), State::default(), echo).await
}
