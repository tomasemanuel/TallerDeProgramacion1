use glib::{clone, Object};
use gtk::prelude::*;
use std::sync::mpsc::Sender;

use super::object_ext::VecOwnExt;

pub fn registration_management(
    nick_entry: &gtk::Entry,
    pass_entry: &gtk::Entry,
    user_entry: &gtk::Entry,
    tx: &Sender<String>,
) {
    let mut nickname = nick_entry.buffer().text();
    nickname.insert_str(0, "NICK ");
    let mut password = pass_entry.buffer().text();
    password.insert_str(0, "PASS ");

    let mut username = user_entry.buffer().text();
    username.insert_str(0, "USER ");
    username.push(' ');

    let host = "host";
    let len = username.len();
    username.insert_str(len, host);
    tx.send(password).unwrap();
    tx.send(username).unwrap();
    tx.send(nickname).unwrap();
}

pub fn show_registration_window(objects: &Vec<Object>, tx: &Sender<String>) {
    let window = objects.search_window_named("registration_window");
    let login_window = objects.search_window_named("login_window");
    let register_button = objects.search_button_named("regis_button");
    let nick_entry = objects.search_entry_named("regis_nick");
    let pass_entry = objects.search_entry_named("regis_pass");
    let user_entry = objects.search_entry_named("regis_user");
    login_window.set_visible(false);
    window.show_all();

    let tx1 = tx.clone();
    register_button.connect_clicked(clone!(@weak nick_entry,@weak pass_entry => move |_|{
        registration_management(&nick_entry,&pass_entry,&user_entry,&tx1);
    }));

    window.show_all();
}

pub fn registration_successful(objects: &Vec<Object>) {
    let window = objects.search_window_named("registration_window");
    window.set_visible(false);
    let main_window = objects.search_window_named("Main_window");
    main_window.set_visible(true);
}
