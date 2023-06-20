use mailfred::{
    self,
    service::{
        response::{Response, ResponseResult},
        Request,
    },
    transport::Connector,
    transports::{Gmail, Imap},
    util::logger,
};

async fn echo(req: Request, _state: ()) -> ResponseResult {
    Response::ok(req.header, req.body)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    logger::configure(log::LevelFilter::Trace);

    let (imap, smtp) = Gmail {
        username: "user".into(),
        password: "1234".into(),
    }
    .split();

    // Modify the default imap folder to use the sent folder instead.
    // Each email server can have each own name for this.
    let clean_sent_imap = Imap {
        folder: "[Gmail]/Sent".into(),
        ..imap.clone()
    };

    // Create a imap connection to consume all messages from that folder
    mailfred::init_consumer_task(clean_sent_imap, "sent").await?;

    mailfred::serve((imap, smtp), (), echo).await
}
