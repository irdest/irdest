use crate::Router;
use tide::{Request, Response};

pub mod v1;

async fn draw_base(req: Request<Router>) -> tide::Result {
    Ok(Response::builder(200)
        .header("content-type", "text/html;charset=utf-8")
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <title>ratmand management UI</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
  </head>
  <body>
    <h2>Currently known addresses</h2>
    <p>ratmand keeps track of any address it has encountered on the network before.  Following is a list of them.  Further diagnostics tools will follow!</p>
    <ul>
    {}
    </ul>
  </body>
</html>"#,
            req.state()
                .known_addresses()
                .await
                .into_iter()
                .map(|(addr, local)| format!("<li><pre>{}{}</pre></li>", addr, if local { " (local)" } else { "" } ))
                .collect::<Vec<String>>()
                .join("\n")
        ))
        .build())
}

pub async fn start(router: Router, bind: &str, port: u16) -> tide::Result<()> {
    // Create a new application with state
    let mut app = tide::with_state(router);

    // Attach some routes to it
    app.at("/").get(draw_base);
    app.at("/api/v1/addrs").get(v1::get_addrs);

    // Then asynchronously run the web server
    let fut = app.listen(format!("{}:{}", bind, port));
    async_std::task::spawn(async move {
        match fut.await {
            Ok(_) => {}
            Err(e) => error!(
                "An error was encountered while running ratmand webui: {:?}",
                e
            ),
        }
    });

    Ok(())
}
