#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelList {
    pub invited_list: Option<Vec<String>>, // si es none el channel es publico
    pub joined_list: Vec<String>,
    pub operators: Vec<String>,
    pub topic: Option<String>,
    pub ban_mask: Option<Vec<String>>,
    pub secret: bool,
    pub private: bool,
}

/// Esta estructura guarda una lista de Usuarios Invitados y de Usuarios ya Registrados en el canal
impl ChannelList {
    /// A partir de un string y una invitacion, se crea un channel list para su uso en join y en invite
    pub fn new(nickname: String, invitacion: bool) -> ChannelList {
        let joined = vec![nickname.clone()];
        let operators = vec![nickname];
        if invitacion {
            let invited = Vec::new();
            return ChannelList {
                invited_list: Some(invited),
                joined_list: joined,
                operators,
                topic: None,
                ban_mask: None,
                secret: false,
                private: false,
            };
        }
        ChannelList {
            invited_list: None,
            joined_list: joined,
            operators,
            topic: None,
            ban_mask: None,
            secret: false,
            private: false,
        }
    }
    pub fn add_nickname(&mut self, nickname: String) -> &mut ChannelList {
        self.joined_list.push(nickname);
        self
    }
    /// Crea una instancia de Channel list a partir de un vector de str. Se usa para updatear la base de datos
    pub fn new_with(vec_str_with_data: Vec<&str>) -> ChannelList {
        let mut vector_data: Vec<String> = vec_str_with_data
            .into_iter()
            .map(|channel| channel.to_string())
            .collect(); //None,fiuba;tomas;,fiuba;,None,None,false,false,
        vector_data.remove(0);
        let invited_list: Option<Vec<String>> = match vector_data[0].as_str() {
            // Invited info{
            "None" => None,
            _ => Some(
                vector_data[0]
                    .split(';')
                    .map(|channel| channel.to_string())
                    .collect(),
            ),
        };
        let joined_list: Vec<String> = vector_data[1]
            .split(';')
            .map(|channel| channel.to_string())
            .collect();
        let operators: Vec<String> = vector_data[2]
            .split(';')
            .map(|channel| channel.to_string())
            .collect();
        let topic: Option<String> = match vector_data[3].as_str() {
            "None" => None,
            _ => Some(vector_data[3].clone()),
        };
        let ban_mask: Option<Vec<String>> = match vector_data[4].as_str() {
            // Invited info{
            "None" => None,
            _ => Some(
                vector_data[4]
                    .split(';')
                    .map(|channel| channel.to_string())
                    .collect(),
            ),
        };
        let secret: bool = matches!(vector_data[5].as_str(), "true");
        let private: bool = matches!(vector_data[6].as_str(), "true");
        ChannelList {
            invited_list,
            joined_list,
            operators,
            topic,
            ban_mask,
            secret,
            private,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_channel_list_with_nick_and_invitation() {
        let invited = Vec::new();
        let joined = vec!["usuario_1".to_string()];
        let channel_list_ok = ChannelList {
            invited_list: Some(invited),
            joined_list: joined.clone(),
            operators: joined,
            topic: None,
            ban_mask: None,
            secret: false,
            private: false,
        };
        let channel_assert = ChannelList::new("usuario_1".to_string(), true);
        assert_eq!(channel_list_ok, channel_assert);
    }
    #[test]
    fn create_channel_list_with_nick() {
        let joined = vec!["usuario_1".to_string()];
        let channel_list_ok = ChannelList {
            invited_list: None,
            joined_list: joined.clone(),
            operators: joined,
            topic: None,
            ban_mask: None,
            secret: false,
            private: false,
        };
        let channel_assert = ChannelList::new("usuario_1".to_string(), false);
        assert_eq!(channel_list_ok, channel_assert);
    }
    #[test]
    fn adds_a_nickname_to_list() {
        let joined = vec!["usuario_1".to_string(), "usuario_2".to_string()];
        let operators = vec!["usuario_1".to_string()];
        let channel_list_ok = ChannelList {
            invited_list: None,
            joined_list: joined.clone(),
            operators,
            topic: None,
            ban_mask: None,
            secret: false,
            private: false,
        };
        let mut channel_assert = ChannelList::new("usuario_1".to_string(), false);
        channel_assert.add_nickname("usuario_2".to_string());
        assert_eq!(channel_list_ok, channel_assert);
    }

    #[test]
    fn create_with_vector() {
        let vector = vec![
            "canal_prueba",
            "None",
            "juan;tomasito",
            "juan",
            "canal de prueba",
            "None",
            "false",
            "false",
        ];

        let channel_assert = ChannelList {
            invited_list: None,
            joined_list: vec!["juan".to_string(), "tomasito".to_string()],
            operators: vec!["juan".to_string()],
            topic: Some("canal de prueba".to_string()),
            ban_mask: None,
            secret: false,
            private: false,
        };
        let channel = ChannelList::new_with(vector);
        assert_eq!(channel, channel_assert);
    }
}
