mod captcha;
mod device_lock;
mod service;

use std::sync::Arc;

use once_cell::sync::OnceCell;
use relm4::gtk::gdk::Paintable;
use relm4::{
    adw, gtk, Component, ComponentController, ComponentParts, ComponentSender, SimpleComponent,
};

use adw::prelude::*;
use adw::{HeaderBar, Toast, ToastOverlay, Window};

use gtk::gdk_pixbuf::Pixbuf;
use gtk::{Box, Label, MenuButton, Orientation, Picture};

use ricq::Client;
use tokio::task;
use widgets::pwd_login::{self, Input, PasswordLogin, PasswordLoginModel, Payload};

use crate::actions::{AboutAction, ShortcutsAction};
use crate::app::AppMessage;
use crate::db::fs::{download_user_avatar_file, get_user_avatar_path};
use crate::global::WINDOW;

use self::service::{get_login_info, login};

type SmsPhone = Option<String>;
type VerifyUrl = String;
type UserId = i64;
type Password = String;

pub static LOGIN_SENDER: OnceCell<ComponentSender<LoginPageModel>> = OnceCell::new();

#[derive(Debug)]
pub struct LoginPageModel {
    // account: String,
    // password: String,
    // is_login_button_enabled: bool,
    pwd_login: PasswordLogin,
    toast: Option<String>,
}

pub enum LoginPageMsg {
    Login(i64, String),
    LoginSuccessful,
    LoginFailed(String),
    NeedCaptcha(String, Arc<Client>, UserId, Password),
    DeviceLock(VerifyUrl, SmsPhone),
    ConfirmVerification,
    LinkCopied,
}

#[relm4::component(pub)]
impl SimpleComponent for LoginPageModel {
    type Input = LoginPageMsg;
    type Output = AppMessage;
    type InitParams = ();
    type Widgets = LoginPageWidgets;

    fn init(
        _init_params: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        if LOGIN_SENDER.set(sender.clone()).is_err() {
            panic!("failed to initialize login sender");
        }
        let (account, password) = get_login_info();
        let avatar = load_avatar(account.parse().ok(), true);

        let pwd_login = PasswordLoginModel::builder()
            .launch(Payload {
                account: account.parse().ok(),
                password: password.into(),
                avatar: avatar,
            })
            .forward(sender.input_sender(), |out| match out {
                pwd_login::Output::Login { account, pwd } => LoginPageMsg::Login(account, pwd),
            });

        let widgets = view_output!();
        let model = LoginPageModel {
            pwd_login,
            toast: None,
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: LoginPageMsg, sender: &ComponentSender<Self>) {
        use LoginPageMsg::*;
        match msg {
            Login(uin, pwd) => {
                task::spawn(login(uin, pwd));
            }
            LoginSuccessful => {
                sender.output(AppMessage::LoginSuccessful);
            }
            LoginFailed(msg) => {
                self.toast = Some(msg);
            }
            NeedCaptcha(verify_url, client, account, password) => {
                sender.input(LoginPageMsg::LoginFailed(
                    "Need Captcha. See more in the pop-up window.".to_string(),
                ));
                let window = Window::builder()
                    .transient_for(&WINDOW.get().unwrap().window)
                    .default_width(640)
                    .build();

                window.connect_destroy(|_| println!("closed window"));

                let verify_url = verify_url.replace('&', "&amp;");

                let captcha = captcha::CaptchaModel::builder()
                    .launch(captcha::PayLoad {
                        client: Arc::clone(&client),
                        verify_url,
                        window: window.clone(),
                        account,
                        password,
                    })
                    .forward(sender.input_sender(), |output| output);

                window.set_content(Some(captcha.widget()));
                window.present();
            }
            LinkCopied => {
                self.toast.replace("Link Copied".into());
            }
            DeviceLock(verify_url, sms) => {
                let window = Window::builder()
                    .transient_for(&WINDOW.get().unwrap().window)
                    .default_width(640)
                    .build();

                let device_lock = device_lock::DeviceLock::builder()
                    .launch(device_lock::Payload {
                        window: window.clone(),
                        unlock_url: verify_url,
                        sms_phone: sms,
                    })
                    .forward(sender.input_sender(), |output| output);

                window.set_content(Some(device_lock.widget()));
                window.present()
            }
            // TODO: proc follow operate
            ConfirmVerification => self.pwd_login.emit(Input::Login),
        }
    }

    menu! {
        main_menu: {
            "Keyboard Shortcuts" => ShortcutsAction,
            "About Gtk QQ" => AboutAction
        }
    }

    view! {
        login_page = Box {
            set_hexpand: true,
            set_vexpand: true,
            set_orientation: Orientation::Vertical,
            #[name = "headerbar"]
            HeaderBar {
                set_title_widget = Some(&Label) {
                    set_label: "Login"
                },
                pack_end = &MenuButton {
                    set_icon_name: "menu-symbolic",
                    set_menu_model: Some(&main_menu),
                }
            },
            #[name = "toast_overlay"]
            ToastOverlay {
                set_child : Some(pwd_login.widget()),
            }
        }
    }

    fn pre_view(&self, widgets: &mut Self::Widgets, sender: &ComponentSender<Self>) {
        if let Some(content) = &self.toast {
            widgets.toast_overlay.add_toast(&Toast::new(content));
        }
    }
}

fn load_avatar(account: Option<i64>, auto_download: bool) -> Option<Paintable> {
    account
        .map(|uin| (uin, get_user_avatar_path(uin)))
        .and_then(|(uin, path)| {
            if path.exists() {
                Some(path)
            } else {
                if auto_download {
                    task::spawn(download_user_avatar_file(uin));
                }
                None
            }
        })
        .and_then(|path| Pixbuf::from_file_at_size(path, 96, 96).ok())
        .map(|pix| Picture::for_pixbuf(&pix))
        .and_then(|pic| pic.paintable())
}
