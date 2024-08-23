use std::time::{Instant, Duration};
use std::io::{self};

use futures::AsyncReadExt;
use async_std::net::TcpStream;

use crate::debug::append_to_file;
use crate::server::core::ServerConfig;
use crate::server::find::server_config_from_headers_buffer_or_use_default;
use crate::stream::errors::{ERROR_400_HEADERS_READ_TIMEOUT, ERROR_400_HEADERS_READING_STREAM, ERROR_400_HEADERS_FAILED_TO_PARSE, ERROR_500_INTERNAL_SERVER_ERROR};
use crate::stream::read_chunked::read_chunked;
use crate::stream::read_unchunked::read_unchunked;

/// Lire depuis le flux jusqu'au délai d'attente ou EOF
/// 
/// Retourne un tuple de deux vecteurs : (headers_buffer, body_buffer)
pub async fn read_with_timeout(
  timeout: Duration,
  stream: &mut TcpStream,
  headers_buffer: &mut Vec<u8>,
  body_buffer: &mut Vec<u8>,
  server_configs: &Vec<ServerConfig>,
  global_error_string: &mut String,
) -> ServerConfig {

  append_to_file("\nINSIDE read_with_timeout").await;
  
  // Demarre le timer
  let start_time = Instant::now();
  
// Lire depuis le flux jusqu'au délai d'attente ou à la fin du fichier (EOF)
  let mut buf = [0; 1];
  
// Collecter la section des en-têtes de la requête  
  loop {
    
// Vérifier si le délai d'attente a expiré
    if start_time.elapsed() >= timeout {
      // eprintln!("ERROR: Headers read timed out");
      *global_error_string = ERROR_400_HEADERS_READ_TIMEOUT.to_string();
      return server_configs[0].clone();
    }
    
    match stream.read(&mut buf).await {
      Ok(0) => {
        append_to_file("read EOF reached").await;
        break;
      },
      Ok(n) => {
        // Lecture réussie de n octets depuis le flux
          headers_buffer.extend_from_slice(&buf[..n]);
        
        // Vérifier si la fin du flux a été atteinte
          if n < buf.len() {
          append_to_file("read EOF reached relatively, because buffer not full after read").await;
          break;
        }
      },
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
            // Le flux n'est pas encore prêt, réessayez plus tard
            continue;
      },
      Err(_e) => {
        // eprintln!("ERROR: Reading headers from stream: {}", e);
        *global_error_string = ERROR_400_HEADERS_READING_STREAM.to_string();
        return server_configs[0].clone();
      },
    }
    
    
    if headers_buffer.ends_with(b"\r\n\r\n") {
      append_to_file("HEADERS BUFFER ENDS WITH \\r\\n\\r\\n").await;
      break;
    }
  }
  
  append_to_file(&format!(
    "HEADERS_BUFFER_STRING:\n{:?}",
    String::from_utf8(headers_buffer.clone())
  )).await;
  
// Choisir la configuration du serveur et vérifier la taille du corps de la requête (client_body_size) dans server_config
// Retourner une erreur 413 si le corps est trop volumineux.
// Fragment de code dupliqué en raison des exigences étranges de la tâche.
  
  let server_config = server_config_from_headers_buffer_or_use_default(
    headers_buffer,
    server_configs.clone()
  ).await;
  
// Vérifier la longueur du corps, selon server_config.client_body_size.
  let client_body_size = server_config.client_body_size;
  
  let dirty_string = String::from_utf8_lossy(&headers_buffer);
  let is_chunked = dirty_string.to_lowercase().contains("transfer-encoding: chunked");
  
  let has_content_length_header =
  dirty_string.to_lowercase().contains("content-length: ");

  if !has_content_length_header && !is_chunked {
    append_to_file(&format!("Neither Content-Length nor Transfer-Encoding: chunked headers found in headers_buffer. Skip body reading.")).await;
    return server_config;
  }
  
  let mut content_length = 0;
  
  if has_content_length_header { 
    let index = match dirty_string.to_lowercase().find("content-length: "){
      Some(v) => v,
      None => {
        // eprintln!("ERROR: [500] Failed to find already confirmed content-length header in headers_buffer");
        *global_error_string = ERROR_500_INTERNAL_SERVER_ERROR.to_string();
        return server_config;
      }
    };

    let start = index + "content-length: ".len();

    let end = match dirty_string[start..].find("\r\n"){
      Some(v) => v,
      None => {
        // eprintln!("ERROR: [500] Failed to find the end( \"\\r\\n\" ) of already confirmed content-length header in headers_buffer");
        *global_error_string = ERROR_500_INTERNAL_SERVER_ERROR.to_string();
        return server_config;
      }
    };

    content_length = match dirty_string[start..start + end].trim().parse(){
      Ok(v) => v,
      Err(_e) => {
        // eprintln!("ERROR: Failed to parse already confirmed content-length header in headers_buffer: \n{}", e);
        *global_error_string = ERROR_400_HEADERS_FAILED_TO_PARSE.to_string();
        return server_config;
      }
    };
    
  }

  append_to_file(&format!("is_chunked: {}", is_chunked)).await;
  append_to_file(&format!("has_content_length_header: {}", has_content_length_header)).await;
  append_to_file(&format!("content_length: {}", content_length)).await;
  append_to_file(&format!("====\nstream: {:?}\nbefore dive into read body", stream)).await;
  
// Collecter la section du corps de la requête
if is_chunked {
    read_chunked(
      stream,
      body_buffer,
      client_body_size,
      timeout,
      global_error_string,
    ).await;
  } else if content_length > 0  {
    read_unchunked(
      stream,
      body_buffer,
      client_body_size,
      content_length,
      timeout,
      global_error_string,
    ).await;
  }
  
  server_config
  
}
