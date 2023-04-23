use gio::prelude::*;
use glib::Object;
use gtk::gdk::{Event, Screen};
use gtk::{prelude::*, MessageDialog, TextBuffer};
use gtk::{Application, Builder, CssProvider, StyleContext};
use rfc::answers::ban_answer::BanAnswer;
use rfc::answers::file_info_answer::FileInfoAnswer;
use rfc::answers::file_request_answer::FileRequestAnswer;
use rfc::answers::invite_answer::InviteAnswer;
use rfc::answers::join_answer::JoinAnswer;
use rfc::answers::kick_answer::KickAnswer;
use rfc::answers::list_answer::ListAnswer;
use rfc::answers::mode_answer::ModeAnswer;
use rfc::answers::names_answer::NamesAnswer;
use rfc::answers::nick_answer::NickAnswer;
use rfc::answers::p2p_answer::P2PAnswer;
use rfc::answers::part_answer::PartAnswer;
use rfc::answers::topic_answer::TopicAnswer;
use rfc::answers::who_answer::WhoAnswer;
use rfc::answers::who_is_answer::WhoIsAnswer;
use rfc::client_parser::*;
use rfc::ui_misc::dcc_window::{dcc_close_window, new_dcc_window};
use rfc::ui_misc::dm_window::new_dm_window_init;
use rfc::ui_misc::object_ext::VecOwnExt;
use rfc::ui_misc::object_ext::{ApplicationOwnExt, ListBoxOwnExt, WindowOwnExt};
use rfc::ui_misc::private_handler::{change_channels, private_handler};
use rfc::ui_misc::registration_window::registration_successful;
use std::collections::HashMap;
use std::sync::RwLock;
use std::{
    sync::{
        mpsc::{self, Sender},
        Arc,
    },
    thread,
};
mod cliente;
use crate::cliente::*;
use rfc::ui_misc::login_window::*;
use rfc::ui_misc::main_window::*;

fn alert_window(secondary_text: &str) {
    let alert_window = MessageDialog::builder()
        .text("Error")
        .secondary_text(secondary_text)
        .build();
    alert_window.set_position(gtk::WindowPosition::Center);
    alert_window.add_button("Ok", gtk::ResponseType::Close);
    alert_window.connect_response(move |a_window, response| {
        if response == gtk::ResponseType::Close {
            a_window.close();
        }
    });
    alert_window.show_all();
}

fn nicknameinuse_error_hand(application: &gtk::Application, objects: &Vec<Object>) {
    let nick_label = objects.search_label_named("nick_label");
    if nick_label.text().is_empty() {
        let main_window = application.get_window_by_name("Main_window");
        main_window.set_visible(false);
        alert_window("El nombre de usuario que ha ingresado ya se encuentra registrado");
    } else {
        objects
            .search_image_named("nick_used_warning")
            .set_visible(true);
        objects
            .search_label_named("nick_used_label")
            .set_visible(true);
    }
}

fn banned_from_channel() {
    alert_window("You are attempting to enter a channel were you were banned");
}
fn not_an_operator() {
    alert_window("You are attempting to modify outside your reach. You are not an operator of this channel/server");
}
fn invite_only_channel() {
    alert_window("You are attempting to enter a channel that you were not invited");
}

fn error_handler(err_string: String, app: &gtk::Application, objects: &Vec<Object>) {
    match err_string.as_str() {
        "ERR_NICKNAMEINUSE" => nicknameinuse_error_hand(app, objects),
        "ERR_CANNOTCONNECTTOSERVER" => app.quit(),
        "ERR_BANNEDFROMCHAN" => banned_from_channel(),
        "ERR_NOTANOPERATOR" => not_an_operator(),
        "ERR_INVITEONLYCHAN" => invite_only_channel(),
        _ => println!("ERROR:  {err_string}"),
    }
}

