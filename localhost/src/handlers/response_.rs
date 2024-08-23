use async_std::path::PathBuf;

use http::{Response, Request, StatusCode};

use crate::files::check::ERROR_PAGES;

use crate::handlers::response_500::custom_response_500;
use crate::handlers::response_4xx::custom_response_4xx;

use crate::server::core::ServerConfig;

use crate::stream::errors::{CUSTOM_ERRORS_400, CUSTOM_ERRORS_413};
use crate::stream::errors::{CUSTOM_ERRORS_500, ERROR_200_OK};


/// Créer une réponse avec un fichier statique, selon la configuration du serveur
pub async fn response_default_static_file(
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: &PathBuf,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{
  let default_file_path = zero_path_buf
  .join("static")
  .join(server_config.static_files_prefix.clone())
  .join(server_config.default_file.clone());
  
  // Lire le fichier par défaut. En cas d'erreur, retourner une réponse d'erreur avec le code de statut 500,
  // car avant le démarrage du serveur, tous les fichiers sont vérifiés, donc il s'agit d'une erreur du serveur.
  let default_file_content = match std::fs::read(default_file_path){
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to read default file: {}", e);
      return custom_response_500(
        request,
        cookie_value,
        zero_path_buf,
        server_config,
      ).await
    }
  };
  
  let response = match Response::builder()
  .status(StatusCode::OK)
  .header("Content-Type", "text/html")
  .header("Set-Cookie", cookie_value.clone())
  .body(default_file_content)
  {
    Ok(v) => v,
    Err(e) => {
      eprintln!("ERROR: Failed to create response with default file: {}", e);
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

/// Vérifier les erreurs et retourner la réponse appropriée, en fonction des tableaux d'erreurs personnalisées dans errors.rs
pub async fn check_custom_errors(
  custom_error_string: String,
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: &PathBuf,
  server_config: ServerConfig,
  response: &mut Response<Vec<u8>>,
) {
  
  if custom_error_string != ERROR_200_OK.to_string(){
    
// Vérifier le tableau des erreurs 400
for error in CUSTOM_ERRORS_400.iter(){
      if custom_error_string == *error{
        *response = custom_response_4xx(
          request,
          cookie_value,
          zero_path_buf,
          server_config.clone(),
          StatusCode::BAD_REQUEST
        ).await;
        return
      }
    }
    
    // verifie l'erreur 413
    for error in CUSTOM_ERRORS_413.iter(){
      if custom_error_string == *error{
        *response = custom_response_4xx(
          request,
          cookie_value,
          zero_path_buf,
          server_config.clone(),
          StatusCode::PAYLOAD_TOO_LARGE
        ).await;
        return
      }
    }
    
// Vérifier l'erreur 500. En fait, il est possible de simplement retourner `custom_response_500` sans vérification. Pas de différence pour le moment.
for error in CUSTOM_ERRORS_500.iter(){
      if custom_error_string == *error{
        *response = custom_response_500(
          request,
          cookie_value,
          zero_path_buf,
          server_config.clone(),
        ).await;
        return
      }
    }
    
// Si l'erreur n'est pas trouvée, retourner une réponse personnalisée 500
*response = custom_response_500(
      request,
      cookie_value,
      zero_path_buf,
      server_config.clone(),
    ).await
  }
  
}

/// Vérifier si le chemin se termine par des pages d'erreur et retourner la réponse appropriée, ou retourner 200 OK
/// 
/// Cela est nécessaire pour les tests manuels ou les demandes de pages d'erreur
pub fn force_status(
  zero_path_buf: PathBuf,
  absolute_path_buf: PathBuf,
  server_config: ServerConfig,
)-> StatusCode {
  
  let error_pages_prefix = server_config.error_pages_prefix.clone();
  
// Vérifier si le chemin se termine par le préfixe des pages d'erreur
for error_page in ERROR_PAGES.iter(){
    
    let error_path = zero_path_buf
    .join("static")
    .join(&error_pages_prefix)
    .join(error_page);
    
    if absolute_path_buf == error_path{
      
      return match error_page{
        &"400.html" => StatusCode::BAD_REQUEST,
        &"403.html" => StatusCode::FORBIDDEN,
        &"404.html" => StatusCode::NOT_FOUND,
        &"405.html" => StatusCode::METHOD_NOT_ALLOWED,
        &"413.html" => StatusCode::PAYLOAD_TOO_LARGE,
        &"500.html" => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::OK, 
      }

    }
    
  }
  
  StatusCode::OK
}