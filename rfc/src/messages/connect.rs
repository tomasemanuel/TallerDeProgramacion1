use std::{collections::HashMap, sync::RwLockReadGuard};

/// Funcion que se usa en channels.rs para devolver los canales a los que esta conectado un usuario a traves de un string
pub fn return_connected_channels(
    joined_channels: &RwLockReadGuard<HashMap<String, Vec<String>>>,
    nickname: &String,
) -> String {
    let mut respuesta = String::from("CHANNELLIST ");
    if let Some(connected_channels) = joined_channels.get(nickname) {
        for channel_name in connected_channels {
            respuesta.push_str(channel_name);
            //respuesta.push_str(",");
            respuesta.push(',');
        }
    }
    respuesta
}
