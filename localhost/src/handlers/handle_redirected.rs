use async_std::path::PathBuf;
use http::{Request, Response, response::Builder, StatusCode, header};

use crate::server::core::ServerConfig;

pub async fn handle_redirected(
  _request: &Request<Vec<u8>>,
  _cookie_value:String,
  _zero_path_buf: &PathBuf,
  _server_config: ServerConfig,
) -> Response<Vec<u8>>{
  let response = Builder::new()
  .status(StatusCode::SEE_OTHER)
  .header(header::LOCATION, "/uploads")
  .body(
    "<!DOCTYPE html>
    <html>
    <head>
    <meta http-equiv=\"refresh\" content=\"4; url=/uploads\" />
    </head>
    <body>
    <script>
    setTimeout(function(){
      window.location.href = '/uploads';
    }, 2000);
    </script>
    </body>
    </html>".into()
  )
  .unwrap();
  
  response
  
}