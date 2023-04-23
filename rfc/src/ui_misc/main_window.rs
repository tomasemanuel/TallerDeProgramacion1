use super::dcc_window::new_dcc_window;
use super::object_ext::VecOwnExt;
use crate::answers::connected_answer::ChannelListAnswer;
use crate::config_file::ConfigFile;
use crate::ui_misc::custom_chat_button::CustomButton;
use crate::ui_misc::join_channel_window::join_channel_window_init;
use crate::ui_misc::object_ext::WidgetOwnExt;
use glib::Object;
use gtk::{
    prelude::*, Inhibit, ListBox, ListBoxRow, Stack, StateFlags, TextBuffer, TextTagTable,
    ToggleButton,
};
use std::collections::HashMap;
use std::fs;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

pub fn remove_chat(listbox: ListBox, row: ListBoxRow, objects: &Vec<Object>) {
    listbox.remove(&row);
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
    objects.search_label_named("view_marker").set_text("");
    objects.search_label_named("actual_name").set_text("");
    objects.search_label_named("name_request").set_text("");
    objects.search_event_named("topic").set_visible(false);
}
pub fn add_new_channel(
    channel_name: &String,
    chats: Arc<RwLock<HashMap<String, TextBuffer>>>,
    listbox: &ListBox,
    initial_string: &str,
    objects: &Vec<Object>,
    tx: &Sender<String>,
    textbuffer_op: Option<TextBuffer>,
) -> Option<CustomButton> {
    let is_channel = channel_name.contains('&') || channel_name.contains('#');
    if chats.read().unwrap().get(channel_name).is_some() {
        return None;
    }
    let name_copy = channel_name.clone().as_str().replace(['&', '#'], "");

    let textbuffer = match textbuffer_op {
        Some(textbuff) => textbuff,
        None => {
            let table: TextTagTable = TextTagTable::new();
            TextBuffer::new(Some(&table))
        }
    };
    chats
        .write()
        .unwrap()
        .insert(channel_name.to_string(), textbuffer);
    let cstm_button = CustomButton::new(name_copy.clone(), channel_name);
    listbox.insert(&cstm_button.container, -1);
    let box_children = listbox.children();
    let last_row = box_children[listbox.children().len() - 1].clone();

    last_row.set_widget_name(channel_name);
    let mut cstm_button_copy = cstm_button.clone();
    last_row.connect_state_flags_changed(move |row, flags| {
        if row.state_flags().intersects(StateFlags::PRELIGHT) {
            cstm_button_copy.leave_event.set_visible(true);
            cstm_button_copy
                .background
                .style_context()
                .add_class("highlight");
            cstm_button_copy
                .chat_label
                .style_context()
                .add_class("bolded");
        } else if flags.intersects(StateFlags::PRELIGHT)
            && !row.state_flags().intersects(StateFlags::PRELIGHT)
        {
            cstm_button_copy.leave_event.set_visible(false);
            cstm_button_copy
                .background
                .style_context()
                .remove_class("highlight");
            cstm_button_copy
                .chat_label
                .style_context()
                .remove_class("bolded");
        }
        cstm_button_copy.refresh_label_event();
        row.style_context().remove_class("activatable");
    });
    let textview = objects.search_textview_named("main_textview");
    let init_string = initial_string.to_owned();
    let mut objs = objects.clone();
    let mut tx_back = tx.clone();
    let mut hash = Arc::clone(&chats);
    let channel_name_copy = channel_name.clone();
    cstm_button_copy = cstm_button.clone();
    let last_row_copy = last_row.clone();
    cstm_button
        .label_event
        .connect_button_press_event(move |_, _| {
            let lock = hash.read().unwrap();
            let buff = lock.get(&channel_name_copy).unwrap();
            let label_textview = objs.search_label_named("view_marker");
            objs.search_label_named("name_request")
                .set_text(&channel_name_copy);
            objs.search_label_named("actual_name")
                .set_text(&channel_name_copy);
            if channel_name_copy.starts_with('&') || channel_name_copy.starts_with('#') {
                tx_back.send(format!("NAMES {channel_name_copy}")).unwrap();
                tx_back.send(format!("TOPIC {channel_name_copy}")).unwrap();
                objs.search_image_named("sleeping").set_visible(false);
                objs.search_event_named("topic").set_visible(true);
            } else {
                tx_back.send(format!("WHOIS {channel_name_copy}")).unwrap();
                objs.search_textview_named("names_channel_textview")
                    .buffer()
                    .unwrap()
                    .set_text("");
                objs.search_event_named("topic").set_visible(false);
            }
            label_textview.set_text(&format!("{}{}", init_string, name_copy.as_str()));
            textview.set_buffer(Some(buff));
            for child in objs.search_listbox_named("user_listbox").children() {
                if child.style_context().has_class("selected") {
                    child.style_context().remove_class("selected");
                }
            }
            for child in objs.search_listbox_named("channel_listbox").children() {
                if child.style_context().has_class("selected") {
                    child.style_context().remove_class("selected");
                }
            }
            let dcc_button = objs.search_button_named("dcc_new_chat");
            if is_channel {
                dcc_button.set_visible(false)
            } else {
                dcc_button.set_visible(true)
            };
            let (notif_name, image_name, chats_name) = if is_channel {
                (
                    "channels_unread_msg",
                    "channels_unread_img",
                    "channels_chat_label",
                )
            } else {
                ("users_unread_msg", "users_unread_img", "users_chats_label")
            };
            let notif_label = objs.search_label_named(notif_name);
            let notif = cstm_button_copy.remove_notif();
            notif_label.set_text(
                &(notif_label.text().as_str().parse::<u64>().unwrap() - notif).to_string(),
            );
            if notif_label.text().as_str() == "0" {
                notif_label.set_visible(false);
                objs.search_image_named(image_name).set_visible(false);
                objs.search_label_named(chats_name).set_xalign(0.5);
            };
            last_row_copy.style_context().add_class("selected");
            Inhibit(true)
        });
    let channel_name_copy = channel_name.clone();
    tx_back = tx.clone();
    let listbox_copy = listbox.clone();
    objs = objects.clone();
    hash = Arc::clone(&chats);
    cstm_button
        .leave_event
        .connect_button_press_event(move |_, _| {
            match is_channel {
                true => tx_back.send(format!("PART {channel_name_copy}")).unwrap(),
                false => {
                    hash.write().unwrap().remove(&channel_name_copy);
                    remove_chat(listbox_copy.clone(), last_row.to_listrow(), &objs)
                }
            }
            Inhibit(true)
        });
    drop(chats);
    Some(cstm_button)
}

