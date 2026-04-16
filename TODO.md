* [x] Hacer que use YAML en lugar de json en los archivos de secrets/collections, usando OpenCollection YAML
* [x] Implementar soporte a workflows de Endpoints, igualmente con YAML, que se guarden en la carpeta de `.endptr/workflows/<NAME_WORKFLOW>.yaml`algo como:
```yaml
name: Test Create Post

steps:
  - http: &create_user
    method: POST
    url: https://api.example.com/users
    body:
      type: json
      data: |-
        {
          "name": "John Doe",
          "email": "john@example.com"
        }
    auth: inherit

  - http: &create_post
    method: POST
    url: https://api.example.com/posts
    body:
      type: json
      data: |-
        {
          "name": "John Doe",
          "text": "This is a test post."
          &create_user.response.body.token
        }
    auth: inherit

  - http: <nombre_collection>.Get 1 post # Este es el nombre de una request guardada

# Aplicar condiciones para poder decir algo como create_post.status_code == 201 para validar que se creó el post correctamente 
# job:
#   script: "echo Hello, Rules!"
#   rules:
#     - if: '$CI_MERGE_REQUEST_TARGET_BRANCH_NAME == "master"'
#       when: always
#     - if: '$VAR =~ /pattern/'
#       when: manual
#     - when: on_success
```
* [ ] Parsear numeritos