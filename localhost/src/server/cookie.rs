use std::time::{SystemTime, Duration};
use chrono::{DateTime, Utc};
use http::Request;
use uuid::Uuid;

use crate::{server::core::Server};



#[derive(Debug, Clone)]
pub struct Cookie {
  pub name: String,
  pub value: String,
/// Temps d'expiration en secondes depuis l'UNIX_EPOCH
  pub expires: u64,
}

impl Cookie {
  async fn is_expired(&self) -> bool {
    let now = SystemTime::now();
    let expiration = SystemTime::UNIX_EPOCH + Duration::from_secs(self.expires);
    expiration < now
  }
  
  async fn to_string(&self) -> String {
    let datetime: DateTime<Utc> = DateTime::from_timestamp(self.expires as i64, 0)
    .unwrap_or(Utc::now() + Duration::from_secs(60));
    let expires_str = datetime.to_rfc2822();
    // format!( "{}={}; Expires={}; HttpOnly; Path=/", self.name, self.value, self.expires )
    // format!( "{}={}", self.name, self.value )
    format!("{}={}; Expires={}; HttpOnly; Path=/", self.name, self.value, expires_str)
  }
}

impl Server {
  
  async fn generate_unique_cookie_and_return(&mut self) -> Cookie {
    let name = Uuid::new_v4().to_string();
    let value = Uuid::new_v4().to_string();
    let cookie = self.set_cookie( name.clone(), value.clone(), Duration::from_secs(60) ).await;
    cookie
  }
  
  async fn set_cookie(&mut self, name: String, value: String, life_time: Duration) -> Cookie {
    let expiration = SystemTime::now() + life_time;
    let expires = match
    expiration.duration_since(SystemTime::UNIX_EPOCH){
      Ok(v) => v,
      Err(_e) => {
        // eprintln!("ERROR: Failed to get duration_since for cookie name {}: {}", name, e);
        Duration::new(0, 0)
      }
    }
    .as_secs();
    
    let cookie = Cookie { name: name.clone(), value, expires };
    
    // append_to_file(
    //   &format!( "===\n self.cookies before insert:\n{:?}\n===", self.cookies )
    // ).await;
    
    let mut guard_cookies = self.cookies.lock().await;
    guard_cookies.insert( cookie.name.clone(), cookie.clone() );
    drop(guard_cookies);
    
    // append_to_file(
    //   &format!( "===\n self.cookies after insert:\n{:?}\n===", self.cookies )
    // ).await;
    
    
    cookie
  }
  