pub fn change_stack(own: &ToggleButton, other: &ToggleButton, stack: Stack, listbox_name: &str) {
    if !other.is_active() {
        other.set_active(false);
    };
    own.set_active(true);
    stack.set_visible_child_name(listbox_name);
}

pub fn show_main_window(objects: &Vec<Object>, tx: &Sender<String>, nick: String) {
    let main_window = objects.search_window_named("Main_window");
    let window = objects.search_window_named("login_window");
    window.set_visible(false);
    let nick_label = objects.search_label_named("nick_label");
    nick_label.set_label(nick.as_str());
    main_window.set_visible(true);
    match tx.send(String::from("CONNECTED")) {
        Ok(_) => println!("Conexión exitosa"),
        Err(_) => println!("Desastre"),
    };
}

pub fn add_channels(
    objects: &Vec<Object>,
    channel_list: ChannelListAnswer,
    chats: Arc<RwLock<HashMap<String, TextBuffer>>>,
    tx: &Sender<String>,
) {
    for channel_name in channel_list.channel_list {
        let channel_listbox = objects.search_listbox_named("channel_listbox");
        if !channel_name.is_empty() {
            add_new_channel(
                &channel_name,
                Arc::clone(&chats),
                &channel_listbox,
                &String::from("From channel: "),
                objects,
                tx,
                None,
            );
        }
    }
}

fn send_message(objects: &Vec<Object>, tx: &Sender<String>) {
    let entry = objects.search_entry_named("entry_bar");
    let textbuffer = objects
        .search_textview_named("main_textview")
        .buffer()
        .unwrap();
    let mut iter = textbuffer.end_iter();
    let name = objects.search_label_named("name_request").text();
    let final_text = format!("PRIVMSG {} :{}", name.as_str(), entry.buffer().text());
    match tx.send(final_text) {
        Ok(_) => {
            let me = if textbuffer.start_iter() != iter {
                "\nme: "
            } else {
                "me: "
            };
            textbuffer.insert(&mut iter, me);
            textbuffer.insert(&mut iter, &entry.buffer().text());
            entry.buffer().delete_text(0, Some(520));
        }
        Err(_) => {
            textbuffer.insert(&mut iter, "Could not send the message");
            textbuffer.insert(&mut iter, "\n");
        }
    }
}

