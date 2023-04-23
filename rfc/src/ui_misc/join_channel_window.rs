use glib::Object;
use gtk::{prelude::*, Entry, TextBuffer};
use std::sync::mpsc::Sender;

use super::object_ext::VecOwnExt;

fn send_message(entry: &Entry, tx: &Sender<String>, textbuffer: &TextBuffer) {
    let text = entry.buffer().text();
    let mut iter = textbuffer.end_iter();
    match tx.send(text) {
        Ok(_) => println!(),
        Err(_) => {
            textbuffer.insert(&mut iter, "Could not send the message");
            textbuffer.insert(&mut iter, "\n");
        }
    }
    entry.buffer().delete_text(0, Some(520));
}

pub fn join_channel_window_init(objects: &Vec<Object>, tx_back: &Sender<String>) {
    let join_channel_window = objects.search_popover_named("join_channel");
    let public_button = objects.search_radio_button_named("public_button");
    let invite_only_button = objects.search_radio_button_named("invite_only_button");

    let entry = objects.search_entry_named("join_channel_entry");
    let textview = objects.search_textview_named("main_textview");
    public_button.join_group(Some(&invite_only_button));

    let inv = invite_only_button.clone();
    public_button.connect_toggled(move |_| {
        inv.set_active(false);
    });

    let pub1 = public_button.clone();
    invite_only_button.connect_toggled(move |_| {
        pub1.set_active(false);
    });
    let tx = tx_back.clone();
    let objs = objects.clone();
    entry.connect_activate(move |entry_c| {
        if entry_c.buffer().length() != 0 {
            if public_button.is_active() {
                entry_c.buffer().insert_text(0, "JOIN &");
            } else {
                entry_c.buffer().insert_text(0, "JOIN #");
            }
            send_message(entry_c, &tx, &textview.buffer().unwrap());
            join_channel_window.set_visible(false);
        }
        objs.search_popover_named("add_chats_popover").popdown();
    });
}
