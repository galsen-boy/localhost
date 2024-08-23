use std::process::Command;
use async_std::path::PathBuf;

use http::{Request, Response, StatusCode};

use crate::server::core::ServerConfig;
use crate::handlers::response_500::custom_response_500;
use crate::handlers::response_4xx::custom_response_4xx;

/// exécute le script python, et vérifie si le chemin est un fichier, un dossier ou n'existe pas/chemin incorrect
/// 
/// potentiellement dangereux, car vous pouvez passer n'importe quel chemin au script en utilisant
/// 
/// cgi/useless.py//some/path/here. mais dans ce cas précis, il est seulement permis de vérifier
pub async fn handle_cgi(
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: &PathBuf,
  script_file_name: String,
  check_file_path: String,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{
  
  // vérifier si la méthode est GET, POST ou DELETE, sinon retourner 405
  if request.method() != "GET" && request.method() != "POST" && request.method() != "DELETE"{
    // eprintln!("ERROR: Method {} is not allowed for cgi", request.method());
    return custom_response_4xx(
      request,
      cookie_value.clone(),
      zero_path_buf,
      server_config,
      StatusCode::METHOD_NOT_ALLOWED,
    ).await
  }

  let script_path = "cgi/".to_owned() + &script_file_name;
  
  // vérifier si le script existe encore, sinon retourner 500, car avant de démarrer le serveur, nous vérifions les fichiers obligatoires
  if !zero_path_buf.join(&script_path).exists().await{
    // eprintln!("ERROR: script_path {:?} is not exist.\nThe file structure was damaged after the server started.", zero_path_buf.join(&script_path));
    return custom_response_500(
      request,
      cookie_value.clone(),
      zero_path_buf,
      server_config,
    ).await
  }

  // Définir le PATH_INFO du système ou envoyer le chemin de la requête comme argument au script python3
  let output = Command::new("python3")
  .arg(script_path)
  .arg(check_file_path)
  .output();
  
  let result = match &output{
    Ok(v) => match std::str::from_utf8(&v.stdout){
      Ok(v) => {
        if v.trim() == ""{
          "Empty output from cgi python3 script"
        } else {
          v
        }
      },
      Err(e) => {
        let error_message = "Failed to convert cgi output to str. ".to_owned() + &e.to_string();
        Box::leak(error_message.into_boxed_str())
      }
    },
    Err(e) => {
      let error_message = "Failed to get cgi output. ".to_owned() + &e.to_string();
      Box::leak(error_message.into_boxed_str())
    } // nouvelle énigme. au lieu de retourner l'erreur sous forme de chaîne (au lieu de coder), cassez-vous la tête pendant des heures avec les emprunts/liaisons/vie, car il est tout simplement impossible de le faire naturellement. Rust est nul
  };
  
  // écrire dans le flux
  let body = format!("Hello from Rust and Python3: {}\n\n", result)
  .as_bytes().to_vec();
  
  let response = match Response::builder()
  .status(StatusCode::OK)
  .header("Content-Type", "text/plain")
  .header("Set-Cookie", cookie_value.clone())
  .body(body)
  {
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to build cgi response body | {}", e);
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
