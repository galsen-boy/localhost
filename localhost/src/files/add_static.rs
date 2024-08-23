use core::panic;
use std::error::Error;

use walkdir::WalkDir;

use crate::server::core::ServerConfig;

// Ajouter des fichiers statiques aux configurations de serveur pour les routes, avec la méthode GET. Ignorer les routes déjà existantes, afin de préserver les méthodes autorisées pour les routes, telles que définies dans les paramètres.
pub async fn add_static_files_to_server_configs(server_configs: &mut Vec<ServerConfig>) -> Result<(), Box<dyn Error>>{
// Préfixe du chemin relatif des fichiers statiques
let static_files_root = "static/".to_owned();
  for server_config in server_configs{
    let static_files_prefix = static_files_root.to_owned() + &server_config.static_files_prefix.to_owned();
    // get the routes to add static files to... css images etc
    let routes = &mut server_config.routes;
    
// Obtenir les routes pour ajouter des fichiers statiques... CSS, images, etc.
  for entry in WalkDir::new(&static_files_prefix).into_iter().filter_map(|e| e.ok()) {
    // Obtenir le chemin du fichier
    let file_path = entry.path();
      // check if it is a file
      if !file_path.is_file(){ continue; }
      
     // Chemin relatif vers le dossier des fichiers statiques
      let relative_file_path = match file_path.strip_prefix(&static_files_prefix){
        Ok(v) => v,
        Err(e) => panic!("Failed to strip prefix: {} from file path: {} | {}", static_files_prefix, file_path.display(), e),
      };
      
      // println!("add \"{}\"", relative_file_path.to_string_lossy().trim_start_matches(&static_files_prefix));
      
      // Ajouter la route à la configuration du serveur, avec la méthode GET
      let key = match relative_file_path.to_str(){
        Some(v) => v.to_owned(),
        None => panic!("Failed to convert file path to str. Static file path: {}", relative_file_path.display()),
      };
      
      // Vérifier si la route existe déjà, puis l'ignorer
      if routes.contains_key(&key){ continue; }
      
      let value = vec!["GET".to_owned()];
      
      routes.insert(key, value);
      
      // let file_name = relative_file_path.file_name()
      
    }
    
  }
  
  return Ok(())
}