  pub async fn get_cookie(&self, name: &str) -> Cookie{
    let guard_cookies = self.cookies.lock().await;
    match guard_cookies.get(name){
      Some(v) => {
        return v.clone();
      },
      None => {
        drop(guard_cookies);
        // eprintln!("ERROR: Failed to get cookie by name: {}", name);
        return Cookie { name: String::new(), value: String::new(), expires: 0}
      },
    };
    
  }
  
/// Extraire les cookies de la requête. Si le cookie n'est pas trouvé, générer un nouveau cookie pour une minute.
/// 
/// Retourner également un booléen. False si le cookie est reconnu comme invalide, pour une réponse de requête incorrecte.
  pub async fn extract_cookies_from_request_or_provide_new(
    &mut self,
    request: &Request<Vec<u8>>
  ) -> (String, bool) {
    
    // append_to_file("EXTRACT COOKIES FROM REQUEST OR PROVIDE NEW").await;
    let cookie_header_value = match request.headers().get("Cookie"){
      Some(v) =>{
        // append_to_file(&format!( "Cookie header value: {:?}", v )).await;
        v
      },
      None =>{ // Pas d'en-tête de cookie, un nouveau cookie sera généré
        // append_to_file("No \"Cookie\" header").await;
        let cookie = self.generate_unique_cookie_and_return().await;
        // append_to_file(&format!( "New cookie: {:?}", cookie )).await;
        return (self.send_cookie(cookie.name).await, true)
      }
    };
    
    let cookie_header_value_str = match cookie_header_value.to_str(){
      Ok(v) => v,
      Err(_e) => {
        // eprintln!("ERROR: Failed to get cookie_header.to_str: {}", e);
        let cookie = self.generate_unique_cookie_and_return().await;
        return (self.send_cookie(cookie.name).await, false)
      }
    }
    .trim();
    
// Diviser l'en-tête de cookie par "; " pour obtenir toutes les parties du cookie, comme "name=value" ou "name" pour les indicateurs.
  let cookie_parts:Vec<&str> = cookie_header_value_str.split("; ").collect();
    let cookie_parts:Vec<&str> = cookie_parts.iter().map(|v| v.trim()).collect();
    
    // append_to_file(
    //   &format!( "===\n incoming Cookie parts: {:?}\n===", cookie_parts )
    // ).await;
    
    // append_to_file(
    //   &format!( "===\n server.cookies: {:?}\n===", self.cookies )
    // ).await;
    
    
// Vérifier toutes les parties du cookie, essayer de les trouver dans server.cookies
// Si le cookie n'est pas trouvé, générer un nouveau cookie pour une minute
// Si le cookie est trouvé, vérifier s'il a expiré. Si oui, le retirer de server.cookies, générer un nouveau cookie pour une minute et le retourner comme valeur pour l'en-tête
// Si le cookie n'est pas expiré, le retourner comme valeur pour l'en-tête
// S'il y a plus d'un cookie trouvé dans server.cookies, générer un nouveau cookie pour une minute et le retourner comme valeur pour l'en-tête
    let mut cookie_found = false;
    let mut broken_cookie_found = false;
    let mut expired_cookie_found = false;
    let mut more_then_one_cookie_found = false;
    let mut found_cookie_name = String::new();
    
    let mut guard_cookies = self.cookies.lock().await;
    
    for cookie_part in cookie_parts.iter(){
      let cookie_part: Vec<&str> = cookie_part.splitn(2, '=').collect();
      let part_name = cookie_part[0];
      
      if let Some(server_cookie) = guard_cookies.get(part_name){
        if cookie_found { more_then_one_cookie_found = true; }
        cookie_found = true;
        
        // verifie si le cookie est correct
        if cookie_part.len() == 2 {
          let part_value = cookie_part[1];
          if part_value != server_cookie.value{
            // eprintln!("ERROR: Cookie\n{}\nfound in server cookies with different value\n{}\n. Potential security risk", cookie_part.join("="), server_cookie.to_string().await);
            broken_cookie_found = true;
          } else if !more_then_one_cookie_found { // first cookie found, use it
            found_cookie_name = part_name.to_string();
          }
        }
        
// Vérifier si le cookie du serveur avec le même nom est expiré
  if server_cookie.is_expired().await{
          expired_cookie_found = true;
          guard_cookies.remove(part_name);
        }
        
      }
      
    }
    
    drop(guard_cookies);
    
    if expired_cookie_found || !cookie_found
    {
      let cookie = self.generate_unique_cookie_and_return().await;
      return (self.send_cookie(cookie.name).await, true)
    } else if broken_cookie_found {
      // eprintln!("ERROR: Found broken cookie. Same name as server cookie, but different value. Potential security risk");
      let cookie = self.generate_unique_cookie_and_return().await;
      return (self.send_cookie(cookie.name).await, false)
    } else {
      return (self.send_cookie(found_cookie_name).await, true)
    }
    
  }
  
  /// Obtenir le cookie par son nom. Si le cookie n'est pas trouvé, générer un nouveau cookie pour une minute
 //
/// Retourner la valeur de l'en-tête du cookie sous forme de chaîne "{}={}; Expires={}; HttpOnly; Path=/" à envoyer dans la réponse

  pub async fn send_cookie(&mut self, name: String) -> String {
    let guard_cookies = self.cookies.lock().await;
    
    if let Some(cookie) = guard_cookies.get(&name){
      return cookie.to_string().await;
    } else { // Si le cookie n'est pas trouvé, générer un nouveau cookie pour une minute
      drop(guard_cookies);
      let cookie = self.generate_unique_cookie_and_return().await;
      return cookie.to_string().await;
    }
    
  }
  
/// Supprimer tous les cookies expirés. Utilisé avec un délai de 60 secondes, pour ne pas vérifier à chaque requête
pub async fn check_expired_cookies(&mut self){
    let now = SystemTime::now();
    if now > self.cookies_check_time {
      let guard_cookies = self.cookies.lock().await;
      
      // Recueillir tous les cookies expirés
      let mut expired_cookies = Vec::new();
      for (name, cookie) in guard_cookies.iter(){
        let expiration = SystemTime::UNIX_EPOCH + Duration::from_secs(cookie.expires);
        if expiration < now {
          expired_cookies.push(name.clone());
          // append_to_file(&format!( "EXPIRED COOKIE: {:?}", cookie )).await;
        }
      }
      drop(guard_cookies);
      
     // Redéclarer comme mutable, et peut-être, permettre, pendant le temps de pause,
    // de l'utiliser à un autre endroit. Pas sûr
      let mut guard_cookies = self.cookies.lock().await;
      // supprime tous les cookies expires
      for name in expired_cookies.iter(){ guard_cookies.remove(name); }
      drop(guard_cookies);
      
    }
// Définir le prochain temps de vérification, une minute à partir de maintenant
  self.cookies_check_time = now + Duration::from_secs(60);
    
  }
  
}