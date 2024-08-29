## Description
Un serveur web Rust, compatible avec le protocole HTTP/1.1, capable d'exécuter des scripts CGI Python3.

## Utilisation (Linux)
- Ouvrez un terminal dans le dossier racine du dépôt (à l'emplacement du fichier `README.md`)

### Construire le projet :
- Terminal : `./build`

### Exécuter le projet :
- Terminal : `./runme`

### Exécuter le projet en mode développement :
- Terminal : `./devrun`

## Fonctionnement du serveur :

- Après avoir construit le projet, le fichier binaire `runme` sera créé dans le dossier racine du dépôt.
- Après avoir exécuté `./runme`, le serveur essaiera de démarrer, selon le fichier `settings` qui suit le format TOML. Si une erreur se produit lors de l'initialisation, le serveur s'arrêtera et affichera le message d'erreur dans le terminal.
- Après le démarrage, le serveur écoutera les configurations ip:port à partir du fichier `settings`, qui seront imprimées dans le terminal, comme les instances `Server`.

### Détails et restrictions :

- `error 403 Forbidden`, la gestion est mise en œuvre pour le cas d'un accès avec un URI de répertoire avec une méthode autre que GET, comme un moyen de faire respecter que seules les requêtes GET sont autorisées pour les URI de répertoire. Ce code d'état est couramment utilisé lorsque le serveur ne souhaite pas révéler exactement pourquoi la requête a été refusée ou lorsqu'aucune autre réponse n'est applicable.
Pour le tester, vous pouvez utiliser les commandes `curl` dans le terminal.

Cas correct en utilisant la méthode GET (renvoie le fichier par défaut comme l'exige la tâche) :
`curl -X GET http://localhost:8080/`

Cas interdit en utilisant la méthode POST (renvoie la page d'erreur 403) :
`curl -X POST http://localhost:8080/`

- La fonctionnalité `cgi` est implémentée dans les gestionnaires séparément, car il s'agit d'une ancienne technologie peu sûre, qu'il n'est pas recommandé d'utiliser.
Selon les exigences de la tâche, un seul script suffit à être implémenté, et un lien vers celui-ci est codé en dur dans le fichier `runme`, pour empêcher toute activité/expérimentation supplémentaire.
- La fonctionnalité `uploads` est implémentée dans les gestionnaires séparément et contrôlée à l'aide d'un paramètre séparé dans le fichier `settings`, pour empêcher toute activité/expérimentation supplémentaire.
Selon la tâche, le serveur doit gérer sans problème un site statique.
La fonctionnalité de téléchargement et de suppression de fichiers ne fait pas partie du site statique, elle est donc implémentée séparément, de manière universelle pour tous les sites, et codée en dur dans le fichier `runme`.
Le fichier `settings` permet de contrôler l'accessibilité de la page `/uploads` pour chaque paramétrage du serveur, en utilisant les méthodes `GET`, `POST`, `DELETE` pour les autorisations de téléchargement, de téléversement et de suppression respectivement.
- En cas d'utilisation de "paires dupliquées pour ip:port dans les paramètres", dans le cadre de différentes configurations, la configuration ajoutée ensuite dans le fichier `settings` remplacera la précédente.
- La configuration par défaut du fichier `settings` met en œuvre trois sites différents, avec la possibilité de tester l'accessibilité de la page `redirect.html`, en fonction des méthodes autorisées, de la commande `curl --resolve`, et un site pour tester la page `empty.html` à l'aide de l'utilitaire de test de charge `siege`.
- Pour garder le flux plus stable, le corps de la requête sera ignoré si l'en-tête de la requête non chunked ne contient pas l'en-tête `Content-Length`. La plupart des clients ajoutent automatiquement cet en-tête, donc ce n'est pas un problème.


## Audit et tests
Pour satisfaire l'exigence de la description de la tâche
> Vous pouvez utiliser le langage de votre choix pour écrire les tests, à condition qu'ils soient exhaustifs et que l'auditeur puisse en vérifier le comportement.
la démarche suivante a été choisie :
- le processus de test est manuel
- les outils utilisés pour les tests sont les utilitaires `curl` et `siege`, ainsi que le navigateur
- la description de l'audit et des tests se trouve dans le fichier `audit.md` dans le dossier racine du dépôt
[audit.md](audit.md)

## Auteurs