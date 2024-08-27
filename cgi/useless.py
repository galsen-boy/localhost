import os
import sys

# Récupère la variable PATH_INFO
path_info = os.environ.get('PATH_INFO', '')
print("PATH_INFO: " + path_info)

# Le fichier à traiter est le premier argument du script
file_to_process = sys.argv[1]

# Combinez l'emplacement du script avec PATH_INFO pour obtenir le chemin complet
full_path = os.path.join(path_info, "cgi", file_to_process)

# Vérifiez si le chemin_complet est un fichier, un dossier ou s'il n'existe pas
if os.path.isfile(full_path):
   print("The \""+ full_path +"\" is File")
elif os.path.isdir(full_path):
   print("The \""+ full_path +"\" is Folder")
else:
   print("The \""+ full_path +"\" is Wrong path")
