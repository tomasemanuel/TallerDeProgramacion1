use glib::Object;
use gtk::{
    prelude::*, Application, Button, CheckButton, Entry, EventBox, FileChooserDialog, Fixed, Image,
    Label, ListBox, ListBoxRow, MenuButton, MessageDialog, Popover, RadioButton, ScrolledWindow,
    Stack, Switch, TextView, ToggleButton, Widget, Window,
};

pub trait ObjectOwnExt {
    fn is_named(&self, name: &str) -> bool;
}
impl ObjectOwnExt for Object {
    fn is_named(&self, name: &str) -> bool {
        self.property_value("name").get::<String>().unwrap() == *name
    }
}

pub trait StackOwnExt {
    fn current_textview(&self) -> TextView;
}
impl StackOwnExt for Stack {
    fn current_textview(&self) -> TextView {
        self.visible_child()
            .unwrap()
            .downcast_ref::<gtk::TextView>()
            .unwrap()
            .clone()
    }
}
pub trait VecOwnExt {
    fn search_by_name(&self, name: &str) -> Object;
    fn search_window_named(&self, name: &str) -> Window;
    fn search_button_named(&self, name: &str) -> Button;
    fn search_entry_named(&self, name: &str) -> Entry;
    fn search_radio_button_named(&self, name: &str) -> RadioButton;
    fn search_listbox_named(&self, name: &str) -> ListBox;
    fn search_label_named(&self, name: &str) -> Label;
    fn search_event_named(&self, name: &str) -> EventBox;
    fn search_textview_named(&self, name: &str) -> TextView;
    fn search_menubutton_named(&self, name: &str) -> MenuButton;
    fn search_popover_named(&self, name: &str) -> Popover;
    fn search_image_named(&self, name: &str) -> Image;
    fn search_switch_named(&self, name: &str) -> Switch;
    fn search_toggle_named(&self, name: &str) -> ToggleButton;
    fn search_checkbutton_named(&self, name: &str) -> CheckButton;
    fn search_filechooser_named(&self, name: &str) -> FileChooserDialog;
    fn search_stack_named(&self, name: &str) -> Stack;
    fn search_fixed_named(&self, name: &str) -> Fixed;
    fn search_messagedialog_named(&self, name: &str) -> MessageDialog;
}
impl VecOwnExt for Vec<Object> {
    fn search_filechooser_named(&self, name: &str) -> FileChooserDialog {
        self.search_by_name(name)
            .downcast_ref::<gtk::FileChooserDialog>()
            .unwrap()
            .clone()
    }

    fn search_by_name(&self, name: &str) -> Object {
        let found = self.iter().find(|&object| object.is_named(name));
        if let Some(found) = found {
            (*found).clone()
        } else {
            (*found.unwrap()).clone()
        }
    }
    fn search_label_named(&self, name: &str) -> Label {
        self.search_by_name(name)
            .downcast_ref::<Label>()
            .unwrap()
            .clone()
    }
    fn search_image_named(&self, name: &str) -> Image {
        self.search_by_name(name)
            .downcast_ref::<Image>()
            .unwrap()
            .clone()
    }
    fn search_listbox_named(&self, name: &str) -> ListBox {
        self.search_by_name(name)
            .downcast_ref::<gtk::ListBox>()
            .unwrap()
            .clone()
    }
    fn search_button_named(&self, name: &str) -> Button {
        self.search_by_name(name)
            .downcast_ref::<gtk::Button>()
            .unwrap()
            .clone()
    }
    fn search_entry_named(&self, name: &str) -> Entry {
        self.search_by_name(name)
            .downcast_ref::<gtk::Entry>()
            .unwrap()
            .clone()
    }
    fn search_radio_button_named(&self, name: &str) -> RadioButton {
        self.search_by_name(name)
            .downcast_ref::<gtk::RadioButton>()
            .unwrap()
            .clone()
    }
    fn search_window_named(&self, name: &str) -> Window {
        self.search_by_name(name)
            .downcast_ref::<gtk::Window>()
            .unwrap()
            .clone()
    }
    fn search_event_named(&self, name: &str) -> EventBox {
        self.search_by_name(name)
            .downcast_ref::<EventBox>()
            .unwrap()
            .clone()
    }
    fn search_textview_named(&self, name: &str) -> TextView {
        self.search_by_name(name)
            .downcast_ref::<TextView>()
            .unwrap()
            .clone()
    }
    fn search_menubutton_named(&self, name: &str) -> MenuButton {
        self.search_by_name(name)
            .downcast_ref::<gtk::MenuButton>()
            .unwrap()
            .clone()
    }
    fn search_popover_named(&self, name: &str) -> Popover {
        self.search_by_name(name)
            .downcast_ref::<gtk::Popover>()
            .unwrap()
            .clone()
    }
    fn search_switch_named(&self, name: &str) -> Switch {
        self.search_by_name(name)
            .downcast_ref::<Switch>()
            .unwrap()
            .clone()
    }
    fn search_toggle_named(&self, name: &str) -> ToggleButton {
        self.search_by_name(name)
            .downcast_ref::<ToggleButton>()
            .unwrap()
            .clone()
    }
    fn search_checkbutton_named(&self, name: &str) -> CheckButton {
        self.search_by_name(name)
            .downcast_ref::<CheckButton>()
            .unwrap()
            .clone()
    }
    fn search_stack_named(&self, name: &str) -> Stack {
        self.search_by_name(name)
            .downcast_ref::<Stack>()
            .unwrap()
            .clone()
    }
    fn search_fixed_named(&self, name: &str) -> Fixed {
        self.search_by_name(name)
            .downcast_ref::<Fixed>()
            .unwrap()
            .clone()
    }
    fn search_messagedialog_named(&self, name: &str) -> MessageDialog {
        self.search_by_name(name)
            .downcast_ref::<MessageDialog>()
            .unwrap()
            .clone()
    }
}

