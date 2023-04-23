# 22C2-Grupo-Fiubense

### Multiples servidores

Para probar con multiples servidores, en local, seguir los siguientes pasos:

1. Clonar el repositorio:  [https://github.com/taller-1-fiuba-rust/22C2-Grupo-Fiubense.git](https://github.com/taller-1-fiuba-rust/22C2-Grupo-Fiubense.git)
2. Hacer una copia del directorio.
3. En el archivo config_file de uno de los directorios copiar el siguiente texto:

‘ ‘

SERVER,MAIN

SERVER_NAME,Servidor_principal

MAIN_PORT,127.0.0.1:8095

SECONDARY_PORT,127.0.0.1:8096

DATA_FILE_PATH,./src/data_file

JOINED_CHANNELS_PATH,./src/joined_channels

USERS_CONNECTED,./src/users_connected

DATA_CHANNELS_PATH,./src/data_channels

SERVER_PASSWORD,fiuba

‘ ‘

4. En el archivo config_file del otro  directorio copiar el siguiente texto:

‘ ‘

SERVER,SECONDARY

SERVER_NAME,Servidor_Vecino

MAIN_PORT,127.0.0.1:8095

SECONDARY_PORT,127.0.0.1:8096

DATA_FILE_PATH,./src/data_file

JOINED_CHANNELS_PATH,./src/joined_channels

USERS_CONNECTED,./src/users_connected

DATA_CHANNELS_PATH,./src/data_channels

SERVER_PASSWORD,fiuba

‘ ‘

5. Abrir dos terminales , una en cada directorio, y pararse en ambas sobre el directorio **rfc**.
6. En la terminal correspondiente al servidor principal ejecutar el comando `cargo run  --bin server`  (IMPORTANTE REALIZAR ESTE PASO ANTES DEL SIGUIENTE).
7. En la terminal correspondiente al servidor secundario ejecutar el comando `cargo run  --bin server` .
8. Abriendo otras dos terminales sobre estos mismos directorios ejecutar el comando `cargo run cliente` .
9. Listo. Ya tenemos dos clientes conectados a dos servidores diferentes pertenecientes a la misma red.

### Archivo de configuracion

El archivo de configuracion esta organizado en un formato donde la primer palabra es el nombre de la variable y la segunda es el contenido de la misma.

| Variable | Descripcion |
| --- | --- |
| SERVER | Indica si el servidor a levantar sera secundario o principal. Los valores admitidos son SECONDARY y MAIN |
| SERVER_NAME | Nombre del servidor, puede ser cualquier string. |
| SECONDARY_PORT | En caso de inicializar un servidor secundario aqui se pone la direccion ip junto con el puerto del servidor secundario en formato <ip>:<puerto> |
| DATA_FILE_PATH | Es el path en donde ser guardaran y levantaran las base de datos de usuarios registrados. Ejemplo:  <./src/nombre_archivo>.  |
| JOINED_CHANNELS_PATH | Es el path en donde ser guardaran y levantaran las base de datos de usuarios unidos a canales. Ejemplo:  <./src/joined_channels>.  |
| USERS_CONNECTED | Es el path en donde ser guardaran y levantaran las base de datos de usuarios conectados a la red. Ejemplo:  <./src/users_connected>.  |
| DATA_CHANNELS_PATH | Es el path en donde ser guardaran y levantaran las base de datos de usuarios conectados a la red. Ejemplo:  <./src/data_channels_path>.  |
| SERVER_PASSWORD | Es la contraseña del servidor. Ejemplo: fiuba |

### Ejecucion local

1. Clonar el repositorio:  [https://github.com/taller-1-fiuba-rust/22C2-Grupo-Fiubense.git](https://github.com/taller-1-fiuba-rust/22C2-Grupo-Fiubense.git)
2. Abrir dos terminales y pararse, en ambas, sobre el directorio **rfc**.
3. En una de las dos terminales ejecutar el comando: `cargo run  --bin server` 
4. En la otra terminal ejecutar el comando `cargo run cliente` 

### Login

Es posible registrarse como un usuario nuevo o tambien logearse con una cuenta ya registrada.

Cuentas ya registradas  disponibles para logearse:

| Nick | Password |
| --- | --- |
| franco | contra |
| tomas | contra |
| fiuba | contra |

Si se desea ingresar con un **usuario nuevo** revisar que el nick no se encuentre en la tabla de arriba.

### Probar con multiples usuarios

Para probar la aplicacion con multiples clientes repetir el paso 4 indicado en el instructivo de  ejecucion local.
