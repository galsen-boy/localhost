use async_std::fs;
use async_std::path::PathBuf;
use async_std::stream::StreamExt; // for `next`

use http::{Request, Response, StatusCode, HeaderValue};

// use crate::debug::append_to_file;
use crate::files::check::bad_file_name;
use crate::handlers::response_::response_default_static_file;
use crate::handlers::response_4xx::custom_response_4xx;
use crate::handlers::response_500::custom_response_500;
use crate::server::core::ServerConfig;
use crate::stream::errors::{ERROR_200_OK, ERROR_500_INTERNAL_SERVER_ERROR};


/// html is generated in code. Not templates etc.
/// 
/// To decrease dependencies and avoid any extra activities.
pub async fn generate_uploads_html(absolute_path: &PathBuf) -> (String, String) {
  let mut html = String::new();
  
  html.push_str("<!DOCTYPE html>\n");
  html.push_str("<html lang=\"en\">\n");
  html.push_str("<head>\n");
  html.push_str(" <meta charset=\"UTF-8\">\n");
  html.push_str(" <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
  html.push_str(" <title>Uploads</title>\n");
  
  // add improved styles
  let style = r#"
<style>
body {
  font-family: Arial, sans-serif;
  line-height: 1.6;
  color: #333;
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
  background-color: #f4f4f4;
}

h1 {
  text-align: center;
  font-size: 2.5em;
  color: #2c3e50;
  margin-bottom: 30px;
}

a {
  color: #3498db;
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}

ul {
  list-style-type: none;
  padding: 0;
}

li {
  background-color: #fff;
  margin-bottom: 10px;
  padding: 15px;
  border-radius: 5px;
  box-shadow: 0 2px 5px rgba(0,0,0,0.1);
}

button {
  background-color: #e74c3c;
  color: white;
  border: none;
  padding: 5px 10px;
  margin-right: 10px;
  cursor: pointer;
  border-radius: 3px;
}

button:hover {
  background-color: #c0392b;
}

form {
  margin-top: 20px;
  background-color: #fff;
  padding: 20px;
  border-radius: 5px;
  box-shadow: 0 2px 5px rgba(0,0,0,0.1);
}

input[type="file"] {
  margin-bottom: 10px;
}

input[type="submit"] {
  background-color: #2ecc71;
  color: white;
  border: none;
  padding: 10px 15px;
  cursor: pointer;
  border-radius: 3px;
}

input[type="submit"]:hover {
  background-color: #27ae60;
}
</style>
"#;
  html.push_str(&style);
  
  html.push_str("</head>\n");
  html.push_str("<body>\n");
  
  html.push_str("<h1>Uploads</h1>");
  
  // add link back to main page
  html.push_str("\n<a href=\"/\">Back to main page</a>");
  
  // list of files
  html.push_str("<ul>");
  
  let mut entries = match fs::read_dir(absolute_path).await {
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to read uploads folder: {}", e);
      return (html, ERROR_500_INTERNAL_SERVER_ERROR.to_string());
    },
  };
  
  while let Some(entry) = entries.next().await {
    
    let entry = match entry {
      Ok(v) => v,
      Err(_e) => {
        // eprintln!("ERROR: Failed to read uploads folder entry: {}\nSKIPPED", e);
        continue;
      }
    };
    
    let path = entry.path();
    if path.is_file().await {
      
      let file_name = match path.file_name() {
        Some(v) => v,
        None => {
          // eprintln!("ERROR: Failed to get file name from path: {:?}\nSKIPPED", path);
          continue;
        }
      };
      
      let file_name_str = match file_name.to_str() {
        Some(v) => v,
        None => {
          // eprintln!("ERROR: Failed to convert file name to str: {:?}\nSKIPPED", file_name);
          continue;
        }
      };
      
      if bad_file_name(file_name_str) {
        // eprintln!("ERROR: Bad file name \"{}\" inside \"uploads\" folder.\nPotential crappers activity :|,\nor file name was sanitised not properly\nin time of uploading\nSKIPPED\n", file_name_str);
        continue;
      }
      if file_name_str == ".gitignore" { continue; }
      
      html.push_str("\n<li>");
      html.push_str(&format!("\n<button onclick=\"deleteFile('{}')\">Delete</button>", file_name_str));
      html.push_str(&format!("\n<a href=\"/uploads/{}\">{}</a>", file_name_str, file_name_str));
      html.push_str("\n</li>");
      
    }
    
  }
  
  html.push_str("\n</ul>");
  
  let form = r#"
  <form method="POST" action="/uploads" id="uploadForm" enctype="multipart/form-data">
  <input type="file" name="file" id="fileInput">
  <input type="submit" value="Upload">
  </form>
  "#;
  
  html.push_str(&form);
  
  let script = r#"
  <script>
  function deleteFile(fileName) {
    fetch('/uploads', {
      method: 'DELETE',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: 'file=' + encodeURIComponent(fileName),
      redirect: 'manual'
    }).then(response => {
      console.log(response.status);
      if (!response.ok) {
        if (response.status === 400) {
          console.log('400 crap piles from 01 delivered');
          window.location.href = '400.html';
        }
        else if (response.status === 403) {
          console.log('403 crap piles from 01 delivered');
          window.location.href = '403.html';
        }
        else if (response.status === 404) {
          console.log('404 crap piles from 01 delivered');
          window.location.href = '404.html';
        }
        else if (response.status === 405) {
          console.log('405 Method Not Allowed');
          window.location.href = '405.html';
        }
        else if (response.status === 413) {
          console.log('413 crap piles from 01 delivered');
          window.location.href = '413.html';
        }
        else {
          console.log('500 crap piles from 01 delivered');
          window.location.href = '500.html';
        }
      } else {
        setTimeout(function() {
          location.reload();
        }, 1000);
      }
    }).catch(error => {
      console.error('Error:', error);
    });
  }
  document.getElementById('uploadForm').addEventListener('submit', function(event) {
    event.preventDefault();
    const fileInput = document.getElementById('fileInput');
    const file = fileInput.files[0];
    const reader = new FileReader();
    reader.readAsArrayBuffer(file);
    reader.onloadend = function() {
      const arrayBuffer = reader.result;
      const uint8Array = new Uint8Array(arrayBuffer);
      fetch('/uploads', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/octet-stream',
          'X-File-Name': file.name
        },
        body: uint8Array,
        redirect: 'manual'
      }).then(response => {
        console.log(response.status);
        if (!response.ok) {
          if (response.status === 400) {
            console.log('400 crap piles from 01 delivered');
            window.location.href = '400.html';
          }
          else if (response.status === 403) {
            console.log('403 crap piles from 01 delivered');
            window.location.href = '403.html';
          }
          else if (response.status === 404) {
            console.log('404 crap piles from 01 delivered');
            window.location.href = '404.html';
          }
          else if (response.status === 405) {
            console.log('405 Method Not Allowed');
            window.location.href = '405.html';
          }
          else if (response.status === 413) {
            console.log('413 crap piles from 01 delivered');
            window.location.href = '413.html';
          }
          else {
            console.log('500 crap piles from 01 delivered');
            window.location.href = '500.html';
          }
        } else {
          setTimeout(function() {
            location.reload();
          }, 1000);
        }
      }).catch(error => {
        console.error('Error:', error);
      });
    };
  });
  </script>
  "#;
  
  html.push_str(&script);
  html.push_str("</body>\n");
  html.push_str("</html>\n");
  
  (html, ERROR_200_OK.to_string())
}


