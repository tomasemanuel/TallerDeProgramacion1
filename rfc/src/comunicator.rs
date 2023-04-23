use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
};

#[derive(Debug)]
pub struct Comunicator {
    pub transmisor_list: HashMap<String, Sender<String>>,
}

impl Default for Comunicator {
    fn default() -> Self {
        Self::new()
    }
}

impl Comunicator {
    pub fn new() -> Comunicator {
        Comunicator {
            transmisor_list: HashMap::new(),
        }
    }

    /// Agrega al hash el transmisor y devuelve el extremo receptor para que sea enviado al nuevo hilo
    pub fn new_comunication(&mut self, name: String) -> Receiver<String> {
        let (tx, rx) = mpsc::channel();
        self.transmisor_list.insert(name, tx);
        rx
    }

    /// Busca el nombre dado en el hash de transmisores y si lo encuentra envia el mensaje pasado por parametro
    /// si no devuelve un error.
    pub fn pass_message_to(&mut self, name: String, message: String) -> Result<(), String> {
        let option_tx = self.transmisor_list.get(&name);

        if let Some(tx) = option_tx {
            if tx.send(message).is_err() {
                return Err("FAILED TO SEND".to_string());
            }
            return Ok(());
        }

        Err("NOT A P2P CONVERSATION".to_string())
    }
}
