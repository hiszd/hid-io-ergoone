use iced::theme;
use iced::widget::{button, column, container, row, text};
use iced::{keyboard, Subscription};
use tokio::sync::RwLock;

use crate::util::MSG;

pub enum Conn {
  Loading,
  Loaded(ConnState),
}

pub struct ConnState {
  receiver: RwLock<Option<tokio::sync::broadcast::Receiver<MSG>>>,
}

pub struct HidIoGui {
  count: u32,
  layer: u16,
  volume: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
  Increment,
  Decrement,
  Layer(u16),
  Volume(u16),
  NAN,
}

impl iced::Application for HidIoGui {
  type Theme = iced::Theme;
  type Executor = iced::executor::Default;
  type Message = Message;
  type Flags = ();

  fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
    (
      HidIoGui {
        count: 0,
        layer: 0,
        volume: 0,
      },
      iced::Command::none(),
    )
  }

  fn title(&self) -> String { String::from("HID-IO GUI") }

  fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
    match message {
      Message::Increment => {
        if self.count != u32::MAX {
          self.count += 1;
        }
        iced::Command::none()
      }
      Message::Decrement => {
        if self.count != u32::MIN {
          self.count -= 1;
        }
        iced::Command::none()
      }
      Message::Layer(l) => {
        println!("Layer: {}", l);
        s.layer = l;
        iced::Command::none()
      }
      Message::Volume(v) => {
        println!("Volume: {}", v);
        s.volume = v;
        iced::Command::none()
      }
      Message::NAN => iced::Command::none(),
    }
  }

  fn subscription(&self) -> Subscription<Message> {
    let b = keyboard::on_key_press(|key, _modifiers| match key.as_ref() {
      keyboard::Key::Character("k") => Some(Message::Increment),
      keyboard::Key::Character("j") => Some(Message::Decrement),
      _ => None,
    });
    let ms = iced::subscription::unfold("gui", 0, |state| async move {
      let mut queue = unsafe { crate::HIDIO_QUEUE.lock().unwrap() };
      match queue.pop() {
        Some(m) => match m {
          MSG::Layer(l) => (Message::Layer(l), 0),
          MSG::Volume(v) => (Message::Volume(v), 0),
        },
        None => (Message::NAN, 0),
      }
    });
    Subscription::batch(vec![b, ms])
  }

  fn theme(&self) -> Self::Theme { Self::Theme::Dark }

  fn view(&self) -> iced::Element<'_, Self::Message> {
    let counter = row![
      button("+").on_press(Message::Increment).padding(20).style(theme::Button::Text),
      text(self.count.to_string()).horizontal_alignment(iced::alignment::Horizontal::Center),
      button("-").on_press(Message::Decrement).padding(20).style(theme::Button::Destructive),
    ]
    .spacing(10)
    .align_items(iced::Alignment::Center);

    let col = column![
      counter,
      text(self.layer.to_string()).horizontal_alignment(iced::alignment::Horizontal::Center),
      text(self.volume.to_string()).horizontal_alignment(iced::alignment::Horizontal::Center),
    ];

    let cont =
      container(col).width(iced::Length::Fill).height(iced::Length::Fill).center_x().center_y();

    cont.into()
  }
}