/// Lorsqu'un fichier est demandé depuis le dossier de téléchargements,
/// 
/// cette fonction gère le traitement de cette demande.
/// 
/// Géré séparément, car le dossier de téléchargements est dynamique. Pas sûr à utiliser.
pub async fn handle_uploads_get_uploaded_file(
  request: &Request<Vec<u8>>,
  cookie_value:String,
  zero_path_buf: &PathBuf,
  file_path: String,
  server_config: ServerConfig,
) -> Response<Vec<u8>>{
  
// Analyser le chemin. Si le chemin est un répertoire, retourner le fichier par défaut, selon la configuration du serveur
  let mut path_str = request.uri().path();
  //enleve le premeier slash s'il debute le path
  if path_str.starts_with("/"){ path_str = &path_str[1..]; }
  
  let absolute_path_buf = zero_path_buf.join("uploads").join(file_path);
  
// Vérifier si le chemin est un répertoire, puis retourner le fichier par défaut comme l'exige la tâche
if path_str.ends_with("/") || absolute_path_buf.is_dir().await {
    
// Implémenter la vérification d'erreur 403 si la méthode n'est pas GET, pour satisfaire les exigences de la tâche
if request.method().to_string() != "GET" {
      // eprint!("ERROR: Status code 403 FORBIDDEN. CUSTOM IMPLEMENTATION.\nOnly the \"GET\" method is allowed to access the directory.");
      return custom_response_4xx(
        request,
        cookie_value,
        zero_path_buf,
        server_config,
        StatusCode::FORBIDDEN,
      ).await
    }
    
    return response_default_static_file(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
    ).await
  } else if !absolute_path_buf.is_file().await {
    
    // eprintln!("ERROR:\n-----------------------------------\nuploads absolute_path IS NOT A FILE \n-----------------------------------"); // todo: remove dev print
    
    return custom_response_4xx(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
      StatusCode::NOT_FOUND,
    ).await
  } // verifie si le fichier existe sinon genere une erreur 404
  
// Seule la méthode GET est autorisée pour ce chemin. Le filtrage se fait plus haut
  let allowed_methods = vec!["GET".to_string()];
  
// Vérifier si la méthode est autorisée pour ce chemin ou retourner 405
  let request_method_string = request.method().to_string();
  if !allowed_methods.contains(&request_method_string){
    // eprintln!("ERROR: Method {} is not allowed for path {}", request_method_string, path_str);
    return custom_response_4xx(
      request,
      cookie_value,
      zero_path_buf,
      server_config,
      http::StatusCode::METHOD_NOT_ALLOWED,
    ).await
  }
  
// Lire le fichier. En cas d'erreur, retourner une réponse d'erreur 500
  let file_content = match std::fs::read(absolute_path_buf.clone()){
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to read file: {}", e);
      return custom_response_500(
        request,
        cookie_value,
        zero_path_buf,
        server_config
      ).await
    }
  };
  
  let mut response = match Response::builder()
  .status(StatusCode::OK)
  .header("Set-Cookie", cookie_value.clone())
  .body(file_content)
  {
    Ok(v) => v,
    Err(_e) => {
      // eprintln!("ERROR: Failed to create response with file: {}", e);
      return custom_response_500(
        request,
        cookie_value.clone(),
        zero_path_buf,
        server_config
      ).await
    }
  };
  
// Obtenir le type MIME du fichier en utilisant mime_guess, ou utiliser text/plain
let mime_type = match mime_guess::from_path(absolute_path_buf.clone()).first(){
    Some(v) => v.to_string(),
    None => "text/plain".to_string(),
  };
  
  // append_to_file(&format!(
  //   "\n-------\n\nmime_type {}\n\n----------\n", mime_type
  // )).await;
  
  response.headers_mut().insert(
    "Content-Type",
    match mime_type.parse(){
      Ok(v) => v,
      Err(_e) => {
        // eprintln!("ERROR: Failed to parse mime type: {}", e);
        HeaderValue::from_static("text/plain")
      }
    }
  );
  
  response
  
}
