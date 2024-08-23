use async_std::sync::Mutex;
use async_std::task;
use futures::AsyncWriteExt;
use async_std::net::TcpListener;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::net::SocketAddr;
use async_std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use http::{Response, Request};
use std::error::Error;

use crate::handlers::response_::check_custom_errors;
use crate::handlers::handle_::handle_request;
use crate::server::cookie::Cookie;
use crate::server::core::{get_usize_unique_ports, Server};
use crate::server::core::ServerConfig;
use crate::stream::errors::{ERROR_200_OK, ERROR_400_HEADERS_INVALID_COOKIE};
use crate::stream::read_::read_with_timeout;
use crate::stream::parse::parse_raw_request;
use crate::stream::write_::write_response_into_stream;
use crate::debug::append_to_file;

pub async fn run(
  zero_path_buf:PathBuf,
  server_configs: Vec<ServerConfig>
) -> Result<(), Box<dyn Error>> {

  let ports = match get_usize_unique_ports(&server_configs).await{
    Ok(ports) => ports,
    Err(_e) => {
      // eprintln!("ERROR: Échec de l'obtention des ports: {}", e);
      return Err("Échec de l'obtention des ports".into());
    },
  };
  
  let server_address = "0.0.0.0"; // pour écouter toutes les interfaces
  
  for port in ports.clone() {
    let addr: SocketAddr = match format!(
      "{}:{}",
      server_address,
      port,
    ).parse(){
      Ok(v) => v,
      Err(_e) => {
        // eprintln!("ERROR: Échec de l'analyse de 0.0.0.0:port en SocketAddr: {}", e);
        return Err("Échec de l'analyse de 0.0.0.0:port en SocketAddr".into());
      },
    };
    
    let listener = match TcpListener::bind(addr).await{
      Ok(v) => v,
      Err(_e) => {
        // eprintln!("ERROR: Échec de la liaison à addr: {}", e);
        return Err("Échec de la liaison à addr".into());
      },
    };
    
    append_to_file(&format!("addr {}", addr)).await;
    
    let zero_path_buf = zero_path_buf.clone();
    let server_configs = server_configs.clone();
    
    // Créer un flux infini de connexions entrantes pour chaque port
    task::spawn(async move {

      // peut également être un pour toutes les tâches (déplacer à l'extérieur), mais cela semble plus sûr/isolé
      let cookies_storage: Arc<Mutex<HashMap<String, Cookie>>> =
        Arc::new(Mutex::new(HashMap::new()));
      
      listener.incoming().for_each_concurrent(None, |stream| async {
        
        let mut stream = match stream{
          Ok(v) => v,
          Err(_e) => {
            // eprintln!("ERROR: Échec de l'obtention du flux: {}", e);
            return;
          },
        };
        
        append_to_file(
          "==================\n= incoming fires =\n=================="
        ).await;
        // append_to_file(&format!("{:?}",stream)).await;
        
        let mut server = Server {
          cookies: cookies_storage.clone(),
          cookies_check_time: SystemTime::now() + Duration::from_secs(60), };
        
        let mut headers_buffer: Vec<u8> = Vec::new();
        let mut body_buffer: Vec<u8> = Vec::new();
        let mut global_error_string = ERROR_200_OK.to_string();
        
        append_to_file(&format!( "\nbefore read_with_timeout\nheaders_buffer: {:?}", headers_buffer )).await;

        let mut response:Response<Vec<u8>> = Response::new(Vec::new());
        
        // codé en dur, mais c'est correct pour ce cas. Et moins de chances pour l'utilisateur de le casser.
        // Pas mal de le gérer comme un flag de l'exécutable.
        let timeout = Duration::from_millis(30000);

        let choosen_server_config = read_with_timeout(
          timeout, &mut stream, &mut headers_buffer, &mut body_buffer,
          &server_configs, &mut global_error_string
        ).await;
        
        append_to_file(&format!(
          "\nafter read_with_timeout\nheaders_buffer_string: {:?}\nbody_buffer_string: {:?}" ,
          String::from_utf8(headers_buffer.clone()),
          String::from_utf8(body_buffer.clone())
        )).await;
        
        let mut request = Request::new(Vec::new());
        if global_error_string == ERROR_200_OK.to_string() {
          parse_raw_request(headers_buffer, body_buffer, &mut request, &mut global_error_string).await;
        }
        
        append_to_file(&format!(
          "\nafter parse_raw_request\nrequest.headers: {:?}\n" ,
          request.headers()
        )).await;

        server.check_expired_cookies().await;
        
        let (cookie_value, cookie_is_ok) = server.extract_cookies_from_request_or_provide_new(&request).await;
        
        if !cookie_is_ok { global_error_string = ERROR_400_HEADERS_INVALID_COOKIE.to_string(); }
        
        if global_error_string == ERROR_200_OK.to_string() {
          response = handle_request(&request, cookie_value.clone(), &zero_path_buf, choosen_server_config.clone(), &mut global_error_string).await;
        }

        append_to_file(&format!(
          "\nafter handle_request\nresponse.headers: {:?}\n" ,
          response.headers()          
        )).await;
        
        check_custom_errors(global_error_string, &request, cookie_value.clone(), &zero_path_buf, choosen_server_config.clone(), &mut response).await;
        
        match write_response_into_stream(&mut stream, response).await{
          Ok(_) => {},
          Err(_e) =>{
            // eprintln!("ERROR: Échec de l'écriture de la réponse dans le flux: {}", e);
            return // pour forcer l'arrêt du flux de tâche, juste au cas. Il se fermera de toute façon
          },
        };
        
        match stream.flush().await{
          Ok(_) => {},
          Err(_e) => {
            // eprintln!("ERROR: Échec de l'effacement du flux: {}", e);
            return // pour forcer l'arrêt du flux de tâche, juste au cas. Il se fermera de toute façon
          },
        };
        
        match stream.shutdown(std::net::Shutdown::Both){
          Ok(_) => {},
          Err(_e) => {
            // eprintln!("ERROR: Échec de l'arrêt du flux: {}", e);
            return // pour forcer l'arrêt du flux de tâche, juste au cas. Il se fermera de toute façon
          },
        };
        
      }).await;
    });
    
  }
  println!("Server Listenning... ( http://localhost:8080 )");
  async_std::task::sleep(Duration::from_secs(u64::MAX)).await;
  Ok(())
}
