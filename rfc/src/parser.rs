use crate::messages::away::*;
use crate::messages::invite::*;
use crate::messages::join::*;
use crate::messages::kick::KickInfo;
use crate::messages::list::*;
use crate::messages::mode::*;
use crate::messages::names::*;
use crate::messages::nick::*;
use crate::messages::oper::*;
use crate::messages::part::*;
use crate::messages::pass::*;
use crate::messages::private_message::*;
use crate::messages::quit::*;
use crate::messages::send::*;
use crate::messages::server_msg::*;
use crate::messages::server_quit::SQuitInfo;
use crate::messages::server_quit_request::SQuitRequestInfo;
use crate::messages::shut::ShutInfo;
use crate::messages::topic::*;
use crate::messages::user::*;
use crate::messages::who::*;
use crate::messages::whois::WhoIsInfo;
use crate::send_file::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Password(PassInfo),
    User(UserInfo),
    Nick(NickInfo),
    Oper(OperInfo),
    Quit(QuitInfo),
    PrivateMessage(PrivateInfo),
    Join(JoinInfo),
    Part(PartInfo),
    Names(NamesInfo),
    Who(WhoInfo),
    Invite(InviteInfo),
    List(ListInfo),
    Kick(KickInfo),
    Away(AwayInfo),
    Connected,
    Topic(TopicInfo),
    Mode(ModeInfo),
    Server(ServerInfo),
    Send(SendInfo),
    ServerQuit(SQuitInfo),
    ServerQuitRequest(SQuitRequestInfo),
    Shut(ShutInfo),
    WhoIs(WhoIsInfo),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageCommand {
    pub prefix: Option<String>,
    pub cmd: Message,
}

/// a partir del prefijo, se setea el indice del comando
fn set_indice(comando_filtrado: &str) -> usize {
    let mut indice_comando: usize = 1;
    if comando_filtrado.as_bytes()[0] != b':' {
        indice_comando = 0;
    }
    indice_comando
}

/// se le eliminan los ultimos dos caracteres de un string, lo usamos para eliminar el \r y \n
fn limpiar_final(cmd: String) -> String {
    let mut nuevo = cmd;
    let len = nuevo.len();
    for _ in 0..2 {
        nuevo.remove(len - 2);
    }
    nuevo
}

/// A partir del indice del comando, se separan los argumentos para pasarle al parser
fn separar_parametros(indice_comando: usize, message_split: &[&str]) -> Vec<String> {
    let mut parametros: Vec<String> = Vec::new();
    for indice in message_split.iter().skip(indice_comando + 1) {
        parametros.push(indice.to_string());
    }
    parametros
}

