use async_std::path::PathBuf;

use http::{Response, Request, StatusCode};

use crate::handlers::response_4xx::custom_response_4xx;
use crate::handlers::response_500::custom_response_500;
use crate::handlers::uploads_delete::delete_the_file_from_uploads_folder;
use crate::handlers::uploads_get::generate_uploads_html;
use crate::handlers::uploads_set::upload_the_file_into_uploads_folder;
use crate::server::core::ServerConfig;
use crate::stream::errors::{ERROR_200_OK, ERROR_400_BAD_REQUEST};
use crate::stream::errors:: ERROR_400_HEADERS_FAILED_TO_PARSE;
use crate::stream::errors::ERROR_400_HEADERS_KEY_NOT_FOUND;
#[allow(unused_assignments)]


/// gérer les requêtes de téléchargement.
/// 
/// La tâche consiste à mettre en œuvre le traitement des requêtes de téléchargement.
/// 
/// Les méthodes GET, POST et DELETE sont autorisées pour
/// 
/// GET - obtenir la page HTML dynamique générée par le code, incluant
/// la liste des fichiers dans le dossier de téléchargements, avec un bouton [DELETE] pour envoyer une requête DELETE,
/// et un bouton [UPLOAD], avec un formulaire pour télécharger un nouveau fichier.
/// 
/// POST - pour télécharger un nouveau fichier, en utilisant le formulaire de la requête GET précédente.
/// 
/// DELETE - pour supprimer le fichier, en utilisant le formulaire de la requête GET précédente.
/// Appuyez sur le bouton [DELETE] pour envoyer la requête DELETE au serveur.

pub async fn handle_uploads(
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: &PathBuf,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{
  let mut path = request.uri().path();
  if path.starts_with("/"){ path = &path[1..]; }
  
  let absolute_path = zero_path_buf.join("uploads");
  
  if !absolute_path.is_dir().await {
    
    // eprintln!("ERROR: absolute_path {:?} is not a folder.\nThe file structure was damaged after the server started.", absolute_path);
    
    return custom_response_500(
      request,
      cookie_value,
      zero_path_buf,
      server_config
    ).await
  }
  
// Méthodes autorisées pour ce chemin, selon la tâche : GET, POST, DELETE
  let allowed_methods:Vec<String> = vec![
  "GET".to_string(),
  "POST".to_string(),
  "DELETE".to_string(),
  ];
  
  // Vérifiez si la méthode est autorisée pour ce chemin ou renvoyez un code 405
  let request_method_string = request.method().to_string();
  if !allowed_methods.contains(&request_method_string){
    eprintln!("ERROR: Method {} is not allowed for uploads", request_method_string);
    return custom_response_4xx(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
      http::StatusCode::METHOD_NOT_ALLOWED,
    ).await
  } else if !server_config.uploads_methods.contains(&request_method_string){
    // eprintln!("ERROR: Method {} is not allowed for uploads in server_config", request_method_string);
    return custom_response_4xx(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
      http::StatusCode::METHOD_NOT_ALLOWED,
    ).await
  }
  
  
  let mut body_content:Vec<u8> = Vec::new();
  
  match request_method_string.as_str(){
    "GET" => { /* Ne rien faire de spécifique. La page HTML est générée ci-dessous */},
    "POST" => {
      
      match upload_the_file_into_uploads_folder(request, &absolute_path).await.as_str(){
        ERROR_200_OK => { },
        ERROR_400_HEADERS_KEY_NOT_FOUND => {
          // eprintln!("ERROR: Header \"X-File-Name\" not found in request");
          return custom_response_4xx(
            request,
            cookie_value,
            zero_path_buf,
            server_config,
            StatusCode::BAD_REQUEST,
          ).await
        },
        ERROR_400_HEADERS_FAILED_TO_PARSE => {
          // eprintln!("ERROR: Failed to parse header_value into file_name");
          return custom_response_4xx(
            request,
            cookie_value,
            zero_path_buf,
            server_config,
            StatusCode::BAD_REQUEST,
          ).await
        },
        _ => {
          // eprintln!("ERROR: Failed to upload the file into uploads folder");
          return custom_response_500(
            request,
            cookie_value,
            zero_path_buf,
            server_config,
          ).await
        },
      };

    },
    "DELETE" => {
      match delete_the_file_from_uploads_folder(request, &absolute_path).await.as_str(){
        ERROR_200_OK => {  },
        ERROR_400_BAD_REQUEST => {
          // eprintln!("ERROR: Failed to parse body into file_name");
          return custom_response_4xx(
            request,
            cookie_value,
            zero_path_buf,
            server_config,
            StatusCode::BAD_REQUEST,
          ).await
        },
        _ => {
          // eprintln!("ERROR: Failed to delete the file from uploads folder");
          return custom_response_500(
            request,
            cookie_value,
            zero_path_buf,
            server_config,
          ).await
        }
      };
    },
    _ => {
      // eprintln!("ERROR: Method {} is not implemented for path {}.\nShould never fire, because checked above!!!", request_method_string, path);
      return custom_response_500(
        request,
        cookie_value,
        zero_path_buf,
        server_config
      ).await
    },
  }
  
// Générer la page HTML avec la liste des fichiers dans le dossier de téléchargements
  let (html, status) = generate_uploads_html( &absolute_path, ).await;
  if status != ERROR_200_OK {
    // eprintln!("ERROR: Failed to generate html page with list of files in uploads folder");
    return custom_response_500(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
    ).await
  }

  body_content.extend_from_slice(html.as_bytes());

  let response = match Response::builder()
  .status(StatusCode::OK)
  .header("Content-Type", "text/html")
  .header("Set-Cookie", cookie_value.clone())
  .body(body_content)
  {
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to create response with body_content: {}", e);
      return custom_response_500(
        request,
        cookie_value.clone(),
        zero_path_buf,
        server_config
      ).await
    }
  };
  
  response
  
}
