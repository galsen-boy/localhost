use async_std::path::PathBuf;

use http::{Response, Request, StatusCode};

use crate::server::core::ServerConfig;

/// Dernier point : si la réponse d'erreur personnalisée 500 échoue
fn hardcoded_response_500(
  cookie_value: String,
) -> Response<Vec<u8>>{

  let body = "Hardcoded status 500. Internal Server Error. Custom error 500 response failed. \n\n".as_bytes().to_vec();
  
  let response = match Response::builder()
  .status(StatusCode::INTERNAL_SERVER_ERROR)
  .header("Content-Type", "text/plain")
  .header("Set-Cookie", cookie_value)
  .body(body)
  {
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to create hardcoded 500 response: {}", e);
      return Response::new("Fatal Internal Server Error.\nFailed to create hardcoded error 500 response.\nStatus 500 does not set properly".as_bytes().to_vec());
    }
  };
  
  response

}

/// Retourner une réponse d'erreur personnalisée 500.
/// En cas d'erreur, retourner une réponse codée en dur avec le code de statut 500
pub async fn custom_response_500(
  _request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: &PathBuf,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{

  let error_page_path = zero_path_buf
  .join("static")
  .join(server_config.error_pages_prefix.clone())
  .join("500.html");
  
// Lire la page d'erreur. En cas d'erreur, retourner une réponse codée en dur avec le code de statut 500
let error_page_content = match std::fs::read(error_page_path){
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to read error page: {}", e);
      return hardcoded_response_500(cookie_value)
    }
  };

  let response = match Response::builder()
  .status(StatusCode::INTERNAL_SERVER_ERROR)
  .header("Content-Type", "text/html")
  .header("Set-Cookie", cookie_value.clone())
  .body(error_page_content)
  {
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to create custom 500 response: {}", e);
      return hardcoded_response_500(cookie_value.clone())
    }
  };

  response
}
