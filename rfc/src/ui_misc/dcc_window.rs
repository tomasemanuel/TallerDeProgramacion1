use std::sync::mpsc::Sender;

use crate::answers::dcc_close_answer::DCCCloseAnswer;
use glib::Object;
use gtk::{
    traits::{
        BuilderExt, ButtonExt, EntryExt, GtkApplicationExt, GtkWindowExt, LabelExt, TextBufferExt,
        TextViewExt, WidgetExt,
    },
    Builder, TextView,
};

use super::object_ext::{ApplicationOwnExt, VecOwnExt};

fn send_message(textview: TextView, message: String) {
    let textbuffer = textview.buffer().unwrap();
    let mut iter = textbuffer.end_iter();
    let me = if textbuffer.start_iter() != iter {
        "\nme: "
    } else {
        "me: "
    };
    textbuffer.insert(&mut iter, me);
    textbuffer.insert(&mut iter, &message);
}

pub fn new_dcc_window(nick: String, tx: &Sender<String>, app: &gtk::Application) {
    let objects: Vec<Object> = Builder::from_string(include_str!("dcc.glade")).objects();
    let window = objects.search_window_named("dcc_window");
    window.set_widget_name(nick.as_str());
    app.add_window(&window);
    objects
        .search_label_named("dcc_label")
        .set_text(nick.as_str());

    let mut tx_back = tx.clone();
    let mut nick_copy = nick.clone();
    let mut objs = objects.clone();
    objects
        .search_entry_named("dcc_entry")
        .connect_activate(move |entry| {
            let message = entry.text().to_string();
            tx_back
                .send(format!("P2PCHAT {nick_copy} :{message}"))
                .unwrap();
            send_message(objs.search_textview_named("dcc_textview"), message);
            entry.set_text("");
        });
    objs = objects.clone();
    tx_back = tx.clone();
    nick_copy = nick.clone();
    objects
        .search_button_named("dcc_send")
        .connect_clicked(move |_| {
            let entry = objs.search_entry_named("dcc_entry");
            let message = entry.text().to_string();
            tx_back
                .send(format!("P2PCHAT {nick_copy} :{message}"))
                .unwrap();
            send_message(objs.search_textview_named("dcc_textview"), message);
            entry.set_text("");
        });
    tx_back = tx.clone();
    window.connect_destroy(move |_| {
        tx_back.send(format!("PRIVMSG {nick} :DCC CLOSE")).unwrap();
    });
    window.show_all();
}

pub fn dcc_close_window(application: &gtk::Application, close_answer: DCCCloseAnswer) {
    let window = application.get_window_by_name(&close_answer.name);
    window.close();
}
