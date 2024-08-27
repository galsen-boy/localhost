#[allow(unused_assignments)]
use std::time::{Instant, Duration};

use async_std::io;
use async_std::net::TcpStream;
use futures::AsyncReadExt;

// use crate::debug::{append_to_file, DEBUG};
use crate::stream::errors::{ERROR_400_BODY_SUM_CHUNK_SIZE_READ_TIMEOUT, ERROR_400_BODY_SUM_CHUNK_SIZE_READING_STREAM, ERROR_400_BODY_SUM_CHUNK_SIZE_PARSE, ERROR_400_BODY_CHUNKED_BUT_ZERO_SUM_CHUNK_SIZE, ERROR_400_BODY_CHUNK_SIZE_READ_TIMEOUT, ERROR_400_BODY_CHUNK_SIZE_READING_STREAM, ERROR_400_BODY_CHUNK_SIZE_PARSE, ERROR_400_BODY_CHUNK_READ_TIMEOUT, ERROR_400_BODY_CHUNK_READING_STREAM, ERROR_400_BODY_CHUNK_IS_BIGGER_THAN_CHUNK_SIZE, ERROR_413_BODY_SIZE_LIMIT};



pub async fn read_chunked(
  stream: &mut TcpStream,
  body_buffer: &mut Vec<u8>,
  client_body_size: usize,
  timeout: Duration,
  global_error_string: &mut String,
) {
  
  // append_to_file(
  //   "===================\n= CHUNKED REQUEST =\n==================="
  // ).await;
  
  let start_time = Instant::now();
  
  let mut body_size = 0;
  
  let mut sum_chunk_size_buffer = Vec::new();
  
  let mut sum_chunk_size = 0;
  
  // étant donné que le délai d'attente est implémenté, ignorer la première ligne de taille de chunk qui est la somme de tous les chunks
  loop { // lire la taille de la somme des chunks
    
    // sommeil asynchrone de 2 ms, pour certains cas de append_to_file() liés au temps.
    // avec un corps volumineux, les délais de lecture peuvent facilement être dépassés. À utiliser avec précaution.
    // if DEBUG { async_std::task::sleep(Duration::from_millis(2)).await; }
    
    // Vérifier si le délai d'attente a expiré
    if start_time.elapsed() >= timeout {
      // eprintln!("ERREUR : Dépassement du délai d'attente pour la lecture de la taille totale des chunks");
      *global_error_string = ERROR_400_BODY_SUM_CHUNK_SIZE_READ_TIMEOUT.to_string();
      return
    }
    
    let mut buf = [0; 1];
    
    // Lire depuis le flux un octet à la fois
    match stream.read(&mut buf).await {
      Ok(0) => {
        // append_to_file("EOF atteint en lecture. Taille totale des chunks lue").await;
        break;
      },
      Ok(n) => { // Lecture réussie de n octets depuis le flux
        
        body_size += n;
        
        // Vérifier si la taille du corps est supérieure à client_body_size
        if body_size > client_body_size {
          // eprintln!("ERREUR : La taille du corps est supérieure à la limite client_body_size : {} > {}", body_size, client_body_size);
          *global_error_string = ERROR_413_BODY_SIZE_LIMIT.to_string();
          return
        }
        
        sum_chunk_size_buffer.extend_from_slice(&buf[..n]);
        
        // Vérifier si la fin du flux a été atteinte
        if n < buf.len() {
          // append_to_file("EOF atteint relativement.\nBuffer non complet après lecture. Taille totale des chunks lue").await;
          return // à faire : pas évident, probablement
        }
      },
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        // Le flux n'est pas encore prêt, réessayez plus tard
        continue;
      },
      Err(_e) => { // Autre erreur survenue
        // eprintln!("ERREUR : Lecture de la taille totale des chunks depuis le flux : {}", e);
        *global_error_string = ERROR_400_BODY_SUM_CHUNK_SIZE_READING_STREAM.to_string();
        return
      },
    }
    
    if sum_chunk_size_buffer.ends_with(b"\r\n") {
      
      // Analyser la taille totale des chunks
      let sum_chunk_size_str = String::from_utf8_lossy(&sum_chunk_size_buffer).trim().to_string();
      
      sum_chunk_size = match usize::from_str_radix(&sum_chunk_size_str, 16){
        Ok(v) => v,
        Err(_e) =>{
          // eprintln!("ERREUR : Échec de l'analyse de sum_chunk_size_str : {}\n {}", sum_chunk_size_str, e);
          *global_error_string = ERROR_400_BODY_SUM_CHUNK_SIZE_PARSE.to_string();
          return
        }
      };
      
      
      break;
      
    }
    
  }
  
  if sum_chunk_size == 0 {
    // eprintln!("ERREUR : Corps chunked avec une taille totale des chunks nulle");
    *global_error_string = ERROR_400_BODY_CHUNKED_BUT_ZERO_SUM_CHUNK_SIZE.to_string();
    return
  }
  
  sum_chunk_size_buffer.clear(); // maintenant plus de mémoire est en sécurité, les créateurs de Rust peuvent dormir sur leurs deux oreilles
  // ------------------------------------
  // Fin de l'ignorance de la première ligne de taille de chunk qui est la somme de tous les chunks.
  // Description au début de la boucle ci-dessus
  // ------------------------------------
  #[allow(unused_assignments)]
  let mut chunk_size = 0;
  let mut chunk_size_buffer = Vec::new();
  
  let mut chunk_buffer = Vec::new();
  
  loop { // lire la taille du chunk
    
    // sommeil asynchrone de 2 ms, pour certains cas de append_to_file() liés au temps.
    // avec un corps volumineux, les délais de lecture peuvent facilement être dépassés. À utiliser avec précaution.
    // if DEBUG { async_std::task::sleep(Duration::from_millis(2)).await; }
    
    // Vérifier si le délai d'attente a expiré
    if start_time.elapsed() >= timeout {
      // eprintln!("ERREUR : Dépassement du délai d'attente pour la lecture de la taille du chunk");
      *global_error_string = ERROR_400_BODY_CHUNK_SIZE_READ_TIMEOUT.to_string();
      return
    }
    
    let mut buf = [0; 1];
    
    // Lire depuis le flux un octet à la fois
    match stream.read(&mut buf).await {
      Ok(0) => {
        // EOF atteint
        // append_to_file("EOF atteint en lecture. Taille du chunk lue").await;
        break;
      },
      Ok(n) => {
        
        body_size += n;
        
        // Vérifier si la taille du corps est supérieure à client_body_size
        if body_size > client_body_size {
          // eprintln!("ERREUR : La taille du corps est supérieure à la limite client_body_size : {} > {}", body_size, client_body_size);
          *global_error_string = ERROR_413_BODY_SIZE_LIMIT.to_string();
          return
        }
        
        // Lecture réussie de n octets depuis le flux
        chunk_size_buffer.extend_from_slice(&buf[..n]);
        
        // Vérifier si la fin du flux a été atteinte
        if n < buf.len() {
          // append_to_file("EOF atteint relativement.\nBuffer non complet après lecture. Taille du chunk lue").await;
          return // à faire : pas évident, probablement
        }
      },
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        // Le flux n'est pas encore prêt, réessayez plus tard
        continue;
      },
      Err(_e) => {
        // Autre erreur survenue
        // eprintln!("ERREUR : Lecture de la taille du chunk depuis le flux : {}", e);
        *global_error_string = ERROR_400_BODY_CHUNK_SIZE_READING_STREAM.to_string();
        return
      },
    }
    
    // Vérifier si la fin de la taille du chunk a été atteinte
    if chunk_size_buffer.ends_with(b"\r\n") {
      // Analyser la taille du chunk
      let chunk_size_str = String::from_utf8_lossy(&chunk_size_buffer).trim().to_string();
      chunk_size = match usize::from_str_radix(&chunk_size_str, 16){
        Ok(v) => v,
        Err(_e) =>{
          // eprintln!("ERREUR : Échec de l'analyse de chunk_size_str : {}\n {}", chunk_size_str, e);
          *global_error_string = ERROR_400_BODY_CHUNK_SIZE_PARSE.to_string();
          return
        }
      };
      // append_to_file(&format!("chunk_size : {}", chunk_size)).await;
      
      
      // Vérifier si la fin du flux a été atteinte
      if chunk_size == 0 {
        // append_to_file("EOF atteint pour le corps chunked").await;
        return
      } else { // il y a un chunk à lire, selon chunk_size
        
        loop { // lire le chunk
          
          // sommeil asynchrone de 2 ms, pour certains cas de append_to_file() liés au temps.
          // avec un corps volumineux, les délais de lecture peuvent facilement être dépassés. À utiliser avec précaution.
          // if DEBUG { async_std::task::sleep(Duration::from_millis(2)).await; }
          
          // Vérifier si le délai d'attente a expiré
          if start_time.elapsed() >= timeout {
            // eprintln!("ERREUR : Dépassement du délai d'attente pour la lecture du corps du chunk");
            *global_error_string = ERROR_400_BODY_CHUNK_READ_TIMEOUT.to_string();
            return
          }
          
          let mut buf = [0; 1];
          
          // Lire depuis le flux un octet à la fois
          match stream.read(&mut buf).await {
            Ok(0) => {
              // append_to_file("EOF atteint en lecture").await;
              break;
            },
            Ok(n) => {
              
              body_size += n;
              
              // Vérifier si la taille du corps est supérieure à client_body_size
              if body_size > client_body_size {
                // eprintln!("ERREUR : La taille du corps est supérieure à la limite client_body_size : {} > {}", body_size, client_body_size);
                *global_error_string = ERROR_413_BODY_SIZE_LIMIT.to_string();
                return
              }
              
              // Lecture réussie de n octets depuis le flux
              chunk_buffer.extend_from_slice(&buf[..n]);
              
              // Vérifier si la fin du flux a été atteinte
              if n < buf.len() {
                // append_to_file("EOF atteint relativement, buffer non complet après lecture. Chunk lu").await;
                return // à faire : pas évident, probablement
              }
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
              // Le flux n'est pas encore prêt, réessayez plus tard
              continue;
            },
            Err(_e) => {
              // Autre erreur survenue
              // eprintln!("ERREUR : Lecture du chunk depuis le flux : {}", e);
              *global_error_string = ERROR_400_BODY_CHUNK_READING_STREAM.to_string();
              return
            },
          }
          
          #[allow(unused_assignments)]
          // Vérifier si la fin du chunk a été atteinte
          if chunk_buffer.ends_with(b"\r\n") {
            // Supprimer le CRLF final
            // append_to_file(&format!("avant la troncature de chunk_buffer : {:?}", chunk_buffer)).await;
            chunk_buffer.truncate(chunk_buffer.len() - 2);
            // append_to_file(&format!("chunk_buffer : {:?}", chunk_buffer)).await;
            body_buffer.extend(chunk_buffer.clone());
            
            chunk_buffer.clear();
            chunk_size_buffer.clear();
            chunk_size = 0;
            break;
          }
          else if chunk_buffer.len() > chunk_size + 2 // à faire : pas évident, probablement
          { // le chunk est cassé, car il est plus grand que chunk_size
            // eprintln!("ERREUR : Le chunk est plus grand que la taille du chunk");
            *global_error_string = ERROR_400_BODY_CHUNK_IS_BIGGER_THAN_CHUNK_SIZE.to_string();
            return
          }
          
        }
        
      }
      
    }
    
  }
  
}
