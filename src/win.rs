// Copyright 2024 the Cartero authors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use crate::app::CarteroApplication;
use crate::components::rowheader::RowHeader;
use crate::config::VERSION;
use glib::{GString, Object};
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use gtk4::{gio, glib, StringObject};
use gtk4::{prelude::*, ListBox};

fn mock_map() -> HashMap<String, String> {
    let user_agent = format!("cartero/{})", VERSION);
    let mut map = HashMap::new();
    map.insert(String::from("Accept"), String::from("text/html"));
    map.insert(String::from("User-Agent"), user_agent);
    map.insert(String::from("Accept-Encoding"), String::from("bzip"));
    map
}

fn populate_list(list_box: &ListBox, map: &HashMap<String, String>) {
    for (name, value) in map.iter() {
        let rowheader = RowHeader::new(name, value);
        list_box.append(&rowheader);
    }
}

mod imp {
    use std::collections::HashMap;
    use std::io::Read;

    use crate::client::build_request;
    use crate::client::{Request, RequestMethod};
    use crate::components::response_panel::ResponsePanel;
    use glib::subclass::object::*;
    use glib::subclass::types::*;
    use glib::subclass::InitializingObject;
    use gtk4::subclass::widget::*;
    use gtk4::{
        subclass::{
            application_window::ApplicationWindowImpl, widget::WidgetImpl, window::WindowImpl,
        },
        CompositeTemplate, TemplateChild,
    };
    use isahc::RequestExt;

    use super::populate_list;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/es/danirod/Cartero/main_window.ui")]
    pub struct CarteroWindow {
        #[template_child(id = "send")]
        pub send_button: TemplateChild<gtk4::Button>,

        #[template_child]
        pub request_headers: TemplateChild<gtk4::ListBox>,

        #[template_child(id = "method")]
        pub request_method: TemplateChild<gtk4::DropDown>,

        #[template_child(id = "url")]
        pub request_url: TemplateChild<gtk4::Entry>,

        #[template_child]
        pub request_body: TemplateChild<sourceview5::View>,

        #[template_child]
        pub response: TemplateChild<ResponsePanel>,
    }

    #[gtk4::template_callbacks]
    impl CarteroWindow {
        #[template_callback]
        fn on_send_request(&self, _: &gtk4::Button) {
            let obj = &self.obj();
            let url = obj.request_url();
            let body = obj.request_body();
            let method = {
                let str = String::from(obj.request_method());
                RequestMethod::try_from(str.as_str())
            };

            let request = match method {
                Ok(method) => {
                    let request = Request {
                        url: String::from(url),
                        method,
                        body: String::from(body),
                        headers: HashMap::new(),
                    };
                    build_request(&request)
                }
                Err(_) => {
                    println!("Error: invalid method");
                    return;
                }
            };

            let response = match request {
                Err(_) => {
                    println!("Error: invalid request");
                    return;
                }
                Ok(req) => req.send(),
            };

            match response {
                Err(_) => {
                    println!("Error: invalid response");
                }
                Ok(mut rsp) => {
                    let mut body_content = String::new();
                    let _ = rsp.body_mut().read_to_string(&mut body_content);
                    println!("{:?}", rsp);
                    println!("{}", body_content);
                }
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CarteroWindow {
        const NAME: &'static str = "CarteroWindow";
        type Type = super::CarteroWindow;
        type ParentType = gtk4::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CarteroWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let fake_headers = super::mock_map();
            populate_list(&self.request_headers, &fake_headers);
        }
    }

    impl WidgetImpl for CarteroWindow {}

    impl WindowImpl for CarteroWindow {}

    impl ApplicationWindowImpl for CarteroWindow {}
}

glib::wrapper! {
    pub struct CarteroWindow(ObjectSubclass<imp::CarteroWindow>)
        @extends gtk4::Widget, gtk4::Window, gtk4::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl CarteroWindow {
    pub fn new(app: &CarteroApplication) -> Self {
        Object::builder().property("application", Some(app)).build()
    }

    pub fn request_url(&self) -> GString {
        self.imp().request_url.text()
    }

    pub fn request_method(&self) -> GString {
        let method = &self.imp().request_method;
        method
            .selected_item()
            .unwrap()
            .downcast::<StringObject>()
            .unwrap()
            .string()
    }

    pub fn request_body(&self) -> GString {
        let body = &self.imp().request_body;
        let (start, end) = body.buffer().bounds();
        body.buffer().text(&start, &end, true)
    }
}
