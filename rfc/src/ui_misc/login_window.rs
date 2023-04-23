use glib::{clone, Object};
use gtk::prelude::*;
use std::sync::mpsc::Sender;

use super::{object_ext::VecOwnExt, registration_window::show_registration_window};
//use crate::object_ext:: VecOwnExt;

fn login_management(nick_entry: &gtk::Entry, pass_entry: &gtk::Entry, tx: &Sender<String>) {
    let mut password = pass_entry.buffer().text();
    password.insert_str(0, "PASS ");
    let mut nickname = nick_entry.buffer().text();
    nickname.insert_str(0, "NICK ");

    tx.send(password).unwrap();
    tx.send(nickname).unwrap();
}
pub fn login_window_init(
    application: &gtk::Application,
    objects: &Vec<Object>,
    tx: &Sender<String>,
) {
    let main_window = objects.search_window_named("Main_window");
    let window = objects.search_window_named("login_window");
    let registration_window = objects.search_window_named("registration_window");
    let login_button = objects.search_button_named("login_button");
    let register_button = objects.search_button_named("register_button");
    let nick_entry = objects.search_entry_named("nick_entry");
    let pass_entry = objects.search_entry_named("pass_entry");

    window.set_application(Some(application));
    main_window.set_application(Some(application));
    registration_window.set_application(Some(application));
    glib::set_application_name("Grupo Fiubense");

    let tx1 = tx.clone();

    login_button.connect_clicked(clone!(@weak nick_entry,@weak pass_entry => move |_|{
        login_management(&nick_entry,&pass_entry,&tx1);
    }));

    let tx2 = tx.clone();
    pass_entry.connect_activate(clone!(@weak nick_entry,@weak pass_entry => move |_|{
        login_management(&nick_entry,&pass_entry,&tx2);
    }));

    let tx3 = tx.clone();
    let obj = objects.clone();
    register_button.connect_clicked(clone!(@weak nick_entry,@weak pass_entry => move |_|{
        show_registration_window(&obj ,&tx3);
    }));
    window.set_visible(true);
}