fn names_handler(
    objects: &Vec<Object>,
    names_msg: NamesAnswer,
    chats: Arc<RwLock<HashMap<String, TextBuffer>>>,
    tx: &Sender<String>,
) {
    let mut filtered_nicklist: Vec<&String> = names_msg
        .nickname_list
        .iter()
        .filter(|nick| !(*nick).is_empty())
        .collect();
    filtered_nicklist.sort();
    filtered_nicklist.dedup();
    let mut final_nicklist: Vec<String> = Vec::new();
    filtered_nicklist
        .iter()
        .for_each(|string| final_nicklist.push((**string).clone()));
    let label_request = objects.search_label_named("name_request");
    let label_textview = objects.search_label_named("view_marker");
    if label_request.text().is_empty() {
        new_dm_window_init(objects, final_nicklist, Arc::clone(&chats), tx);
    } else if label_textview.text().starts_with("From channel") {
        let textview = objects.search_textview_named("names_channel_textview");
        let buff = textview.buffer().unwrap();
        buff.set_text("");
        buff.insert_at_cursor("Users connected:\n");
        final_nicklist
            .iter()
            .for_each(|nick| buff.insert_at_cursor(&format!("{}{}", nick, "\n")));
    }
}

fn part_handler(
    objects: &Vec<Object>,
    part_msg: PartAnswer,
    chats: Arc<RwLock<HashMap<String, TextBuffer>>>,
) {
    for channel_name in part_msg.from_channels {
        chats.write().unwrap().remove(&channel_name);
        let channel_listbox = objects.search_listbox_named("channel_listbox");
        if let Some(parent) = channel_listbox.search_listboxrow_named(&channel_name) {
            remove_chat(channel_listbox, parent, objects);
        }
    }
}

fn list_handler(objects: &Vec<Object>, list_msg: ListAnswer) {
    let buffer = objects
        .search_textview_named("list_textview")
        .buffer()
        .unwrap();
    buffer.set_text("");
    if !list_msg.channel_list.is_empty() {
        for channel in list_msg.channel_list {
            if channel.contains('&') {
                buffer.insert_at_cursor(&format!("{}{}", channel.replace('&', ""), " (public)\n"));
            } else {
                buffer.insert_at_cursor(&format!(
                    "{}{}",
                    channel.replace('#', ""),
                    " (invite-only)\n"
                ));
            }
        }
    }
}

fn topic_handler(objects: &Vec<Object>, topic_msg: TopicAnswer) {
    let topic_display = objects.search_event_named("topic");
    match topic_msg.topic {
        Some(topic) => topic_display.set_tooltip_text(Some(&topic)),
        None => {
            println!("entre con none");
            topic_display.set_tooltip_text(Some("This channel doesn't have a topic"))
        }
    }
}
fn invite_handler(invite_msg: InviteAnswer, tx: &Sender<String>) {
    if !invite_msg.channel_name.is_empty()
        && !invite_msg.nick.is_empty()
        && tx
            .send(format!(
                "{}{}{}{}{}",
                "PRIVMSG ", invite_msg.nick, " :", "RPL_INVITING ", invite_msg.channel_name
            ))
            .is_err()
    {
        println!("ERROR CON EL PRIVMSG");
    }
}