/// Funcion que parsea a partir de un String, generando un nuevo Message command que vamos a usar a lo largo
/// de la ejecucion de ese comando. A partir de lo que se envien a las funciones de new, se devuelve el error en algunos casos
pub fn parser(cmd: String) -> Result<MessageCommand, String> {
    if is_send(&cmd) {
        let cmd = Message::Send(from_bytes(cmd.as_bytes().to_vec()));
        return Ok(MessageCommand { prefix: None, cmd });
    }

    if cmd.len() == 2 {
        return Err(String::from(""));
    }
    let last_two: Vec<char> = cmd.chars().rev().take(2).collect();
    if !(last_two[0] == '\n' && last_two[1] == '\r') {
        return Err(String::from("error de formato: no hay cr-nl"));
    }
    let nuevo_comando = limpiar_final(cmd);
    let comando_filtrado = nuevo_comando.trim();
    let indice_comando = set_indice(comando_filtrado);

    let split = comando_filtrado.split(' ');
    let message_split = split.collect::<Vec<&str>>();
    let comando = message_split[indice_comando];

    let parametros = separar_parametros(indice_comando, &message_split);

    let mensaje: Message = match comando {
        "PASS" => Message::Password(PassInfo::new(parametros)?),
        "USER" => Message::User(UserInfo::new(parametros)?),
        "NICK" => Message::Nick(NickInfo::new(parametros)?),
        "OPER" => Message::Oper(OperInfo::new(parametros)?),
        "QUIT" => Message::Quit(QuitInfo::new(parametros)),
        "PRIVMSG" => Message::PrivateMessage(PrivateInfo::new(parametros)?),
        "JOIN" => Message::Join(JoinInfo::new(parametros)?),
        "PART" => Message::Part(PartInfo::new(parametros)?),
        "NAMES" => Message::Names(NamesInfo::new(parametros)?),
        "WHO" => Message::Who(WhoInfo::new(parametros)?),
        "INVITE" => Message::Invite(InviteInfo::new(parametros)?),
        "LIST" => Message::List(ListInfo::new(parametros)?),
        "KICK" => Message::Kick(KickInfo::new(parametros)?),
        "AWAY" => Message::Away(AwayInfo::new(parametros)),
        "CONNECTED" => Message::Connected,
        "TOPIC" => Message::Topic(TopicInfo::new(parametros)?),
        "MODE" => Message::Mode(ModeInfo::new(parametros)?),
        "SERVER" => Message::Server(ServerInfo::new(parametros)?),
        "SQUIT" => Message::ServerQuit(SQuitInfo::new(parametros)?),
        "SERVERQ" => Message::ServerQuitRequest(SQuitRequestInfo::new(parametros)),
        "SHUT" => Message::Shut(ShutInfo::new(parametros)),
        "WHOIS" => Message::WhoIs(WhoIsInfo::new(parametros)?),
        // "SENDINFO" => Message::Send(SendInfo::new(parametros)),
        _ => return Err(String::from("No se identifico el Comando")),
    };

    /*init message command*/
    let mut prefix: Option<String> = match indice_comando {
        0 => None,
        1 => {
            message_split[0].to_string().remove(0);
            Some(message_split[0].to_string())
        }
        _ => return Err(String::from("No se identifico el indice del comando")),
    };
    if let Some(slice) = prefix {
        prefix = Some(slice[1..].to_string());
    }
    let message_cmd = MessageCommand {
        prefix,
        cmd: mensaje,
    };

    Ok(message_cmd)
}

