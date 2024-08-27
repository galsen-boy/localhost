use std::time::{Instant, Duration};

use async_std::io;
use async_std::net::TcpStream;
use futures::AsyncReadExt;

// use crate::debug::{append_to_file, DEBUG};
use crate::stream::errors::{ERROR_400_BODY_BUFFER_LENGHT_IS_BIGGER_THAN_CONTENT_LENGTH, ERROR_400_BODY_READ_TIMEOUT, ERROR_400_BODY_READING_STREAM, ERROR_413_BODY_SIZE_LIMIT};


pub async fn read_unchunked(
  stream: &mut TcpStream,
  body_buffer: &mut Vec<u8>,
  client_body_size: usize,
  content_length: usize,
  timeout: Duration,
  global_error_string: &mut String,
) {
  
  // append_to_file(
  //   "=======================\n= NOT CHUNKED REQUEST =\n======================="
  // ).await;
  
  // append_to_file(&format!("\nstream: {:?}\ninside read body", stream)).await;
  
  // Démarrer le chronomètre pour la lecture du corps
    let start_time = Instant::now();
  
  if content_length > client_body_size {
    // eprintln!("ERROR: Content-Length header value is greater than client_body_size limit: {} > {}", content_length, client_body_size);
    *global_error_string = ERROR_413_BODY_SIZE_LIMIT.to_string();
    return
  }
  
  loop{
   // Pause asynchrone de 2 ms, pour certains cas d'impression avec append_to_file() liés au temps.
// Avec un corps volumineux, cela peut facilement provoquer des dépassements de délai de lecture. À utiliser avec prudence.
    // if DEBUG { async_std::task::sleep(Duration::from_millis(2)).await; }
    
    // verifie le temps d'expiration
    if start_time.elapsed() >= timeout {
      // eprintln!("ERROR: Body read timed out");
      // append_to_file("ERROR: Body read timed out").await;
      *global_error_string = ERROR_400_BODY_READ_TIMEOUT.to_string();
      return 
    } else {
      // append_to_file(&format!("body_buffer.len(): {}", body_buffer.len())).await;
      // append_to_file(&format!("time {} < timeout {}", start_time.elapsed().as_millis(), timeout.as_millis())).await;
    }
    
    let mut buf = [0; 1024];
    
// Lire depuis le flux un octet à la fois
    match stream.read(&mut buf).await {
      Ok(0) => {
        // append_to_file("read EOF reached. Read unchunked body size").await;
        return
      },
      Ok(n) => {
        // Lecture réussie de n octets depuis le flux
        // println!("tentative de lecture de {} octets depuis le flux", n);
        body_buffer.extend_from_slice(&buf[..n]);
        
// Vérifier la longueur du body_buffer
        if body_buffer.len() > content_length{
          // eprintln!("ERROR: body_buffer.len() > content_length");
          *global_error_string = ERROR_400_BODY_BUFFER_LENGHT_IS_BIGGER_THAN_CONTENT_LENGTH.to_string();
          return 
        }
        
// Vérifier si la fin du flux a été atteinte
        if n < buf.len() {
          // append_to_file("read EOF reached relatively, because buffer not full after read").await;
          return
        } else if body_buffer.len() == content_length{
          return
        }
      },
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        // Stream is not ready yet, try again later
        continue;
      },
      Err(_e) => {
        // Other error occurred
        // eprintln!("ERROR: Reading body from stream: {}", e);
        *global_error_string = ERROR_400_BODY_READING_STREAM.to_string();
        return
      },
    }
    
  }
  
}
