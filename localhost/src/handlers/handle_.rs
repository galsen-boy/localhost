use http::{Request, Response};
use async_std::path::PathBuf;

use crate::server::core::ServerConfig;
use crate::handlers::handle_cgi::handle_cgi;
use crate::handlers::handle_all::handle_all;
use crate::handlers::handle_uploads::handle_uploads;
use crate::handlers::handle_redirected::handle_redirected;
use crate::handlers::uploads_get::handle_uploads_get_uploaded_file;


// traiter toutes les demandes.
// Les demandes CGI sont traitées comme des cas de correspondance séparés.
// Les demandes de téléchargement sont traitées comme des cas de correspondance séparés.
pub async fn handle_request(
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: &PathBuf,
  server_config: ServerConfig,
  _global_error_string: &mut String, 
) -> Response<Vec<u8>>{
  
// essayer de gérer le cas de la demande CGI de manière stricte et séparée,
// afin de réduire les vulnérabilités, car CGI est ancien, peu sûr et déconseillé.
// De plus, la tâche est de faible qualité, car la question de l'audit demande uniquement de vérifier
// les requêtes CGI avec des requêtes chunkées et non chunkées, donc la vérification des méthodes n'est pas implémentée,
// car selon la norme HTTP/1.1, une méthode autre que POST peut aussi avoir un corps.

  let path = request.uri().path();
  let parts: Vec<&str> = path.split('/').collect();
  
  let response = match parts.as_slice(){
    ["", "cgi", "useless.py", file_path @ ..] => {
      handle_cgi(
        request,
        cookie_value,
        zero_path_buf,
        "useless.py".to_string(),
        file_path.join(&std::path::MAIN_SEPARATOR.to_string()),
        server_config,
      ).await

    },
    ["", "uploads"] => {
      handle_uploads(
        request,
        cookie_value,
        zero_path_buf,
        server_config,
      ).await
      
    },
    ["", "redirected"] => {
      handle_redirected(
        request,
        cookie_value,
        zero_path_buf,
        server_config,
      ).await

    },
    ["", "uploads", file_path ] => {
      handle_uploads_get_uploaded_file(
        request,
        cookie_value,
        zero_path_buf,
        file_path.to_string(),
        server_config,
      ).await
    },
    _ => {
// réponse pour les autres cas
    handle_all(
        request,
        cookie_value,
        zero_path_buf,
        server_config,
      ).await
    }
  };
  
  response
}
