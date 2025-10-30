// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{cmp::Ordering, collections::BTreeMap};

use anyhow::Result;
use bon::Builder;
use iced::{
    Element,
    Length::Fill,
    Task,
    padding::top,
    widget::{PickList, Text, column, container, row, text},
};
use iced_aw::{Card, TabBar, TabLabel};
use sqlx::MySqlPool;
use tracing::info;
use uuid::Uuid;

use crate::{message::Message, state::Screen};

#[derive(Builder, Clone, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) struct TabKey {
    id: Uuid,
    label: String,
}

impl Ord for TabKey {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.label.cmp(&other.label) {
            Ordering::Equal => self.id.cmp(&other.id),
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
        }
    }
}

impl PartialOrd for TabKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Builder, Clone, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) struct Tab {
    cmd_names: Option<Vec<String>>,
    selected_cmd: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) enum MainScreenMessage {
    Load,
    NamesLoaded(Option<Vec<String>>),
    TabSelected(Option<TabKey>),
    TabContent(Option<(TabKey, Vec<String>)>),
    CommandSelected((TabKey, String)),
}

#[derive(Builder, Clone, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) struct MainScreen {
    tabs: BTreeMap<TabKey, Tab>,
    active_tab: Option<TabKey>,
}

impl MainScreen {
    pub(crate) fn render(&self) -> Element<'_, MainScreenMessage> {
        let tab_bar = self.tabs.iter().fold(
            TabBar::new(MainScreenMessage::TabSelected),
            |tab_bar, (tab_key, _tab)| {
                tab_bar.push(Some(tab_key.clone()), TabLabel::Text(tab_key.label.clone()))
            },
        );
        let tab_bar = if let Some(active_tab) = &self.active_tab {
            tab_bar.set_active_tab(&Some(active_tab.clone()))
        } else {
            tab_bar
        };

        let tab_content = if let Some(active_tab) = &self.active_tab {
            let key_c = active_tab.clone();
            self.tabs.get(active_tab).map_or_else(
                || container(text("Tab not found")).height(Fill),
                move |tab| {
                    if let Some(cmd_names) = &tab.cmd_names {
                        let cmds = cmd_names.iter().map(String::as_str).collect::<Vec<&str>>();
                        let cmd_name_picker = row![
                            Text::new("Choose a command: ").size(20),
                            PickList::new(cmds, tab.selected_cmd.as_deref(), move |name| {
                                MainScreenMessage::CommandSelected((key_c.clone(), name.to_owned()))
                            })
                        ]
                        .spacing(5);
                        container(column![Card::new(cmd_name_picker, Text::new("Body"))])
                            .height(Fill)
                            .padding(top(10).bottom(10))
                    } else {
                        container(text("Loading...")).height(Fill)
                    }
                },
            )
        } else {
            container(text("No tab selected")).height(Fill)
        };

        column![tab_bar, tab_content].height(Fill).into()
    }

    pub(crate) fn handle_init_message(
        pool_opt: Option<MySqlPool>,
        message: MainScreenMessage,
    ) -> Task<Message> {
        match message {
            MainScreenMessage::Load => Task::perform(
                get_distinct_names_opt(pool_opt),
                MainScreenMessage::NamesLoaded,
            )
            .map(Message::from),
            MainScreenMessage::NamesLoaded(names_opt) => {
                info!("Names Loaded");
                let tabs = names_opt.as_ref().map_or_else(BTreeMap::new, |names| {
                    names
                        .iter()
                        .map(|name| {
                            let tab_key = TabKey::builder()
                                .id(Uuid::new_v4())
                                .label(name.clone())
                                .build();
                            let tab = Tab::builder().build();
                            (tab_key, tab)
                        })
                        .collect::<BTreeMap<TabKey, Tab>>()
                });
                let active_tab_opt = tabs.first_key_value().map(|(key, _)| key.clone());
                let message = Message::from(MainScreenMessage::TabSelected(active_tab_opt.clone()));
                let main_screen = MainScreen::builder()
                    .tabs(tabs)
                    .maybe_active_tab(active_tab_opt)
                    .build();
                let load_screen = Message::LoadScreen(Screen::Main(main_screen.clone()));
                Task::done(load_screen).chain(Task::done(message))
            }
            _ => Task::none(),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn handle_message(
        &mut self,
        pool_opt: Option<MySqlPool>,
        message: MainScreenMessage,
    ) -> Task<Message> {
        match message {
            MainScreenMessage::TabSelected(active_tab_opt) => {
                if let Some(active_tab) = &active_tab_opt {
                    self.active_tab = Some(active_tab.clone());
                    let name = active_tab.label.clone();
                    let tab_key_c = active_tab.clone();
                    let pool = pool_opt.clone();

                    Task::perform(
                        get_distinct_cmd_names_opt(pool, tab_key_c, name),
                        MainScreenMessage::TabContent,
                    )
                } else {
                    Task::none()
                }
            }
            MainScreenMessage::TabContent(content_opt) => {
                if let Some((tab_key, cmd_names)) = content_opt {
                    let _old = self.tabs.get_mut(&tab_key).map(|tab| {
                        tab.cmd_names = Some(cmd_names.clone());
                    });
                }
                Task::none()
            }
            MainScreenMessage::CommandSelected((tab_key, name)) => {
                let _old = self.tabs.get_mut(&tab_key).map(|tab| {
                    tab.selected_cmd = Some(name.clone());
                });
                Task::none()
            }
            _ => Task::none(),
        }
        .map(Message::from)
    }
}

async fn get_distinct_names_opt(pool_opt: Option<MySqlPool>) -> Option<Vec<String>> {
    get_distinct_names(pool_opt).await.ok()
}

async fn get_distinct_names(pool_opt: Option<MySqlPool>) -> Result<Vec<String>> {
    if let Some(pool) = &pool_opt {
        let mut names = sqlx::query!("select distinct output.bartoc_name from output")
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|record| record.bartoc_name)
            .collect::<Vec<String>>();
        names.sort();
        Ok(names)
    } else {
        Ok(vec![])
    }
}

async fn get_distinct_cmd_names_opt(
    pool_opt: Option<MySqlPool>,
    tab_key: TabKey,
    name: String,
) -> Option<(TabKey, Vec<String>)> {
    get_distinct_cmd_names(pool_opt, tab_key, name).await.ok()
}

async fn get_distinct_cmd_names(
    pool_opt: Option<MySqlPool>,
    tab_key: TabKey,
    name: String,
) -> Result<(TabKey, Vec<String>)> {
    Ok(if let Some(pool) = &pool_opt {
        let names = sqlx::query!(
            "select distinct output.cmd_name from output where output.bartoc_name = ?",
            name
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|record| record.cmd_name)
        .collect::<Vec<String>>();
        (tab_key, names)
    } else {
        (tab_key, vec![])
    })
}
