use http::{Request, HeaderMap, HeaderName, HeaderValue};
use crate::server::core::ServerConfig;
use crate::stream::parse::parse_request_line;


pub async fn server_config_from_headers_buffer_or_use_default(
  headers_buffer: &Vec<u8>,
  server_configs: Vec<ServerConfig>
) -> ServerConfig{
  
  let mut server_config = server_configs[0].clone(); // default server config

  if headers_buffer.is_empty() {
    // eprintln!("ERROR: headers_buffer is empty");
    return server_config
  }

  let headers_string = match String::from_utf8( headers_buffer.clone() ){
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to convert headers_buffer to string:\n {}", e);
      return server_config
    }
  };

 // Diviser la chaîne de requête en lignes

// Séparer la requête brute en ... morceaux sous forme de vecteur
  let mut headers_lines: Vec<String> = Vec::new();
  for line in headers_string.split('\n'){
    headers_lines.push(line.to_string());
  }
  
  if headers_lines.is_empty() {
    // eprintln!("ERROR: headers_lines is empty");
    return server_config
  }

// Initialiser un nouveau HeaderMap pour stocker les en-têtes HTTP
  let mut headers = HeaderMap::new();
  
// Analyser la ligne de requête, qui doit être la première
  let request_line: String = match headers_lines.get(0) {
    Some(value) => {value.to_string()},
    None => {
      // eprintln!("ERROR: Fail to get request_line");
      return server_config
    },
  };
  
  let (method, uri, version) = match parse_request_line(request_line.clone()).await{
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to parse request_line: {}", e);
      return server_config
    }
  };

  // Parse the headers
  for line_index in 1..headers_lines.len() {
    // global_index += 1;
    let line: String = match headers_lines.get(line_index){
      Some(value) => {value.to_string()},
      None => {
        // eprintln!("ERROR: Fail to get header line");
        return server_config
      },
    };
    
    if line.is_empty() { break } // S'attendre à ce que cela puisse marquer la fin de la section des en-têtes
    
    let parts: Vec<String> = line.splitn(2, ": ").map(|s| s.to_string()).collect();
    if parts.len() == 2 {
      let header_name = match HeaderName::from_bytes(parts[0].as_bytes()) {
        Ok(v) => v,
        Err(_e) =>{
          // eprintln!("ERROR: Invalid header name: {}\n {}", parts[0], e);
          return server_config
        },
      };
      
      let value = HeaderValue::from_str( parts[1].trim());
      match value {
        Ok(v) => headers.insert(header_name, v),
        Err(_e) =>{
          // eprintln!("ERROR: Invalid header value: {}\n {}", parts[1], e);
          return server_config
        },
      };
      
    }
  }
  
  let body_buffer: Vec<u8> = Vec::new(); // Juste un espace pour compléter le builder
// Construire l'objet http::Request
  let mut request = match Request::builder()
  .method(method)
  .uri(uri)
  .version(version)
  .body(body_buffer){
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to construct the http::Request object: {}", e);
      return server_config
    }
  };
  
 // Essayer de remplir les en-têtes, car dans le constructeur, il semble qu'il n'y ait pas de méthode
// pour créer des en-têtes à partir de HeaderMap, mais peut-être que le remplacement forcé peut également être utilisé
  let request_headers = request.headers_mut();

  for (key,value) in headers{
    let header_name = match key {
      Some(v) => v,
      None => {
        // eprintln!("ERROR: Invalid header name"); 
        return server_config
      },
    };
    
    request_headers.append(header_name, value);
  }

  // Choisir la configuration du serveur en fonction du nom du serveur et du port de la requête,
// ou utiliser "par défaut", comme l'exige la tâche
  
  server_config = server_configs[0].clone(); // default server config
  let request_server_host  = match request.headers().get("host"){
    Some(value) => {
      match value.to_str(){
        Ok(v) => v.to_string(),
        Err(_e) => {
          // eprintln!("ERROR: Failed to convert request host header value \"{:?}\" to str: {}.\n=> USE \"default\" server config with first port", value, e);
          server_config.server_name.clone() + ":" + &server_config.ports[0]
        }
      }
    },
    None => { 
      // eprintln!("ERROR: Failed to get request host.\n=> USE \"default\" server config with first port");
      server_config.server_name.clone() + ":" + &server_config.ports[0]
    },
  };
  
 // Itérer sur les configurations du serveur et utiliser celle qui correspond, deux variantes possibles :
// Correspondance de serverconfig.server_name + ":" + &serverconfig.ports[x] (pour chaque port) == request_server_host
// Correspondance de server_config.server_address + ":" + &server_config.ports[x] (pour chaque port) == request_server_host
  for config in server_configs{
    let server_name = config.server_name.to_owned();
    let server_address = config.server_address.to_owned();
    for port in config.ports.clone(){
      let name_port_host = server_name.to_owned() + ":" + &port;
      let address_port_host = server_address.to_owned() + ":" + &port;
      if name_port_host == request_server_host
      || address_port_host == request_server_host
      {
        server_config = config.clone();
        break;
      }
    }
  }
  
  server_config
  
}