fn kick_handler(
    objects: &Vec<Object>,
    kick_msg: KickAnswer,
    textbuffers: Arc<RwLock<HashMap<String, TextBuffer>>>,
) {
    textbuffers.write().unwrap().remove(&kick_msg.channel);
    let channel_listbox = objects.search_listbox_named("channel_listbox");
    if let Some(parent) = channel_listbox.search_listboxrow_named(&kick_msg.channel) {
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
    let channel_name = kick_msg.channel.replace(['#', '&'], " ");
    let alert_window = MessageDialog::builder()
        .text(format!(
            "{}{}",
            "You were kicked of channel: ", channel_name
        ))
        .width_request(100)
        .height_request(80)
        .build();
    alert_window.set_position(gtk::WindowPosition::Center);
    alert_window.add_button("Ok", gtk::ResponseType::Close);
    alert_window.connect_response(move |a_window, response| {
        if response == gtk::ResponseType::Close {
            a_window.close();
        }
    });
    if let Some(comment) = kick_msg.comment {
        alert_window.set_secondary_text(Some(&comment));
    }
    alert_window.show_all();
}

fn register_handler(objects: &Vec<Object>) {
    //TIENE CODIGO REPETIDO DE ARRIBA

    let alert_window = MessageDialog::builder()
        .text("Registration successful")
        .width_request(100)
        .height_request(80)
        .build();

    alert_window.set_position(gtk::WindowPosition::Center);
    alert_window.add_button("Ok", gtk::ResponseType::Close);
    alert_window.connect_response(move |a_window, response| {
        if response == gtk::ResponseType::Close {
            a_window.close();
        }
    });
    registration_successful(objects)
}

fn nick_handler(objects: &Vec<Object>, nick_msg: NickAnswer) {
    if !nick_msg.new_nick.is_empty() {
        objects
            .search_label_named("nick_label")
            .set_text(nick_msg.new_nick.as_str());
        objects
            .search_popover_named("nick_popover")
            .set_visible(false);
    }
}
fn whois_handler(objects: &Vec<Object>, whois_msg: WhoIsAnswer) {
    let nick = objects.search_label_named("actual_name").text();
    let operator = match whois_msg.operator {
        true => "is an Operator",
        false => "is not an Operator",
    };
    let away = match whois_msg.away {
        true => "is away",
        false => "is active",
    };
    if let Some(buffer) = objects
        .search_textview_named("names_channel_textview")
        .buffer()
    {
        buffer.insert_at_cursor(&format!("{}{}\n", "Nick : ", nick));
        buffer.insert_at_cursor(&format!("{nick} {operator}\n"));
        buffer.insert_at_cursor(&format!("{nick} {away}\n"));
        if let Some(channels) = whois_msg.joined_channels {
            buffer.insert_at_cursor(&format!("{}{}", "Channels:", '\n'));
            let split_channels = channels.split(',');
            let array_channels = split_channels.collect::<Vec<&str>>();
            array_channels.iter().for_each(|channel| {
                let channel = channel.replace(['&', '#'], "");
                buffer.insert_at_cursor(&format!("  *{channel}\n"))
            });
        };
    }
    if whois_msg.away {
        objects.search_image_named("sleeping").set_visible(true);
    } else {
        objects.search_image_named("sleeping").set_visible(false);
    }
}

fn ban_handler(objects: &Vec<Object>, ban_msg: BanAnswer) {
    let textview = objects.search_textview_named("main_textview");
    if let Some(buffer) = textview.buffer() {
        buffer.insert_at_cursor("From server: After BAN, banlist:\n");
        ban_msg
            .ban_list
            .iter()
            .for_each(|nick| buffer.insert_at_cursor(&format!("{nick}\n")));
    }
}
fn who_handler(objects: &Vec<Object>, who_msg: WhoAnswer) {
    let textview = objects.search_textview_named("who_textview");
    if let Some(buffer) = textview.buffer() {
        buffer.insert_at_cursor(&who_msg.matches);
    }
}

fn mode_handler(
    objects: &Vec<Object>,
    mode_msg: ModeAnswer,
    textbuffers: Arc<RwLock<HashMap<String, TextBuffer>>>,
    tx: &Sender<String>,
) {
    match mode_msg.flag.as_str() {
        "+i" => {
            let new_name = mode_msg.channel.replace('&', "#");
            tx.send(format!(
                "PRIVMSG {} :MODE +i {}",
                new_name, mode_msg.channel
            ))
            .unwrap();
            change_channels(mode_msg.channel, new_name, objects, textbuffers, tx)
        }
        "-i" => {
            let new_name = mode_msg.channel.clone().replace('#', "&");
            tx.send(format!(
                "PRIVMSG {} :MODE -i {}",
                new_name, mode_msg.channel
            ))
            .unwrap();
            change_channels(mode_msg.channel, new_name, objects, textbuffers, tx)
        }
        _ => println!(),
    }
}

fn dcc_msg_handler(app: &gtk::Application, dcc_msg: P2PAnswer) {
    let window = app.get_window_by_name(&dcc_msg.sender);
    let textbuffer = window.get_textview_named("dcc_textview").buffer().unwrap();
    let mut iter = textbuffer.end_iter();
    if textbuffer.start_iter() != iter {
        textbuffer.insert(&mut iter, "\n");
    }
    textbuffer.insert(&mut iter, &format!("{}:", dcc_msg.sender));
    textbuffer.insert(&mut iter, &dcc_msg.message);
}

fn file_request_handler(objects: &Vec<Object>, filereq_answer: FileRequestAnswer) {
    let dialog = objects.search_messagedialog_named("file_dialog");
    let file_name = filereq_answer.file_name.replace(' ', "_");
    dialog.set_secondary_text(Some(&format!(
        "{} would like to send you the file named: {} (Size:{})",
        filereq_answer.file_owner, file_name, filereq_answer.file_size
    )));
    dialog.show();
}
fn file_info_handler(objects: &Vec<Object>, file_info: FileInfoAnswer) {
    if file_info.bytes_total != file_info.bytes_transfered {
        let dialog = objects.search_messagedialog_named("incomplete_file");
        dialog.set_text(Some(&format!(
            "File {} has not been completely downloaded",
            file_info.file_name
        )));
        dialog.set_secondary_text(Some(&format!(
            "{} bytes have been transfered of a file with a total of {} bytes ",
            file_info.bytes_transfered, file_info.bytes_total
        )));
        dialog.show();
    } else {
        let dialog = objects.search_messagedialog_named("download_complete");
        dialog.set_text(Some(&format!(
            "File {} has been completely downloaded",
            file_info.file_name
        )));
        dialog.show();
    }
}

fn send_err_handler(objects: &Vec<Object>, _send_err: String) {
    let dialog = objects.search_messagedialog_named("send_err");
    dialog.set_text(Some("Error sending file"));
    dialog.show();
}

fn p2p_err_handler(objects: &Vec<Object>, p2p_err: String, application: &gtk::Application) {
    let window = application.get_window_by_name(&p2p_err);
    window.close();
    let dialog = objects.search_messagedialog_named("send_err");
    dialog.set_text(Some("p2p error"));
    dialog.show();
}

fn join_handler(
    objects: &Vec<Object>,
    join_struct: JoinAnswer,
    textbuffers: Arc<RwLock<HashMap<String, TextBuffer>>>,
    tx_back: &Sender<String>,
) {
    let channel_listbox = objects.search_listbox_named("channel_listbox");
    let label = add_new_channel(
        &join_struct.channel_name,
        Arc::clone(&textbuffers),
        &channel_listbox,
        &String::from("From channel: "),
        objects,
        tx_back,
        None,
    );
    if let Some(cstm_button) = label {
        cstm_button.label_event.emit_by_name_with_values(
            "button-press-event",
            &[Event::new(gtk::gdk::EventType::ButtonPress).to_value()],
        );
    };
    objects.search_popover_named("new_dm_popover").popdown();
    objects.search_popover_named("add_chats_popover").popdown();
    let user_button = objects.search_toggle_named("users_button");
    let channels_button = objects.search_toggle_named("channels_button");
    let stack = objects.search_stack_named("textview_stack");
    change_stack(&channels_button, &user_button, stack, "Channels");
    user_button.set_active(false);
}

fn spawn_local_handler(
    application: &gtk::Application,
    rx_on_gui: glib::Receiver<Answer>,
    tx_back: Sender<String>,
    objects: &[Object],
) {
    let textbuffers: Arc<RwLock<HashMap<String, TextBuffer>>> =
        Arc::new(RwLock::new(HashMap::new()));
    main_window(objects.to_owned(), &tx_back, application);
    let app = application.clone();
    let objects = objects.to_owned();
    rx_on_gui.attach(None, move |answer| {
        match answer {
            Answer::ErrMsg(string) => error_handler(string, &app, &objects),
            Answer::Join(join_struct) => {
                join_handler(&objects, join_struct, Arc::clone(&textbuffers), &tx_back)
            }
            Answer::Welcome(nick) => show_main_window(&objects, &tx_back, nick),
            Answer::PrivMsg(privmsg) => {
                private_handler(&objects, privmsg, Arc::clone(&textbuffers), &tx_back)
            }

            Answer::ChannelList(channel_list) => {
                add_channels(&objects, channel_list, Arc::clone(&textbuffers), &tx_back)
            }

            Answer::List(list_msg) => list_handler(&objects, list_msg),
            Answer::Names(names_msg) => {
                names_handler(&objects, names_msg, Arc::clone(&textbuffers), &tx_back)
            }
            Answer::Quit(_quit_msg) => app.quit(),
            Answer::Ban(ban_msg) => ban_handler(&objects, ban_msg),
            Answer::Server(_server_msg) => println!(),
            Answer::Part(part_msg) => part_handler(&objects, part_msg, Arc::clone(&textbuffers)),
            Answer::Topic(topic_msg) => topic_handler(&objects, topic_msg),
            Answer::Invite(invite_msg) => invite_handler(invite_msg, &tx_back),
            Answer::WhoIs(whois_msg) => whois_handler(&objects, whois_msg),
            Answer::Kick(kick_msg) => kick_handler(&objects, kick_msg, Arc::clone(&textbuffers)),
            Answer::Register(_register_msg) => register_handler(&objects),
            Answer::Nick(nick_msg) => nick_handler(&objects, nick_msg),
            Answer::Mode(mode_msg) => {
                mode_handler(&objects, mode_msg, Arc::clone(&textbuffers), &tx_back)
            }
            Answer::Who(who_msg) => who_handler(&objects, who_msg),
            Answer::P2PChat(p2pchat) => dcc_msg_handler(&app, p2pchat),
            Answer::StartDCC(start_msg) => new_dcc_window(start_msg.nick, &tx_back, &app),
            Answer::FileRequest(file_req) => file_request_handler(&objects, file_req),
            Answer::DCClose(close_answer) => dcc_close_window(&app, close_answer),
            Answer::FileInfo(file_info) => file_info_handler(&objects, file_info),
            Answer::SendError(send_error) => send_err_handler(&objects, send_error),
            Answer::P2PError(p2p_err) => p2p_err_handler(&objects, p2p_err, &app),
        }
        glib::Continue(true)
    });
}

fn build_ui(application: &gtk::Application, glade_src: &str) {
    let builder: Builder = Builder::from_string(glade_src);

    let (tx_to_client, rx_client_from_gui) = mpsc::channel(); //escribe ui, recibe cliente
    let (tx_to_gui, rx_on_gui) = glib::MainContext::channel(glib::PRIORITY_DEFAULT); //escribe cliente

    let objects = builder.objects();

    login_window_init(application, &objects, &tx_to_client);

    spawn_local_handler(application, rx_on_gui, tx_to_client, &objects);

    let _join_handle: thread::JoinHandle<_> = thread::spawn(move || {
        init_client(rx_client_from_gui, tx_to_gui);
    });
}

fn load_css() {
    let provider = CssProvider::new();
    provider
        .load_from_data(include_str!("ui_misc/style.css").as_bytes())
        .expect("Could not read style file");
    StyleContext::add_provider_for_screen(
        &Screen::default().expect("Could not connect to a screen"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn main() {
    let glade_src = include_str!("ui_misc/ui.glade");

    let application = Application::builder().build();

    application.connect_startup(|_| load_css());
    application.connect_activate(move |app| build_ui(app, glade_src));
    application.run();
}
