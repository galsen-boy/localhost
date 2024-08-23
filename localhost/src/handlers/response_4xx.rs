use async_std::path::PathBuf;

use http::{Request, Response, StatusCode};

use crate::server::core::ServerConfig;
use crate::handlers::response_500::custom_response_500;

const ALLOWED_4XX_STATUS_CODES: [StatusCode; 5] = [
  StatusCode::BAD_REQUEST,
  StatusCode::FORBIDDEN,
  StatusCode::NOT_FOUND, // Géré à l'intérieur de handle_all
  StatusCode::METHOD_NOT_ALLOWED, // Géré à l'intérieur de handle_all
  StatusCode::PAYLOAD_TOO_LARGE,
];
/// Retourner une réponse d'erreur 4xx personnalisée.
/// Selon la tâche, les erreurs 4xx personnalisées suivantes doivent être gérées :
/// 400, 403, 404, 405, 413
/// En cas d'erreur, retourner `custom_response_500`
pub async fn custom_response_4xx(
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: &PathBuf,
  server_config: ServerConfig,
  status_code: StatusCode,
) -> Response<Vec<u8>>{

  // Vérifier si le code de statut est dans la liste 4xx : 400, 403, 404, 405, 413
  if !ALLOWED_4XX_STATUS_CODES.contains(&status_code){
    // eprintln!("ERROR: Internal Server Error\ncustom_response_4xx: status code {:?}\nis not in 4xx list {:?}", status_code, ALLOWED_4XX_STATUS_CODES);
    return custom_response_500(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
    ).await
  }

  let error_page_path = zero_path_buf
  .join("static")
  .join(server_config.error_pages_prefix.clone())
  .join(status_code.as_str().to_string() + ".html");
  
// Lire la page d'erreur. En cas d'erreur, retourner `custom_response_500`
let error_page_content = match std::fs::read(error_page_path){
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to read error page: {}", e);
      return custom_response_500(
        request,
        cookie_value,
        zero_path_buf,
        server_config,
      ).await
    }
  };

  let response = match Response::builder()
  .status(status_code)
  .header("Content-Type", "text/html")
  .header("Set-Cookie", cookie_value.clone())
  .body(error_page_content)
  {
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to create custom 4xx response: {}", e);
      return custom_response_500(
        request,
        cookie_value.clone(),
        zero_path_buf,
        server_config,
      ).await
    }
  };
  
  response

}