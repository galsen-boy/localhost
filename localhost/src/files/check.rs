use std::path::Path;
use sanitise_file_name::sanitise;

use crate::server::core::ServerConfig;


/// Nom du fichier : file_exists, mais il vérifie le chemin, pas seulement les fichiers. Cependant, il est utilisé uniquement pour les fichiers.
pub fn file_exists(path: &str) -> bool{
  let path = Path::new(path);
    if path.exists() {
        // println!("File exists! {:?}", path); //todo: remove this dev print
        return true;
    } else {
        eprintln!("ERROR: File does not exist! {:?}", path);
        return false;
    }
}

pub const ERROR_PAGES: [&str; 6] = ["400.html", "403.html", "404.html", "405.html", "413.html", "500.html"];

/// Vérifier les chemins relatifs. Le niveau parent est le dossier de l'exécutable.
/// 
/// Vérification minimale des fichiers, pour éviter que le serveur ne se lance sans les fichiers requis par la tâche
pub async fn all_files_exists(server_configs: &Vec<ServerConfig>) -> bool{
  
// Vérifier le script CGI requis par la tâche
  if !file_exists("cgi/useless.py"){
    eprintln!("ERROR: Path cgi/useless.py does not exist");
    return false
  }

// vérifier les pages d'erreur personnalisées requises par la tâche
  for server_config in server_configs{
    let error_prefix =
    "static/".to_owned()+&server_config.error_pages_prefix; // error pages path prefix
    for file_name in ERROR_PAGES{
      if !file_exists( &(error_prefix.to_owned() + "/" + file_name)){
        eprintln!("ERROR: Error page {} does not exist", file_name);
        return false
      }
    }

// vérifier le fichier par défaut requis par la tâche
    let static_prefix =
    "static/".to_owned()+&server_config.static_files_prefix; // préfixe du chemin des fichiers statiques
    if !file_exists( &(static_prefix.to_owned() + "/" + &server_config.default_file)){
      eprintln!("ERROR: Default file {} does not exist", &server_config.default_file);
      return false
    }

  }

  true
}

// vérifier que le chemin du fichier se termine par le nom de la page d'erreur. Seules les pages d'erreur mises en œuvre sont vérifiées.
pub fn is_implemented_error_page(path: &str) -> bool{
  for error_page in ERROR_PAGES{
    if path.ends_with(error_page){ return true }
  }
  false
}

// assainir + remplacer les espaces par des underscores + remplacer les doubles underscores par un seul underscore
pub fn sanitise_file_name(file_name: &str) -> String{
  sanitise( file_name ).replace(" ", "_").replace("__", "_")
}
pub fn bad_file_name(file_name: &str) -> bool{ sanitise_file_name(file_name) != file_name }