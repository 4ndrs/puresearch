use iced::widget;

#[derive(Default)]
struct State {
    search: String,
}

#[derive(Clone)]
enum Message {
    Update(String),
}

impl State {
    fn update(&mut self, message: Message) {
        match message {
            Message::Update(text) => self.search = text,
        }
    }
    fn view(&self) -> iced::Element<'_, Message> {
        widget::container(
            widget::column![
                widget::text_input("Search...", &self.search)
                    .size(24)
                    .font(iced::Font {
                        weight: iced::font::Weight::Semibold,
                        ..Default::default()
                    })
                    .style(|theme: &iced::Theme, status| {
                        let mut theme = widget::text_input::default(theme, status);

                        theme.border.color = iced::Color::TRANSPARENT;

                        theme
                    })
                    .on_input(Message::Update),
                widget::rule::horizontal(1)
            ]
            .spacing(6),
        )
        .padding(15)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .style(|theme: &iced::Theme| {
            let mut style = widget::container::Style::default();

            style.border.radius = 24.0.into();
            style.background = Some(iced::Background::Color(theme.palette().background));

            style
        })
        .into()
    }
}

fn main() -> iced::Result {
    iced::application(State::default, State::update, State::view)
        .title("PureSearch")
        .decorations(false)
        .resizable(false)
        .window_size(iced::Size {
            width: 850.0,
            height: 500.0,
        })
        .transparent(true)
        .style(|_, theme| iced::theme::Style {
            text_color: theme.palette().text,
            background_color: iced::Color::TRANSPARENT,
        })
        .run()
}
