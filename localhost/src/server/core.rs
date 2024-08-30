use async_std::sync::Mutex;
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use std::time::SystemTime;

use crate::server::cookie::Cookie;


#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
  pub server_name: String,
  pub ports: Vec<String>,
  pub server_address: String,
  pub client_body_size: usize,
  pub static_files_prefix: String,
  pub default_file: String,
  pub error_pages_prefix: String,
  pub uploads_methods: Vec<String>,
  pub routes: HashMap<String, Vec<String>>,
}

impl ServerConfig {
/// Éliminer tous les ports hors de la plage 0..65535, ainsi que les ports répétés
  pub async fn check(&mut self){
    self.check_ports();
  }
  
  fn check_ports(&mut self){
    let old_ports = self.ports.clone();
    let mut ports: HashSet<String> = HashSet::new();
    for port in self.ports.iter(){
      let port: u16 = match port.parse(){
        Ok(v) => v,
        Err(_e) =>{
          // eprintln!("ERROR: Config \"{}\" Failed to parse port: {} into u16", self.server_name, e);
          continue;
        }
      };
      ports.insert(port.to_string());
    }
    self.ports = ports.into_iter().collect();
    if self.ports.len() != old_ports.len(){
      println!("=== La configuration de port du server \"{}\" passe ===\nDe {:?}\n  à {:?}", self.server_name, old_ports, self.ports);
    }
    
  }
  
}

/// Obtenir la liste des ports uniques depuis `server_configs`, pour utiliser `listen 0.0.0.0:port`
/// 
/// Et gérer les serveurs pseudo, car la tâche nécessite une redirection
/// 
/// Si aucun nom de serveur déclaré dans la requête, utiliser le serveur "par défaut"
pub async fn get_usize_unique_ports(server_configs: &Vec<ServerConfig>) -> Result<Vec<usize>, Box<dyn Error>>{
  let mut ports: HashSet<usize> = HashSet::new();
  for server_config in server_configs.iter(){
    for port in server_config.ports.iter(){
      let port: u16 = match port.parse(){
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to parse port: {} into u16: {}", port, e).into()),
      };
      ports.insert(port as usize);
    }
  }
  let ports: Vec<usize> = ports.into_iter().collect();
  
  if ports.len() < 1 {
    return Err("Not enough correct ports declared in \"settings\" file".into());
  }
  
  Ok(ports)
  
}

#[derive(Debug)]
pub struct Server {
  pub cookies: Arc<Mutex<HashMap<String, Cookie>>>,
  pub cookies_check_time: SystemTime,
}
