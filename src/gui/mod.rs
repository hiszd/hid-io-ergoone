use std::borrow::BorrowMut;
use std::sync::RwLock;

use iced::futures::{SinkExt, StreamExt};
use iced::theme;
use iced::widget::{button, column, container, row, text};
use iced::{keyboard, Subscription};

use crate::util::MSG;

#[derive(Debug)]
pub enum Conn {
  Loading,
  Loaded,
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
        self.layer = l;
        iced::Command::none()
      }
      Message::Volume(v) => {
        println!("Volume: {}", v);
        self.volume = v;
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
    let ms = iced::subscription::channel("ms", 10, |mut output| async move {
      let mut state = Conn::Loading;
      let mut rec: RwLock<Option<iced_futures::futures::channel::mpsc::Receiver<MSG>>> =
        RwLock::new(None);

      loop {
        match state {
          Conn::Loading => {
            // Create channel
            let (sender, mut receiver) = iced_futures::futures::channel::mpsc::channel(10);

            // Send the sender back to the application
            output.send(Message::NAN).await.unwrap();

            // We are ready to receive messages
            rec = RwLock::new(Some(receiver));
            state = Conn::Loaded;
          }
          Conn::Loaded => {
            let mut receiver = rec.get_mut().unwrap().take().unwrap();
            // Read next input sent from `Application`
            let input = receiver.select_next_some().await;

            match input {
              MSG::Layer(l) => {
                output.send(Message::Layer(l)).await.unwrap();
              }
              MSG::Volume(v) => {
                output.send(Message::Volume(v)).await.unwrap();
              }
            }
          }
        }
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