fn send_broadcast(tx: &Sender<String>, message: &gtk::Entry, receivers: &gtk::Entry) {
    let send_to = receivers.buffer().text();
    let text = message.buffer().text();

    let final_text = format!("{}{}{}{}", "PRIVMSG ", send_to, " :", text);
    tx.send(final_text).unwrap();
}

pub fn main_window(objects: Vec<Object>, tx_back: &Sender<String>, application: &gtk::Application) {
    let main_window = objects.search_window_named("Main_window");
    if let Ok(configf) = ConfigFile::new("./src/config_file".to_string()) {
        let address: String = if configf.server_type.as_str() == "MAIN" {
            configf.main_port
        } else {
            configf.secondary_port
        };
        objects.search_label_named("server_name").set_text(&address);
    }
    let entry = objects.search_entry_named("entry_bar");
    let send_button = objects.search_event_named("send_button");
    let join_button = objects.search_menubutton_named("join_channel_button");

    let tx = tx_back.clone();
    let objs = objects.clone();
    join_channel_window_init(&objs, &tx);
    send_button.connect_button_press_event(move |_, _| {
        send_message(&objs, &tx);
        Inhibit(true)
    });

    let tx = tx_back.clone();
    let objs = objects.clone();
    entry.connect_activate(move |_| send_message(&objs, &tx));

    let tx = tx_back.clone();
    join_button.connect_clicked(move |_| tx.send("LIST".to_owned()).unwrap());

    let tx = tx_back.clone();
    let objs = objects.clone();
    objects
        .search_menubutton_named("new_dm_button")
        .connect_clicked(move |_| {
            objs.search_label_named("name_request").set_text("");
            tx.send("NAMES".to_owned()).unwrap();
        });

    let tx = tx_back.clone();
    let objs = objects.clone();
    objects
        .search_event_named("names_request_event")
        .connect_button_press_event(move |_, _| {
            let channel_name = objs.search_label_named("actual_name").text();
            if channel_name.starts_with('&') || channel_name.starts_with('#') {
                tx.send(format!("{}{}", "NAMES ".to_owned(), channel_name))
                    .unwrap();
                tx.send(format!("{}{}", "TOPIC ".to_owned(), channel_name))
                    .unwrap();
            } else {
                tx.send(format!("{}{}", "WHOIS ", channel_name)).unwrap();
                objs.search_textview_named("names_channel_textview")
                    .buffer()
                    .unwrap()
                    .set_text("");
                objs.search_event_named("topic").set_visible(false);
            }
            Inhibit(false)
        });

    let tx = tx_back.clone();
    let objs = objects.clone();
    objects
        .search_entry_named("invite_entry")
        .connect_activate(move |entry| {
            objs.search_popover_named("invite_nick").set_visible(false);
            let nick = entry.buffer().text();
            tx.send(format!(
                "{}{}{}{}",
                "INVITE ",
                nick,
                " ",
                objs.search_label_named("actual_name").text().as_str()
            ))
            .unwrap();
            entry.buffer().set_text("");
        });

    let tx = tx_back.clone();
    let objs = objects.clone();
    objects
        .search_entry_named("kick_entry")
        .connect_activate(move |entry| {
            objs.search_popover_named("kick_nick").set_visible(false);
            let nick = entry.buffer().text();
            tx.send(format!(
                "{}{}{}{}",
                "KICK ",
                objs.search_label_named("actual_name").text().as_str(),
                " ",
                nick
            ))
            .unwrap();
            entry.buffer().set_text("");
        });

    let tx = tx_back.clone();
    let objs = objects.clone();
    objects
        .search_entry_named("topic_entry")
        .connect_activate(move |entry| {
            objs.search_popover_named("topic_popover")
                .set_visible(false);
            let new_topic = entry.buffer().text();
            tx.send(format!(
                "{}{}{}{}",
                "TOPIC ",
                objs.search_label_named("actual_name").text().as_str(),
                " ",
                new_topic
            ))
            .unwrap();
            entry.buffer().set_text("");
        });

    let tx = tx_back.clone();
    let objs = objects.clone();
    objects
        .search_button_named("yes_invite")
        .connect_clicked(move |_| {
            let text = objs.search_label_named("actual_name_invite").text();
            tx.send(format!("{}{}", "JOIN ", text.as_str())).unwrap();
            objs.search_popover_named("invite_popover")
                .set_visible(false);
            objs.search_menubutton_named("invite_notif_button")
                .set_visible(false);
        });

    let objs = objects.clone();
    objects
        .search_button_named("no_invite")
        .connect_clicked(move |_| {
            objs.search_popover_named("invite_popover")
                .set_visible(false);
            objs.search_menubutton_named("invite_notif_button")
                .set_visible(false);
        });

    let objs = objects.clone();
    let tx = tx_back.clone();
    objects
        .search_entry_named("servername_entry")
        .connect_activate(move |entry| {
            let comment_entry = objs.search_entry_named("comment_squit_entry");
            if tx
                .send(format!(
                    "{}{}{}{}",
                    "SQUIT ",
                    entry.buffer().text(),
                    " ",
                    comment_entry.buffer().text()
                ))
                .is_err()
            {
                println!("Error Send Squit");
            }
            entry.buffer().set_text("");
            comment_entry.buffer().set_text("");
            objs.search_popover_named("squit_popover")
                .set_visible(false);
        });
    let objs = objects.clone();
    let tx = tx_back.clone();
    objects
        .search_entry_named("comment_squit_entry")
        .connect_activate(move |entry| {
            let servername_entry = objs.search_entry_named("servername_entry");
            if tx
                .send(format!(
                    "{}{}{}{}",
                    "SQUIT ",
                    servername_entry.buffer().text(),
                    " ",
                    entry.buffer().text()
                ))
                .is_err()
            {
                println!("Error Send Quit");
            }
            entry.buffer().set_text("");
            servername_entry.buffer().set_text("");
            objs.search_popover_named("squit_popover")
                .set_visible(false);
        });
    let objs = objects.clone();
    let tx = tx_back.clone();
    objects
        .search_entry_named("nickname_oper_entry")
        .connect_activate(move |entry| {
            if !entry.text().is_empty() {
                let pass_entry = objs.search_entry_named("password_oper_entry");
                let password = pass_entry.text();
                let image = objs.search_image_named("no_password_warning");
                let pass_warning = objs.search_label_named("no_password_text");
                if password.is_empty() {
                    image.set_visible(true);
                    pass_warning.set_visible(true);
                } else {
                    tx.send(format!(
                        "{}{}{}{}",
                        "OPER ",
                        entry.text().as_str(),
                        " ",
                        password.as_str()
                    ))
                    .unwrap();
                    entry.buffer().set_text("");
                    pass_entry.buffer().set_text("");
                    image.set_visible(false);
                    pass_warning.set_visible(false);
                    objs.search_popover_named("oper_popover").set_visible(false);
                }
            }
        });
    let objs = objects.clone();
    let tx = tx_back.clone();
    objects
        .search_entry_named("password_oper_entry")
        .connect_activate(move |entry| {
            let image = objs.search_image_named("no_password_warning");
            let pass_warning = objs.search_label_named("no_password_text");
            if !entry.text().is_empty() {
                let nick_entry = objs.search_entry_named("nickname_oper_entry");
                let nick = nick_entry.text();
                if !nick.as_str().is_empty() {
                    tx.send(format!(
                        "{}{}{}{}",
                        "OPER ",
                        nick.as_str(),
                        " ",
                        entry.text().as_str()
                    ))
                    .unwrap();
                    entry.buffer().set_text("");
                    nick_entry.buffer().set_text("");
                    image.set_visible(false);
                    pass_warning.set_visible(false);
                    objs.search_popover_named("oper_popover").set_visible(false);
                }
            } else {
                image.set_visible(true);
                pass_warning.set_visible(true);
            }
        });
    let objs = objects.clone();
    let tx = tx_back.clone();
    objects
        .search_button_named("private_flag")
        .connect_clicked(move |_| {
            let switch = objs.search_switch_named("add_remove_oper");
            let channel_name = objs.search_label_named("actual_name").text();
            if switch.is_active() {
                tx.send(format!("{}{}{}", "MODE ", channel_name, " +p"))
                    .unwrap();
            } else {
                tx.send(format!("{}{}{}", "MODE ", channel_name, " -p"))
                    .unwrap();
            }
            objs.search_popover_named("mode_popover").set_visible(false);
        });

    let objs = objects.clone();
    let tx = tx_back.clone();
    objects
        .search_button_named("secret_flag")
        .connect_clicked(move |_| {
            let switch = objs.search_switch_named("add_remove_oper");
            let channel_name = objs.search_label_named("actual_name").text();
            if switch.is_active() {
                tx.send(format!("{}{}{}", "MODE ", channel_name, " +s"))
                    .unwrap();
            } else {
                tx.send(format!("{}{}{}", "MODE ", channel_name, " -s"))
                    .unwrap();
            }
            objs.search_popover_named("mode_popover").set_visible(false);
        });

    let objs = objects.clone();
    let tx = tx_back.clone();
    objects
        .search_button_named("invite-only_flag")
        .connect_clicked(move |_| {
            let switch = objs.search_switch_named("add_remove_oper");
            let channel_name = objs.search_label_named("actual_name").text();
            if switch.is_active() {
                tx.send(format!("{}{}{}", "MODE ", channel_name, " +i"))
                    .unwrap();
            } else {
                tx.send(format!("{}{}{}", "MODE ", channel_name, " -i"))
                    .unwrap();
            }
            objs.search_popover_named("mode_popover").set_visible(false);
        });

    let objs = objects.clone();
    objects
        .search_button_named("operator_privileges")
        .connect_clicked(move |_| {
            let popover = objs.search_popover_named("operator_popover");
            popover.set_visible(true);
        });
    let objs = objects.clone();
    objects
        .search_button_named("ban_nick")
        .connect_clicked(move |_| {
            let popover = objs.search_popover_named("ban_popover");
            popover.set_visible(true);
        });

    let objs = objects.clone();
    let tx = tx_back.clone();
    objects
        .search_entry_named("ban_entry")
        .connect_activate(move |entry| {
            if !entry.text().is_empty() {
                let switch = objs.search_switch_named("add_remove_oper");
                let channel_name = objs.search_label_named("actual_name").text();
                let nick = entry.text();
                if switch.is_active() {
                    tx.send(format!("{}{}{}{}", "MODE ", channel_name, " +b ", nick))
                        .unwrap();
                } else {
                    tx.send(format!("{}{}{}{}", "MODE ", channel_name, " -b ", nick))
                        .unwrap();
                }
                entry.buffer().set_text("");
            }
            let popover = objs.search_popover_named("ban_popover");
            popover.set_visible(false);
        });

    let objs = objects.clone();
    let tx = tx_back.clone();
    objects
        .search_entry_named("operator_privileges_entry")
        .connect_activate(move |entry| {
            if !entry.text().is_empty() {
                let switch = objs.search_switch_named("add_remove_oper");
                let channel_name = objs.search_label_named("actual_name").text();
                let nick = entry.text();
                if switch.is_active() {
                    tx.send(format!("{}{}{}{}", "MODE ", channel_name, " +o ", nick))
                        .unwrap();
                } else {
                    tx.send(format!("{}{}{}{}", "MODE ", channel_name, " -o ", nick))
                        .unwrap();
                }
                entry.buffer().set_text("");
            }
            let popover = objs.search_popover_named("operator_popover");
            popover.set_visible(false);
        });

    let objs = objects.clone();
    let tx = tx_back.clone();
    objects
        .search_toggle_named("away_toggle")
        .connect_toggled(move |button| {
            let away_message = objs.search_label_named("away_message").text();
            let main_entry = objs.search_entry_named("entry_bar");
            let away_display = objs.search_label_named("away_message_display");
            let send_button = objs.search_event_named("send_button");
            if button.is_active() {
                tx.send(format!("{}{}", "AWAY :", away_message)).unwrap();
                main_entry.set_visible(false);
                away_display.set_visible(true);
                send_button.set_visible(false);
            } else {
                tx.send("AWAY".to_owned()).unwrap();
                main_entry.set_visible(true);
                away_display.set_visible(false);
                send_button.set_visible(true);
            }
        });

    let objs = objects.clone();
    objects
        .search_entry_named("away_entry")
        .connect_activate(move |entry| {
            let new_away = entry.text();
            if !new_away.is_empty() {
                objs.search_label_named("away_message")
                    .set_text(entry.text().as_str());
                objs.search_label_named("away_message_display")
                    .set_text(&format!(
                        "{}{}",
                        "You are away with message :",
                        entry.text()
                    ));
                entry.buffer().set_text("");
            }
            objs.search_popover_named("away_popover").set_visible(false);
        });

    let tx = tx_back.clone();
    let objs = objects.clone();
    objects
        .search_entry_named("change_nick_entry")
        .connect_activate(move |entry| {
            let new_nick = entry.text();
            if !new_nick.is_empty() {
                tx.send(format!("{}{}", "NICK ", new_nick)).unwrap();
                objs.search_image_named("nick_used_warning")
                    .set_visible(false);
                objs.search_label_named("nick_used_label")
                    .set_visible(false);
            }
            entry.buffer().set_text("");
        });

    let tx = tx_back.clone();
    let objs = objects.clone();
    objects
        .search_entry_named("message_diffusion_entry")
        .connect_activate(move |entry| {
            let message = objs.search_entry_named("message_diffusion_entry");
            let receivers = objs.search_entry_named("users_diffusion_entry");
            send_broadcast(&tx, &message, &receivers);
            entry.buffer().set_text("");
        });

    let tx = tx_back.clone();
    let objs = objects.clone();
    objects
        .search_entry_named("users_diffusion_entry")
        .connect_activate(move |entry| {
            let message = objs.search_entry_named("message_diffusion_entry");
            let receivers = objs.search_entry_named("users_diffusion_entry");
            send_broadcast(&tx, &message, &receivers);
            entry.buffer().set_text("");
        });

    let tx = tx_back.clone();
    let mut objs = objects.clone();
    objects
        .search_entry_named("who_entry")
        .connect_activate(move |entry| {
            let who_operator_button = objs.search_checkbutton_named("who_operator_button");
            if who_operator_button.is_active() {
                tx.send(format!("WHO {} o", entry.text())).unwrap();
            } else {
                tx.send(format!("WHO {} ", entry.text())).unwrap();
            }

            entry.buffer().set_text("");
        });

    let mut tx = tx_back.clone();
    objs = objects.clone();
    main_window.connect_destroy(move |_| {
        tx.send(format!(
            "{}{}{}",
            "QUIT",
            " ",
            objs.search_label_named("nick_label").text().as_str()
        ))
        .unwrap();
    });
    let user_button = objects.search_toggle_named("users_button");
    let channels_button = objects.search_toggle_named("channels_button");
    let stack = objects.search_stack_named("textview_stack");
    let mut channels_copy = channels_button.clone();
    channels_button.set_active(true);
    let stack_copy = stack.clone();
    user_button.connect_button_press_event(move |button, _| {
        change_stack(button, &channels_copy.clone(), stack_copy.clone(), "Users");
        Inhibit(true)
    });

    channels_copy = channels_button.clone();
    user_button.connect_button_release_event(move |_, _| {
        channels_copy.set_active(false);
        Inhibit(true)
    });
    let users_copy = user_button.clone();
    channels_button.connect_button_press_event(move |button, _| {
        change_stack(button, &user_button.clone(), stack.clone(), "Channels");
        Inhibit(true)
    });
    channels_button.connect_button_release_event(move |_, _| {
        users_copy.set_active(false);
        Inhibit(true)
    });

    let add_chat_event = objects.search_event_named("add_chat_event");
    objs = objects.clone();
    add_chat_event.connect_button_release_event(move |_, _| {
        objs.search_popover_named("add_chats_popover").popup();
        Inhibit(true)
    });

    let objs = objects.clone();
    let filechooser = objs.search_filechooser_named("file_chooser");

    objects
        .search_event_named("send_file")
        .connect_button_press_event(move |_, _| {
            filechooser.show();
            Inhibit(true)
        });
    let filechooser = objs.search_filechooser_named("file_chooser");

    let mut objs = objects.clone();
    tx = tx_back.clone();
    filechooser.connect_delete_event(move |filechooser, _| {
        filechooser.set_visible(false);
        Inhibit(true)
    });

    objects
        .search_button_named("send")
        .connect_clicked(move |_| {
            let path = filechooser.filename().unwrap();
            let nick = objs.search_label_named("actual_name").text();
            let text = objs.search_label_named("server_name").text();
            let (ip, _) = text.split_once(':').unwrap();
            match fs::metadata(path.clone()) {
                Ok(metadata) => {
                    let message = format!(
                        "PRIVMSG {} :DCC SEND {:?} {:?} {} {}",
                        nick,
                        path.as_path().to_str().unwrap().replace(' ', "*"),
                        path.file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .replace(' ', "*"),
                        ip,
                        metadata.len()
                    );
                    tx.send(message).unwrap();
                    filechooser.set_visible(false);
                }
                Err(error) => println!("{:?}", error.to_string()),
            };
        });

    objs = objects.clone();
    tx = tx_back.clone();
    let app = application.clone();
    objects
        .search_button_named("dcc_new_chat")
        .connect_clicked(move |_| {
            let text = objs.search_label_named("server_name").text();
            let (ip, _) = text.split_once(':').unwrap();
            let receiver = objs.search_label_named("actual_name").text().to_string();
            if !receiver.is_empty() {
                tx.send(format!("PRIVMSG {receiver} :DCC CHAT chat {ip}"))
                    .unwrap();
                new_dcc_window(receiver, &tx, &app);
                entry.set_text("");
            }
        });

    tx = tx_back.clone();
    objects
        .search_button_named("franco_button")
        .connect_clicked(move |_| {
            tx.send("PASS contra".to_string()).unwrap();
            tx.send("NICK franco".to_string()).unwrap();
        });
    tx = tx_back.clone();
    objects
        .search_button_named("juan_button")
        .connect_clicked(move |_| {
            tx.send("PASS contra".to_string()).unwrap();
            tx.send("NICK juan".to_string()).unwrap();
        });
    // name FILE_ACCEPTED
    tx = tx_back.clone();
    objs = objects.clone();
    objects
        .search_button_named("accept_file_button")
        .connect_clicked(move |_| {
            let file_dialog = objs.search_messagedialog_named("file_dialog");
            if let Some(text) = file_dialog.secondary_text() {
                let text_splitted: Vec<&str> = text.split_whitespace().collect();
                let sender = text_splitted[0];
                tx.send(format!("FILE_ACCEPTED {} :   vs", sender)).unwrap();

                file_dialog.set_visible(false);
            }
        });
    tx = tx_back.clone();
    objs = objects.clone();
    objects
        .search_button_named("deny_file_button")
        .connect_clicked(move |_| {
            let file_dialog = objs.search_messagedialog_named("file_dialog");
            if let Some(text) = file_dialog.secondary_text() {
                let text_splitted: Vec<&str> = text.split_whitespace().collect();
                let sender = text_splitted[0];
                tx.send(format!("FILE_DENIED {} :", sender)).unwrap();
                file_dialog.set_visible(false);
            }
        });

    objs = objects.clone();
    objects
        .search_popover_named("new_dm_popover")
        .connect_closed(move |_| {
            let listbox = objs.search_listbox_named("direct_msg_listbox");
            for widget in listbox.children() {
                listbox.remove(&widget);
            }
        });

    objs = objects.clone();
    tx = tx_back.clone();
    objects
        .search_button_named("resume")
        .connect_clicked(move |_| {
            let dialog = objs.search_messagedialog_named("incomplete_file");
            let text = dialog.text().unwrap();
            let text_splitted: Vec<&str> = text.split_whitespace().collect();

            let secondary_text = dialog.secondary_text().unwrap();
            let secondary_text_splitted: Vec<&str> = secondary_text.split_whitespace().collect();

            let file_dialog = objs.search_messagedialog_named("file_dialog");
            let file_dialog_text = file_dialog.secondary_text().unwrap();
            let file_text_splitted: Vec<&str> = file_dialog_text.split_whitespace().collect();

            let sender = file_text_splitted[0];
            let filename = text_splitted[1];
            let bytes_written = secondary_text_splitted[0];
            tx.send(format!(
                "PRIVMSG {} :DCC RESUME {} 8048 {}",
                sender, filename, bytes_written
            ))
            .unwrap();
            dialog.set_visible(false);
        });

    objs = objects.clone();
    objects
        .search_button_named("send_err_ok")
        .connect_clicked(move |_| {
            let dialog = objs.search_messagedialog_named("send_err");
            dialog.set_visible(false);
        });

    objs = objects.clone();
    objects
        .search_button_named("not_resume")
        .connect_clicked(move |_| {
            let dialog = objs.search_messagedialog_named("incomplete_file");
            dialog.set_visible(false);
        });

    objs = objects.clone();
    objects
        .search_button_named("download_completed")
        .connect_clicked(move |_| {
            let dialog = objs.search_messagedialog_named("download_complete");
            dialog.set_visible(false);
        });
}
