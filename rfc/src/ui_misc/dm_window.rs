use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, RwLock},
};

use crate::ui_misc::main_window::{add_new_channel, change_stack};
use glib::Object;
use gtk::{gdk::Event, prelude::*, Button, TextBuffer};

use super::object_ext::VecOwnExt;

pub fn new_dm_window_init(
    objects: &Vec<Object>,
    mut nicklist: Vec<String>,
    chats: Arc<RwLock<HashMap<String, TextBuffer>>>,
    tx_back: &Sender<String>,
) {
    let listbox = objects.search_listbox_named("direct_msg_listbox");
    let mut nicks_added: Vec<String> = vec![];
    for widget in listbox.children() {
        nicks_added.push(widget.widget_name().to_string());
    }
    nicklist.retain(|nick| !nicks_added.contains(nick));
    for nick in nicklist {
        let button = Button::builder()
            .name(nick.clone())
            .label(nick.clone())
            .height_request(30)
            .width_request(196)
            .visible(true)
            .build();
        listbox.insert(&button, -1);
        let last_row = &listbox.children()[listbox.children().len() - 1];
        last_row.set_widget_name(&nick);
        let objs = objects.clone();
        let tx = tx_back.clone();
        let hash = chats.clone();
        button.connect_clicked(move |_| {
            let nick_listbox = objs.search_listbox_named("user_listbox");
            let label = add_new_channel(
                &nick,
                hash.clone(),
                &nick_listbox,
                &String::from("From user :"),
                &objs,
                &tx,
                None,
            );
            if let Some(cstm_button) = label {
                cstm_button.label_event.emit_by_name_with_values(
                    "button-press-event",
                    &[Event::new(gtk::gdk::EventType::ButtonPress).to_value()],
                );
                objs.search_label_named("name_request").set_text(&nick);
            };
            objs.search_popover_named("new_dm_popover").popdown();
            objs.search_popover_named("add_chats_popover").popdown();
            let user_button = objs.search_toggle_named("users_button");
            let channels_button = objs.search_toggle_named("channels_button");
            let stack = objs.search_stack_named("textview_stack");
            change_stack(&user_button, &channels_button, stack, "Users");
            channels_button.set_active(false);
        });
    }
}
