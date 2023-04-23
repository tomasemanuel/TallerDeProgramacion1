use super::{custom_chat_button::CustomButton, main_window::add_new_channel, object_ext::*};
use crate::answers::privmsg_answer::PrivmsgAnswer;
use glib::Object;
use gtk::{
    traits::{ContainerExt, LabelExt, StyleContextExt, TextBufferExt, TextViewExt, WidgetExt},
    TextBuffer,
};
use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, RwLock},
};

pub fn change_channels(
    old_name: String,
    new_name: String,
    objects: &Vec<Object>,
    textbuffers: Arc<RwLock<HashMap<String, TextBuffer>>>,
    tx: &Sender<String>,
) {
    let mut hash = textbuffers.write().unwrap();
    let buffer = hash.remove(&old_name);
    drop(hash);
    let listbox = objects.search_listbox_named("channel_listbox");
    if let Some(textbuffer) = buffer {
        add_new_channel(
            &new_name,
            textbuffers.clone(),
            &listbox,
            "From channel: ",
            objects,
            tx,
            Some(textbuffer),
        );
    }
    let channel_listbox = objects.search_listbox_named("channel_listbox");
    if let Some(parent) = channel_listbox.search_listboxrow_named(&old_name) {
        channel_listbox.remove(&parent);
        objects
            .search_textview_named("names_channel_textview")
            .buffer()
            .unwrap()
            .set_text("");
        objects
            .search_textview_named("main_textview")
            .buffer()
            .unwrap()
            .set_text("");
        objects
            .search_label_named("view_marker")
            .set_text("From channel:");
        objects.search_label_named("actual_name").set_text("");
        objects.search_label_named("name_request").set_text("");
        objects.search_event_named("topic").set_visible(false);
    }
}

fn write_on_buffer(privmsg: PrivmsgAnswer, textbuffer: Option<&TextBuffer>) {
    if let Some(textbuff) = textbuffer {
        let mut iter = textbuff.end_iter();
        if textbuff.start_iter() != iter {
            textbuff.insert(&mut iter, "\n");
        }
        textbuff.insert(
            &mut iter,
            &format!("{}{}{}", privmsg.from_user, " : ", privmsg.message),
        );
    }
}

fn inviting_handling(privmsg: PrivmsgAnswer, objects: &Vec<Object>) {
    let message_slice = privmsg.message.split(' ').collect::<Vec<&str>>();
    let channel_name = message_slice[1].replace('#', "");
    objects
        .search_label_named("invite_label")
        .set_text(&format!(
            "{}{}{}",
            "You were invited to channel: ", channel_name, " do you accept?"
        ));
    objects
        .search_label_named("actual_name_invite")
        .set_text(message_slice[1]);
    objects
        .search_menubutton_named("invite_notif_button")
        .set_visible(true);
}

fn mode_handling(
    privmsg: PrivmsgAnswer,
    objects: &Vec<Object>,
    textbuffers: Arc<RwLock<HashMap<String, TextBuffer>>>,
    tx_back: &Sender<String>,
) {
    let message = privmsg.message; //:MODE +i channel nick
    let message_split = message.split(' ');
    let vec: Vec<String> = message_split.into_iter().map(|a| a.to_string()).collect();
    match vec[1].as_str() {
        "+i" => {
            let new_name = vec[2].clone().replace('&', "#");
            change_channels(vec[2].clone(), new_name, objects, textbuffers, tx_back)
        }
        "-i" => {
            let new_name = vec[2].clone().replace('#', "&");
            change_channels(vec[2].clone(), new_name, objects, textbuffers, tx_back)
        }
        _ => println!(),
    }
}

pub fn private_handler(
    objects: &Vec<Object>,
    privmsg: PrivmsgAnswer,
    textbuffers: Arc<RwLock<HashMap<String, TextBuffer>>>,
    tx_back: &Sender<String>,
) {
    let is_channel = privmsg.from_channel.is_some();
    if privmsg.message.starts_with("DCC") {
        return;
    };
    if privmsg.message.starts_with("RPL_INVITING") {
        inviting_handling(privmsg, objects);
    } else if privmsg.message.starts_with("MODE") {
        mode_handling(privmsg, objects, textbuffers, tx_back);
    } else {
        let hash = textbuffers.read().unwrap();
        let textbuffer = hash.get(&privmsg.from_user);
        if let Some(channel_name) = privmsg.clone().from_channel {
            let textbuffer2 = hash.get(&channel_name);
            write_on_buffer(privmsg.clone(), textbuffer2);
        } else if textbuffer.is_none() {
            //no esta en el hash
            drop(hash);
            let user_listbox = objects.search_listbox_named("user_listbox");
            add_new_channel(
                &privmsg.from_user,
                Arc::clone(&textbuffers),
                &user_listbox,
                &String::from("From user: "),
                objects,
                tx_back,
                None,
            );
            let hash = textbuffers.read().unwrap();
            let nuevo = hash.get(&privmsg.from_user);
            write_on_buffer(privmsg.clone(), nuevo);
        } else {
            write_on_buffer(privmsg.clone(), textbuffer);
        }
        let (notif_label_name, notif_image_name, align, title_label_name) = match is_channel {
            true => (
                "channels_unread_msg",
                "channels_unread_img",
                0.0,
                "channels_chat_label",
            ),
            false => (
                "users_unread_msg",
                "users_unread_img",
                0.25,
                "users_chats_label",
            ),
        };
        let row = if is_channel {
            objects
                .search_listbox_named("channel_listbox")
                .search_listboxrow_named(&privmsg.from_channel.unwrap())
        } else {
            objects
                .search_listbox_named("user_listbox")
                .search_listboxrow_named(&privmsg.from_user)
        };
        if let Some(row) = row {
            if !row.style_context().has_class("selected") {
                let custom_button = CustomButton::from_row(row);
                custom_button.add_notif();
                let label = objects.search_label_named(notif_label_name);
                let new_number = label.text().as_str().parse::<u64>().unwrap() + 1;
                label.set_text(&new_number.to_string());
                label.set_visible(true);
                objects
                    .search_image_named(notif_image_name)
                    .set_visible(true);
                objects
                    .search_label_named(title_label_name)
                    .set_xalign(align);
            }
        }
    }
}
