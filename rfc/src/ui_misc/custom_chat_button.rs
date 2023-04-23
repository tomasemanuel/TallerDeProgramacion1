use gtk::{
    gdk_pixbuf::Pixbuf,
    traits::{ContainerExt, FixedExt, LabelExt, StyleContextExt, WidgetExt},
    EventBox, Fixed, Image, Label, ListBoxRow,
};

use super::object_ext::WidgetOwnExt;

#[derive(Debug, Clone)]
pub struct CustomButton {
    pub label_event: EventBox,
    pub chat_label: Label,
    pub leave_event: EventBox,
    pub background: Label,
    pub base: Fixed,
    pub container: gtk::Box,
    pub notif_label: Label,
    pub notif_image: Image,
}

impl CustomButton {
    pub fn new(name: String, real_name: &String) -> CustomButton {
        let cstm_button = gtk::Box::builder()
            .name("cstm_button")
            .visible(true)
            .can_focus(false)
            .orientation(gtk::Orientation::Vertical)
            .build();

        let fixed = Fixed::builder()
            .width_request(195)
            .height_request(30)
            .visible(true)
            .can_focus(false)
            .build();
        let label_event = EventBox::builder()
            .width_request(170)
            .height_request(30)
            .visible(true)
            .can_focus(false)
            .name(real_name)
            .build();
        let label = Label::builder()
            .name(real_name)
            .label(name)
            .xalign(0.6)
            .can_focus(false)
            .visible(true)
            .build();

        let image_event = EventBox::builder()
            .width_request(20)
            .height_request(20)
            .visible(false)
            .can_focus(false)
            .name("event_leave")
            .build();
        let image = Image::from_pixbuf(Some(
            &Pixbuf::from_file("src/ui_misc/leave_channel_button.png").unwrap(),
        ));
        let bkgrd_label = Label::builder()
            .name("background_label")
            .width_request(195)
            .height_request(30)
            .visible(true)
            .label("")
            .build();
        let alert_notif = Image::from_pixbuf(Some(
            &Pixbuf::from_file("src/ui_misc/alert_dot.png").unwrap(),
        ));
        alert_notif.set_widget_name("notif_image");
        let notif_label = Label::builder()
            .name("notif_label")
            .visible(false)
            .label("0")
            .build();
        label_event.add(&label);
        image.set_visible(true);
        image_event.add(&image);
        fixed.put(&alert_notif, 145, 5);
        fixed.put(&notif_label, 151, 6);
        fixed.put(&image_event, 168, 5);
        fixed.put(&label_event, 0, 0);
        fixed.put(&bkgrd_label, 0, 0);

        cstm_button.add(&fixed);
        CustomButton {
            label_event,
            chat_label: label,
            leave_event: image_event,
            background: bkgrd_label,
            base: fixed,
            container: cstm_button,
            notif_label,
            notif_image: alert_notif,
        }
    }
    pub fn refresh_label_event(&self) {
        self.base.remove(&self.label_event);
        self.base.put(&self.label_event, 0, 0);
    }
    pub fn from_row(row: ListBoxRow) -> CustomButton {
        let container = row.children()[0].to_box();
        let base = container.children()[0].to_fixed();
        let (mut label_event, mut leave_event, mut background, mut notif_label, mut notif_image) = (
            EventBox::new(),
            EventBox::new(),
            Label::new(None),
            Label::new(None),
            Image::new(),
        );
        for child in base.children() {
            if child.is_named("event_leave") {
                leave_event = child.to_event();
            } else if child.is_named("background_label") {
                background = child.to_label();
            } else if child.is_named("notif_label") {
                notif_label = child.to_label();
            } else if child.is_named("notif_image") {
                notif_image = child.to_image();
            } else {
                label_event = child.to_event();
            }
        }
        let chat_label = label_event.children()[0].to_label();
        notif_label.style_context().add_class("notif");
        CustomButton {
            background,
            label_event,
            chat_label,
            leave_event,
            base,
            container,
            notif_image,
            notif_label,
        }
    }

    pub fn add_notif(&self) {
        self.notif_image.set_visible(true);
        self.notif_label.set_visible(true);
        let new_number = self.notif_label.text().as_str().parse::<u64>().unwrap() + 1;
        self.notif_label.set_text(&new_number.to_string());
        self.chat_label.set_xalign(0.4);
    }
    pub fn remove_notif(&self) -> u64 {
        self.notif_label.set_visible(false);
        self.notif_image.set_visible(false);
        let notifications = self.notif_label.text().as_str().parse::<u64>().unwrap();
        self.notif_label.set_text("0");
        self.chat_label.set_xalign(0.6);
        notifications
    }
}
