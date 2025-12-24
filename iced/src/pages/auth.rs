use iced::{
    Element, Font, Padding, Task,
    alignment::Vertical,
    widget::{button, center, column, container, row, rule, scrollable, space, text, text_input},
};
use ochat_common::{
    data::{Data, RequestType},
    save_token,
};
use ochat_types::{
    WORD_ART,
    user::{SigninData, SignupData, Token},
};

use crate::{
    Application, DATA, InputMessage, Message,
    font::{BODY_SIZE, HEADER_SIZE, SUB_HEADING_SIZE, get_bold_font},
    style,
};

#[derive(Debug, Clone)]
pub enum AuthMessage {
    InstanceUrl(InputMessage),
    UpdateEmail(String),
    UpdateName(String),
    UpdatePass(String),
    UpdatePass2(String),
    ChangePage,
    Submit,
    SignedIn(String),
}

impl AuthMessage {
    pub fn handle(self, app: &mut Application) -> Task<Message> {
        macro_rules! UpdateLoginDetail {
            ($prop:ident) => {{
                app.view_data.auth.$prop = $prop;
                Task::none()
            }};
        }
        match self {
            Self::InstanceUrl(InputMessage::Update(url)) => {
                app.view_data.auth.instance_url = url;
                Task::none()
            }
            Self::InstanceUrl(_) => {
                let instance = app.view_data.auth.instance_url.clone();
                Task::done(Message::Cache(crate::CacheMessage::SetInstanceUrl(
                    instance,
                )))
            }
            Self::UpdateEmail(email) => UpdateLoginDetail!(email),
            Self::UpdateName(name) => UpdateLoginDetail!(name),
            Self::UpdatePass(pass) => UpdateLoginDetail!(pass),
            Self::UpdatePass2(pass2) => UpdateLoginDetail!(pass2),
            Self::ChangePage => {
                app.view_data.auth.page = match app.view_data.auth.page {
                    Page::Signin => Page::Signup,
                    Page::Signup => Page::Signin,
                };
                Task::none()
            }
            Self::SignedIn(x) => {
                app.popups.clear();
                let url = {
                    let mut data = DATA.write().unwrap();
                    data.jwt = Some(x.clone());
                    data.instance_url.clone()
                };
                save_token(&Token { token: x.clone() });

                Task::future(async {
                    if let Ok(x) = Data::get(url, Some(x)).await {
                        *DATA.write().unwrap() = x;
                    }
                    Message::None
                })
                .chain(Application::update_data_cache())
            }
            Self::Submit => {
                let view = &app.view_data.auth;

                if view.name.is_empty() {
                    return Task::done(Message::Err(String::from("Username is empty")));
                }

                if view.pass.is_empty() {
                    return Task::done(Message::Err(String::from("Password is empty")));
                }

                match view.page {
                    Page::Signin => {
                        let data = SigninData {
                            name: view.name.clone(),
                            password: view.pass.clone(),
                        };

                        Task::future(async move {
                            let req = DATA.read().unwrap().to_request();
                            match req
                                .make_request::<Token, SigninData>(
                                    "signin/",
                                    &data,
                                    RequestType::Post,
                                )
                                .await
                            {
                                Ok(jwt) => Message::Auth(AuthMessage::SignedIn(jwt.token)),
                                Err(e) => Message::Err(e),
                            }
                        })
                    }
                    Page::Signup => {
                        if view.email.is_empty() {
                            return Task::done(Message::Err(String::from("Email is empty")));
                        }

                        if view.pass2.is_empty() {
                            return Task::done(Message::Err(String::from(
                                "Please confirm password",
                            )));
                        }

                        if view.pass != view.pass2 {
                            return Task::done(Message::Err(String::from(
                                "Passwords do not match",
                            )));
                        }

                        let data = SignupData {
                            name: view.name.clone(),
                            email: view.email.clone(),
                            password: view.pass.clone(),
                        };

                        Task::future(async move {
                            let req = DATA.read().unwrap().to_request();
                            match req
                                .make_request::<Token, SignupData>(
                                    "signup/",
                                    &data,
                                    RequestType::Post,
                                )
                                .await
                            {
                                Ok(jwt) => Message::Auth(AuthMessage::SignedIn(jwt.token)),
                                Err(e) => Message::Err(e),
                            }
                        })
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthPage {
    pub page: Page,
    pub instance_url: String,
    pub email: String,
    pub name: String,
    pub pass: String,
    pub pass2: String,
}

impl Default for AuthPage {
    fn default() -> Self {
        Self {
            page: Page::Signin,
            instance_url: String::from("http://localhost:1212"),
            email: String::new(),
            name: String::new(),
            pass: String::new(),
            pass2: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum Page {
    #[default]
    Signin,
    Signup,
}

impl AuthPage {
    pub fn view<'a>(&self, _app: &'a Application) -> Element<'a, Message> {
        let heading = |txt: &'static str| {
            text(txt)
                .size(BODY_SIZE)
                .font(get_bold_font())
                .style(style::text::primary)
        };
        let sub_heading = |txt: &'static str| text(txt).size(BODY_SIZE).style(style::text::text);
        let banner = text(WORD_ART)
            .font(Font::MONOSPACE)
            .style(style::text::primary);

        let instance = style::svg_input::primary(
            Some(String::from("link.svg")),
            text_input("Enter the instance url...", &self.instance_url)
                .on_input(move |x| Message::Auth(AuthMessage::InstanceUrl(InputMessage::Update(x))))
                .on_submit(Message::Auth(AuthMessage::InstanceUrl(
                    InputMessage::Submit,
                ))),
            SUB_HEADING_SIZE,
        );

        let name = style::svg_input::primary(
            Some(String::from("account.svg")),
            text_input("Enter your username...", &self.name)
                .on_input(move |x| Message::Auth(AuthMessage::UpdateName(x))),
            SUB_HEADING_SIZE,
        );

        let email = style::svg_input::primary(
            Some(String::from("mail.svg")),
            text_input("Enter your email...", &self.email)
                .on_input(move |x| Message::Auth(AuthMessage::UpdateEmail(x))),
            SUB_HEADING_SIZE,
        );

        let pass1 = style::svg_input::primary(
            Some(String::from("pass.svg")),
            text_input("Enter your password...", &self.pass)
                .on_input(move |x| Message::Auth(AuthMessage::UpdatePass(x)))
                .secure(true),
            SUB_HEADING_SIZE,
        );

        let pass2 = style::svg_input::primary(
            Some(String::from("pass2.svg")),
            text_input("Confirm your password...", &self.pass2)
                .on_input(move |x| Message::Auth(AuthMessage::UpdatePass2(x)))
                .secure(true),
            SUB_HEADING_SIZE,
        );

        let back = container(
            button(
                text(match &self.page {
                    Page::Signin => "Sign-Up",
                    Page::Signup => "Sign-In",
                })
                .height(HEADER_SIZE),
            )
            .on_press(Message::Auth(AuthMessage::ChangePage))
            .style(style::button::transparent_back_white_text),
        )
        .style(style::container::back_bordered);

        let next = container(
            style::svg_button::text("forward_arrow.svg", HEADER_SIZE)
                .on_press(Message::Auth(AuthMessage::Submit)),
        )
        .style(style::container::back_bordered);

        center(
            container(
                scrollable::Scrollable::new(
                    column![
                        banner,
                        rule::horizontal(1),
                        text(match &self.page {
                            Page::Signin => "Sign-In",
                            Page::Signup => "Sign-Up",
                        })
                        .font(get_bold_font())
                        .size(HEADER_SIZE)
                        .style(style::text::primary),
                        rule::horizontal(1),
                        heading("Instance Url"),
                        instance,
                        heading("Details"),
                        match &self.page {
                            Page::Signup => column![
                                sub_heading("Username"),
                                name,
                                sub_heading("Email"),
                                email,
                                sub_heading("Password"),
                                pass1,
                                sub_heading("Confirm Password"),
                                pass2,
                            ],
                            Page::Signin => column![
                                sub_heading("Username"),
                                name,
                                sub_heading("Password"),
                                pass1,
                            ],
                        }
                        .spacing(5),
                        rule::horizontal(1),
                        row![back, space::horizontal(), next]
                            .spacing(10)
                            .align_y(Vertical::Center),
                    ]
                    .spacing(10),
                )
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::default(),
                )),
            )
            .max_width(800)
            .padding(Padding::new(20.0))
            .style(style::container::neutral_back),
        )
        .into()
    }
}
