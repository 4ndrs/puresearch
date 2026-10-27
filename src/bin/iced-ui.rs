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
        let images = std::fs::read_dir("experiments/images")
            .unwrap()
            .map(|dir| {
                widget::image(dir.unwrap().path())
                    .width(200)
                    .height(200)
                    .content_fit(iced::ContentFit::Cover)
                    .border_radius(8)
                    .into()
            })
            .collect::<Vec<iced::Element<_>>>();

        widget::container(widget::column![
            widget::text_input("Search...", &self.search)
                .size(24)
                .font(iced::Font {
                    weight: iced::font::Weight::Semibold,
                    ..Default::default()
                })
                .style(|theme: &iced::Theme, status| {
                    let mut theme = widget::text_input::default(theme, status);

                    theme.background = iced::Background::Color(iced::Color::TRANSPARENT);
                    theme.border.color = iced::Color::TRANSPARENT;

                    theme
                })
                .on_input(Message::Update)
                .padding(15),
            widget::container(widget::rule::horizontal(1)).padding(iced::Padding {
                right: 15.0,
                left: 15.0,
                ..Default::default()
            }),
            widget::scrollable(
                widget::row(images)
                    .padding(iced::Padding {
                        left: 15.0,
                        right: 15.0,
                        top: 30.0,
                        bottom: 10.0
                    })
                    .spacing(5)
                    .wrap()
            )
            .direction(widget::scrollable::Direction::Vertical(
                widget::scrollable::Scrollbar::new()
                    .width(8)
                    .margin(2)
                    .scroller_width(5)
            ))
            .style(|theme: &iced::Theme, status| {
                let mut style = widget::scrollable::default(theme, status);

                style.vertical_rail.background = None;
                style.vertical_rail.scroller.border.radius = 50.into();

                style
            })
        ])
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .style(|theme: &iced::Theme| {
            let mut style = widget::container::Style::default();

            style.border.radius = 24.0.into();
            style.background = Some(iced::Background::Color(theme.palette().background));

            style
        })
        .padding(iced::Padding {
            bottom: 20.0,
            ..Default::default()
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
