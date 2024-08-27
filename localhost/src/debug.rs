#![allow(dead_code)]use std::{fs::{File, OpenOptions, create_dir_all}, io::{self, Write}};

// pub const DEBUG: bool = false; //set to false to disable debug.txt stuff

// pub const DEBUG_FILE: &str = "debug.txt";

pub async fn try_recreate_file_according_to_value_of_debug_boolean() -> io::Result<()> {
//   if DEBUG { File::create(DEBUG_FILE)?; }
//   Ok(())
// }

// Fonction pour ajouter les donnees dans un fichier
// pub async fn append_to_file(data: &str){
//   if DEBUG {
//     let file = match OpenOptions::new()
//     .create(true)
//     .append(true)
//     .open(DEBUG_FILE){
//       Ok(file) => file,
//       Err(_e) => {
//         // eprintln!("ERROR: DEBUG: Problem opening debug file: {:?}", e);
//         return
//       }
//     };
    
//     let mut writer = io::BufWriter::new(file);
//     match writeln!(writer, "{}", data){
//       Ok(_) => {},
//       Err(_e) => {
//         // eprintln!("ERROR: DEBUG: Problem writing to debug file: {:?}", e)
//       },
//     };
//   }

// }

/// Créer un fichier nommé "something" dans le dossier uploads,
/// 
/// pour permettre à l'utilisateur de supprimer ce fichier en utilisant la méthode DELETE
/// 
/// comme l'exige la question d'audit
pub async fn create_something_in_uploads_folder() -> io::Result<()> {
  // println!("Check the \"uploads\" folder, on executable folder level");
  create_dir_all("uploads")?;
  File::create("uploads/something")?; 
  Ok(())
}
}
