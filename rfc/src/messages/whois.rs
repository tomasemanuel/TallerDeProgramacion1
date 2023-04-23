use std::{collections::HashMap, sync::RwLockReadGuard};

use crate::{datauser::DataUserFile, server::change_vec_to_string};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhoIsInfo {
    pub name: String,
}

impl WhoIsInfo {
    /// crea un nuevo WhoIsInfo para su uso en parser.rs
    pub fn new(parametros: Vec<String>) -> Result<WhoIsInfo, String> {
        if parametros.is_empty() {
            return Err(String::from("ERR_NONICKNAMEGIVEN"));
        }
        Ok(WhoIsInfo {
            name: parametros[0].clone(),
        })
    }
}
/// Devuelve un answer a partir de la informacion propuesta por who is. Devolviendo si es un operador, a los canales que se unio, y si esta away.
pub fn whois_function(
    operators: RwLockReadGuard<Vec<String>>,
    data_user: &DataUserFile,
    joined_channels: &RwLockReadGuard<HashMap<String, Vec<String>>>,
) -> Result<String, String> {
    let mut answer = String::from("WHOIS ");
    answer.push_str("OP:");
    answer.push_str(operators.contains(&data_user.nickname).to_string().as_str());
    answer.push_str(" JC:");
    if let Some(vec_joined) = joined_channels.get(&data_user.nickname) {
        let string_channels = change_vec_to_string(vec_joined.clone());
        answer.push_str(&string_channels);
    }
    answer.push_str(" AW:");
    match data_user.away {
        Some(_) => answer.push_str("true"),
        None => answer.push_str("false"),
    }
    Ok(answer)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn who_is_with_less_parameters() {
        let parametros = vec![];
        if let Err(error_msg) = WhoIsInfo::new(parametros) {
            assert_eq!(error_msg, "ERR_NONICKNAMEGIVEN".to_string());
        }
    }

    #[test]
    fn who_is_with_correct_parameters() {
        let parametros = vec!["nickname".to_string()];
        if let Ok(whois_info) = WhoIsInfo::new(parametros) {
            assert_eq!(whois_info.name, "nickname".to_string());
        }
    }
}
