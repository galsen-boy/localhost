use std::str;
use std::error::Error;
use http::{Request, Method, Uri, Version, HeaderMap, HeaderValue, HeaderName};

/// Fonction pour analyser une requête HTTP brute à partir d'un tampon Vec<u8> en un http::Request
pub async fn parse_raw_request(
  headers_buffer: Vec<u8>,
  body_buffer: Vec<u8>,
  request: &mut Request<Vec<u8>>,
  global_error_string: &mut String,
) {
  
  if headers_buffer.is_empty() {
    // eprintln!("ERROR: parse_raw_request: headers_buffer is empty");
    *global_error_string = ERROR_400_HEADERS_BUFFER_IS_EMPTY.to_string();
    return;
  }
  
  let headers_string = match String::from_utf8(headers_buffer.clone()){
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to convert headers_buffer to string:\n {}", e);
      *global_error_string = ERROR_400_HEADERS_BUFFER_TO_STRING.to_string();
      return;
    }
  };
  
  // Diviser la chaîne de requête en lignes
 // let mut lines = request_str.lines(); // à faire : ne jamais utiliser cette méthode. Elle est inadaptée pour des approches plus complexes que "hello\nworld"
// Séparer la requête brute en ... morceaux sous forme de vecteur
  let mut headers_lines: Vec<String> = Vec::new();
  for line in headers_string.split('\n'){
    headers_lines.push(line.to_string());
  }
  
  if headers_lines.is_empty() {
    // eprintln!("ERROR: headers_lines is empty");
    *global_error_string = ERROR_400_HEADERS_LINES_IS_EMPTY.to_string();
    return;
  }
  
// Initialiser un nouveau HeaderMap pour stocker les en-têtes HTTP
  let mut headers = HeaderMap::new();
  
// Analyser la ligne de requête, qui doit être la première
let request_line: String = match headers_lines.get(0) {
    Some(value) => {value.to_string()},
    None => {
      // eprintln!("ERROR: Failed to get request_line");
      *global_error_string = ERROR_500_INTERNAL_SERVER_ERROR.to_string();
      return;
    },
  };
  
  let (method, uri, version) = match parse_request_line(request_line.clone()).await{
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to parse request_line: {}", e);
      *global_error_string = ERROR_400_HEADERS_FAILED_TO_PARSE.to_string();
      return;
    }
  };
  
  // Parse les entetes
  for line_index in 1..headers_lines.len() {
    let line: String = match headers_lines.get(line_index){
      Some(value) => {value.to_string()},
      None => {
        // eprintln!("ERROR: Failed to get header line");
        *global_error_string = ERROR_500_INTERNAL_SERVER_ERROR.to_string();
        return;
      },
    };
    
    if line.is_empty() { break } //expect this can be the end of headers section
    
    let parts: Vec<String> = line.splitn(2, ": ").map(|s| s.to_string()).collect();
    if parts.len() == 2 {
      let header_name = match HeaderName::from_bytes(parts[0].as_bytes()) {
        Ok(v) => v,
        Err(_e) =>{
          // eprintln!("ERROR: Invalid header name: {}\n {}", parts[0], e);
          *global_error_string = ERROR_400_HEADERS_INVALID_HEADER_NAME.to_string();
          return;
        },
      };
      
      let value = HeaderValue::from_str( parts[1].trim());
      match value {
        Ok(v) => headers.insert(header_name, v),
        Err(_e) =>{
          // eprintln!("ERROR: Invalid header value: {}\n {}", parts[1], e);
          *global_error_string = ERROR_400_HEADERS_INVALID_HEADER_VALUE.to_string();
          return;
        },
      };
      
    }
  }
  
// Construire l'objet http::Request
  *request = match Request::builder()
  .method(method)
  .uri(uri)
  .version(version)
  .body(body_buffer){
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to construct the http::Request object: {}", e);
      *global_error_string = ERROR_500_INTERNAL_SERVER_ERROR.to_string();
      return;
    }
  };
  
  // Essayer de remplir les en-têtes, car dans le constructeur, il semble qu'il n'y ait pas de méthode
// pour créer des en-têtes à partir de HeaderMap, mais peut-être que le remplacement forcé peut également être utilisé
  let request_headers = request.headers_mut();
// request_headers.clear(); // à faire : pas sûr, peut-être que certains paramètres par défaut doivent être présents
  for (key,value) in headers{
    let header_name = match key {
      Some(v) => v,
      None => {
        // eprintln!("ERROR: Invalid header name");
        *global_error_string = ERROR_400_HEADERS_INVALID_HEADER_NAME.to_string();
        return;
      },
    };
    
    request_headers.append(header_name, value);
  }
  
}

use std::str::FromStr;

use crate::{stream::errors::{ERROR_400_HEADERS_INVALID_REQUEST_LINE, ERROR_400_HEADERS_INVALID_METHOD, ERROR_400_HEADERS_INVALID_VERSION, ERROR_400_HEADERS_BUFFER_IS_EMPTY, ERROR_400_HEADERS_BUFFER_TO_STRING, ERROR_400_HEADERS_LINES_IS_EMPTY, ERROR_500_INTERNAL_SERVER_ERROR, ERROR_400_HEADERS_FAILED_TO_PARSE, ERROR_400_HEADERS_INVALID_HEADER_VALUE, ERROR_400_HEADERS_INVALID_HEADER_NAME}};
/// Parse la ligne de requête en ses composants
pub async fn parse_request_line(
  request_line: String
) -> Result<(Method, Uri, Version), Box<dyn Error>> {
  
  // append_to_file(&format!( "raw request_line: {:?}", request_line )).await;
  
  let parts:Vec<&str> = request_line.trim().split_whitespace().collect();
  if parts.len() != 3 {
    return Err(ERROR_400_HEADERS_INVALID_REQUEST_LINE.into());
  }
  
  let (method, uri, version) = (parts[0], parts[1], parts[2]);
  
  let method = match Method::from_str(method){
    Ok(v) => v,
    Err(_e) =>{
      // eprintln!("ERROR: Invalid method: {}\n {}", method, e);
      return Err(ERROR_400_HEADERS_INVALID_METHOD.into())
    },
  };
  
  let uri = match Uri::from_str(uri){
    Ok(v) => v,
    Err(_e) =>{
      // eprintln!("ERROR: Invalid uri: {}\n {}", uri, e);
      return Err(ERROR_400_HEADERS_INVALID_METHOD.into())
    }
  };
  
  if version.to_ascii_uppercase() != "HTTP/1.1" {
    // eprintln!("ERROR: Invalid version: {} . According to task requirements it must be HTTP/1.1 \"It is compatible with HTTP/1.1 protocol.\" ", version);
    return Err(ERROR_400_HEADERS_INVALID_VERSION.into());
  }
  
  Ok((method, uri, Version::HTTP_11))
  
}
