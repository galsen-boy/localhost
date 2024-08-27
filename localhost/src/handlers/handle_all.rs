use async_std::path::PathBuf;

use http::{Response, Request, StatusCode, HeaderValue};

// use crate::debug::append_to_file;
use crate::files::check::is_implemented_error_page;
use crate::handlers::response_500::custom_response_500;
use crate::handlers::response_::{response_default_static_file, force_status};
use crate::handlers::response_4xx::custom_response_4xx;
use crate::server::core::ServerConfig;


// traiter toutes les demandes, sauf CGI et sauf téléchargements.
// De plus, si l'URI est un répertoire, la tâche exige de renvoyer le fichier par défaut,
// conformément à la configuration du serveur. Dans ce cas, il n'est pas nécessaire de vérifier la méthode
// autorisée pour la route.
pub async fn handle_all(
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: &PathBuf,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{

// remplacer /uploads/ par /, pour éviter les chemins incorrects. Les fichiers de téléchargement sont servis séparément au niveau supérieur
  let binding_path_string = request.uri().path().replacen("uploads/", "", 1);
  let mut path_str = binding_path_string.as_str();
  
  if path_str.starts_with("/"){ path_str = &path_str[1..]; }
  
// vérifier si le chemin est une page d'erreur
  let is_error_page = is_implemented_error_page(path_str);
// chemin vers le dossier du site dans le dossier statique
let relative_static_path_string =
  if is_error_page {
    
    let file_name = match path_str.split('/').last(){
      Some(v) => v,
      None => {
        // eprintln!("ERROR: path_str.split('/').last()\nFailed with path {}", path_str);
        // eprintln!(" Must never fire, because path checked/confirmed before.\nSo return [500]");
        return custom_response_500(
          request,
          cookie_value,
          zero_path_buf,
          server_config,
        ).await
      }
    };
    format!("static/{}/{}", server_config.error_pages_prefix, file_name)
  }
  else { format!("static/{}/{}", server_config.static_files_prefix, path_str)};
  
  let absolute_path_buf = zero_path_buf.join(relative_static_path_string);
  
// vérifier si le chemin est un répertoire, puis renvoyer le fichier par défaut comme l'exige la tâche
if path_str.ends_with("/") || absolute_path_buf.is_dir().await {
    
// implémenter la vérification de l'erreur 403 si la méthode n'est pas GET, pour satisfaire les exigences de la tâche
if request.method().to_string() != "GET" {
      // eprint!("ERROR: Status code 403 FORBIDDEN. CUSTOM IMPLEMENTATION.\nOnly the \"GET\" method is allowed to access the directory.");
      return custom_response_4xx(
        request,
        cookie_value,
        zero_path_buf,
        server_config,
        StatusCode::FORBIDDEN,
      ).await
    }
    
    return response_default_static_file(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
    ).await
  } else if !absolute_path_buf.is_file().await {
    
    // eprintln!("ERROR:\n------------\nIS NOT A FILE\n-------------");
    
    return custom_response_4xx(
      request, 
      cookie_value,
      zero_path_buf, 
      server_config,
      StatusCode::NOT_FOUND,
    ).await
  } // vérifier si le fichier existe ou renvoyer une erreur 404

  
// vérifier si le chemin est dans les routes, puis obtenir les méthodes autorisées pour ce chemin
let mut rust_handicap_binding:Vec<String> = Vec::new();
  let allowed_methods: &Vec<String> = match server_config.routes.get(path_str){
    Some(v) => {v},
    None => {
      if is_error_page {
        rust_handicap_binding.push("GET".to_string());
        &rust_handicap_binding
        
      } else {
        // eprintln!("ERROR: Path {} is not inside routes", path_str);
        return custom_response_4xx(
          request,
          cookie_value,
          zero_path_buf,
          server_config,
          http::StatusCode::NOT_FOUND,
        ).await
      }
    }
  };
  
 // vérifier si la méthode est autorisée pour ce chemin, sinon renvoyer une erreur 405
  let request_method_string = request.method().to_string();
  if !allowed_methods.contains(&request_method_string){
    // eprintln!("ERROR: Method {} is not allowed for path {}", request_method_string, path_str);
    return custom_response_4xx(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
      http::StatusCode::METHOD_NOT_ALLOWED,
    ).await
  }
  
// lire le fichier. En cas d'erreur, renvoyer une réponse d'erreur 500
let file_content = match std::fs::read(absolute_path_buf.clone()){
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to read file: {}", e);
      return custom_response_500(
        request,
        cookie_value,
        zero_path_buf,
        server_config
      ).await
    }
  };
  
  let mut response = match Response::builder()
  .status(
    force_status(
      zero_path_buf.clone(),
      absolute_path_buf.clone(),
      server_config.clone(),
    )
  )
  .header("Set-Cookie", cookie_value.clone())
  .body(file_content)
  {
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to create response with file: {}", e);
      return custom_response_500(
        request,
        cookie_value.clone(),
        zero_path_buf,
        server_config
      ).await
    }
  };
  
// obtenir le type MIME du fichier en utilisant mime_guess, ou utiliser text/plain
let mime_type = match mime_guess::from_path(absolute_path_buf.clone()).first(){
    Some(v) => v.to_string(),
    None => "text/plain".to_string(),
  };
  // append_to_file(&format!("\n-------\n\nmime_type {}\n\n----------\n", mime_type)).await;
  
  response.headers_mut().insert(
    "Content-Type",
    match mime_type.parse(){
      Ok(v) => v,
      Err(_e) => {
        // eprintln!("ERROR: Failed to parse mime type: {}", e);
        HeaderValue::from_static("text/plain")
      }
    }
  );
  
  response
  
}