pub trait WidgetOwnExt {
    fn is_named(&self, name: &str) -> bool;
    fn to_label(&self) -> Label;
    fn to_listrow(&self) -> ListBoxRow;
    fn to_box(&self) -> gtk::Box;
    fn to_fixed(&self) -> Fixed;
    fn to_event(&self) -> EventBox;
    fn to_image(&self) -> Image;
}
impl WidgetOwnExt for Widget {
    fn is_named(&self, name: &str) -> bool {
        self.property_value("name").get::<String>().unwrap() == *name
    }
    fn to_label(&self) -> Label {
        self.downcast_ref::<Label>().unwrap().clone()
    }
    fn to_listrow(&self) -> ListBoxRow {
        self.downcast_ref::<ListBoxRow>().unwrap().clone()
    }
    fn to_box(&self) -> gtk::Box {
        self.downcast_ref::<gtk::Box>().unwrap().clone()
    }
    fn to_fixed(&self) -> Fixed {
        self.downcast_ref::<Fixed>().unwrap().clone()
    }
    fn to_event(&self) -> EventBox {
        self.downcast_ref::<EventBox>().unwrap().clone()
    }
    fn to_image(&self) -> Image {
        self.downcast_ref::<Image>().unwrap().clone()
    }
}
//
pub trait VecWidgetExt {
    fn search_by_name(&self, name: &str) -> Widget;
    fn search_button_named(&self, name: &str) -> Button;
    fn search_scrolledwindow_named(&self, name: &str) -> ScrolledWindow;
}
impl VecWidgetExt for Vec<Widget> {
    fn search_by_name(&self, name: &str) -> Widget {
        let found = self.iter().find(|&widget| widget.is_named(name));
        if let Some(found) = found {
            (*found).clone()
        } else {
            (*found.unwrap()).clone()
        }
    }
    fn search_button_named(&self, name: &str) -> Button {
        self.search_by_name(name)
            .downcast_ref::<gtk::Button>()
            .unwrap()
            .clone()
    }
    fn search_scrolledwindow_named(&self, name: &str) -> ScrolledWindow {
        self.search_by_name(name)
            .downcast_ref::<ScrolledWindow>()
            .unwrap()
            .clone()
    }
}

pub trait ApplicationOwnExt {
    fn get_window_by_name(&self, name: &str) -> Window;
}
impl ApplicationOwnExt for Application {
    fn get_window_by_name(&self, name: &str) -> Window {
        let windows = self.windows();
        let option = windows
            .iter()
            .find(|&window| window.upcast_ref::<Object>().is_named(name));
        match option {
            Some(window) => window.clone(),
            None => {
                println!("ventanas: {:?}", self.windows());
                option.unwrap().clone()
            }
        }
    }
}

pub trait WindowOwnExt {
    fn get_textview_named(&self, name: &str) -> TextView;
}

impl WindowOwnExt for Window {
    fn get_textview_named(&self, _name: &str) -> TextView {
        let fixed = self.children()[0]
            .downcast_ref::<gtk::Fixed>()
            .unwrap()
            .clone();
        let scrolledwindow = fixed
            .children()
            .search_scrolledwindow_named("dcc_scrolledWindow");
        scrolledwindow.children()[0]
            .downcast_ref::<gtk::TextView>()
            .unwrap()
            .clone()
    }
}

pub trait ListBoxOwnExt {
    fn search_listboxrow_named(&self, name: &str) -> Option<ListBoxRow>;
}
impl ListBoxOwnExt for ListBox {
    fn search_listboxrow_named(&self, name: &str) -> Option<ListBoxRow> {
        let children = self.children();
        for widget in children {
            if widget.is_named(name) {
                return Some(widget.downcast_ref::<ListBoxRow>().unwrap().clone());
            };
        }
        None
    }
}