fn is_send(cmd: &str) -> bool {
    let split = cmd.split(' ');
    let message_split = split.collect::<Vec<&str>>();
    let comando = message_split[0];
    comando == "SEND"
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_pass() {
        let pass_command = MessageCommand {
            prefix: None,
            cmd: Message::Password(PassInfo {
                pass: "contra".to_string(),
            }),
        };
        let string_to_parse = "PASS contra\r\n".to_string();

        match parser(string_to_parse) {
            Ok(mess_comm) => assert_eq!(pass_command, mess_comm),
            Err(_) => assert_eq!(true, false),
        }
    }
    #[test]
    fn parse_user() {
        let user_cmd = MessageCommand {
            prefix: None,
            cmd: Message::User(UserInfo {
                user: "tomas".to_string(),
                host: "host".to_string(),
                servername: "Default servername".to_string(),
                realname: "Default realname".to_string(),
            }),
        };
        let string_to_parse = "USER tomas host\r\n".to_string();

        match parser(string_to_parse) {
            Ok(mess_comm) => assert_eq!(user_cmd, mess_comm),
            Err(_) => assert_eq!(true, false),
        }
    }

    #[test]
    fn parse_nick() {
        let nick_cmd = MessageCommand {
            prefix: None,
            cmd: Message::Nick(NickInfo {
                nick: "tomas".to_string(),
            }),
        };
        let string_to_parse = "NICK tomas\r\n".to_string();

        match parser(string_to_parse) {
            Ok(mess_comm) => assert_eq!(nick_cmd, mess_comm),
            Err(_) => assert_eq!(true, false),
        }
    }

    #[test]
    fn parse_oper() {
        let oper_cmd = MessageCommand {
            prefix: None,
            cmd: Message::Oper(OperInfo {
                nick: "tomas".to_string(),
                pass: "contra".to_string(),
            }),
        };
        let string_to_parse = "OPER tomas contra\r\n".to_string();

        match parser(string_to_parse) {
            Ok(mess_comm) => assert_eq!(oper_cmd, mess_comm),
            Err(_) => assert_eq!(true, false),
        }
    }
    #[test]
    fn parse_quit() {
        let quit_cmd = MessageCommand {
            prefix: None,
            cmd: Message::Quit(QuitInfo {
                msg: ":me voy".to_string(),
            }),
        };
        let string_to_parse = "QUIT :me voy\r\n".to_string();

        match parser(string_to_parse) {
            Ok(mess_comm) => assert_eq!(quit_cmd, mess_comm),
            Err(_) => assert_eq!(true, false),
        }
    }
    #[test]
    fn parse_priv_msg() {
        let private_message_cmd = MessageCommand {
            prefix: None,
            cmd: Message::PrivateMessage(PrivateInfo {
                receivers: vec!["tomas".to_string()],
                message: "este es el mensaje".to_string(),
            }),
        };
        let string_to_parse = "PRIVMSG tomas :este es el mensaje\r\n".to_string();

        match parser(string_to_parse) {
            Ok(mess_comm) => assert_eq!(private_message_cmd, mess_comm),
            Err(_) => assert_eq!(true, false),
        }
    }
    #[test]
    fn parse_join() {
        let join_cmd = MessageCommand {
            prefix: None,
            cmd: Message::Join(JoinInfo {
                channel_list: vec!["&canal1".to_string()],
                channel_key: None,
            }),
        };
        let string_to_parse = "JOIN &canal1\r\n".to_string();

        match parser(string_to_parse) {
            Ok(mess_comm) => assert_eq!(join_cmd, mess_comm),
            Err(_) => assert_eq!(true, false),
        }
    }
    #[test]
    fn parse_part() {
        let part_cmd = MessageCommand {
            prefix: None,
            cmd: Message::Part(PartInfo {
                channel_list: vec!["&canal1".to_string()],
            }),
        };
        let string_to_parse = "PART &canal1\r\n".to_string();

        match parser(string_to_parse) {
            Ok(mess_comm) => assert_eq!(part_cmd, mess_comm),
            Err(_) => assert_eq!(true, false),
        }
    }
    #[test]
    fn parse_names() {
        let names_cmd = MessageCommand {
            prefix: None,
            cmd: Message::Names(NamesInfo {
                channel_list: vec!["&canal1".to_string()],
            }),
        };
        let string_to_parse = "NAMES &canal1\r\n".to_string();

        match parser(string_to_parse) {
            Ok(mess_comm) => assert_eq!(names_cmd, mess_comm),
            Err(_) => assert_eq!(true, false),
        }
    }

    #[test]
    fn parse_invite() {
        let invite_cmd = MessageCommand {
            prefix: None,
            cmd: Message::Invite(InviteInfo {
                channel: "&canal1".to_string(),
                nick: "tomas".to_string(),
            }),
        };
        let string_to_parse = "INVITE tomas &canal1\r\n".to_string();

        match parser(string_to_parse) {
            Ok(mess_comm) => assert_eq!(invite_cmd, mess_comm),
            Err(_) => assert_eq!(true, false),
        }
    }

    #[test]
    fn parse_list() {
        let list_cmd = MessageCommand {
            prefix: None,
            cmd: Message::List(ListInfo {
                channel_list: Some(vec!["&canal1".to_string()]),
            }),
        };
        let string_to_parse = "LIST &canal1 \r\n".to_string();

        match parser(string_to_parse) {
            Ok(mess_comm) => assert_eq!(list_cmd, mess_comm),
            Err(_) => assert_eq!(true, false),
        }
    }

    #[test]
    fn parse_mode() {
        let mode_cmd = MessageCommand {
            prefix: None,
            cmd: Message::Mode(ModeInfo {
                channel: Some("&canal1".to_string()),
                nick: None,
                flag: "b".to_string(),
                limit: None,
                user: None,
                ban_mask: Some("tomas".to_string()),
                set: true,
            }),
        };
        let string_to_parse = "MODE &canal1 +b tomas \r\n".to_string();

        match parser(string_to_parse) {
            Ok(mess_comm) => assert_eq!(mode_cmd, mess_comm),
            Err(_) => assert_eq!(true, false),
        }
    }
}
