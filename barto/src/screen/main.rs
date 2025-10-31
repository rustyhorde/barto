// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{cmp::Ordering, collections::BTreeMap, sync::LazyLock};

use anyhow::Result;
use bon::Builder;
use iced::{
    Element,
    Length::Fill,
    Task,
    padding::top,
    widget::{Column, PickList, Text, button, column, container, row, scrollable, text},
};
use iced_aw::{Card, TabBar, TabLabel};
use libbarto::clean_output_string;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::{message::Message, state::Screen};

static SCROLLABLE: LazyLock<scrollable::Id> = LazyLock::new(scrollable::Id::unique);

#[derive(Builder, Clone, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) struct TabKey {
    id: Uuid,
    label: String,
}

impl TabKey {
    fn none() -> Self {
        TabKey::builder()
            .id(Uuid::nil())
            .label(String::new())
            .build()
    }
}

impl Ord for TabKey {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.label.cmp(&other.label) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.id.cmp(&other.id),
            Ordering::Greater => Ordering::Greater,
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
    body: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
pub(crate) enum MainScreenMessage {
    Load,
    NamesLoaded(Option<Vec<String>>),
    TabSelected(TabKey),
    TabContent(Option<Vec<String>>),
    CommandSelected(String),
    CommandOutputLoaded(Option<BTreeMap<u64, String>>),
    FetchNext,
}

#[derive(Builder, Clone, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) struct MainScreen {
    tabs: BTreeMap<TabKey, Tab>,
    active_tab_opt: Option<(TabKey, Tab)>,
    max_id: Option<u64>,
}

impl MainScreen {
    pub(crate) fn render(&self) -> Element<'_, MainScreenMessage> {
        let tab_bar = self.tabs.iter().fold(
            TabBar::new(MainScreenMessage::TabSelected),
            |tab_bar, tab| tab_bar.push(tab.0.clone(), TabLabel::Text(tab.0.label.clone())),
        );
        let tab_bar = if let Some((key, _active_tab)) = &self.active_tab_opt {
            tab_bar.set_active_tab(key)
        } else {
            tab_bar
        };

        let tab_content = if let Some((_key, active_tab)) = &self.active_tab_opt {
            if let Some(cmd_names) = &active_tab.cmd_names {
                let cmds = cmd_names.iter().map(String::as_str).collect::<Vec<&str>>();
                let cmd_name_picker = row![
                    Text::new("Choose a command: ").size(20),
                    PickList::new(cmds, active_tab.selected_cmd.as_deref(), |name| {
                        MainScreenMessage::CommandSelected(name.to_string())
                    })
                ]
                .spacing(5);

                if let Some(body) = active_tab.body.as_ref() {
                    let body_text = body
                        .iter()
                        .map(|line| Text::new(line).wrapping(text::Wrapping::None))
                        .collect::<Vec<Text<'_>>>();
                    let len = body_text.len();
                    let mut output_column =
                        Column::from_vec(body_text.into_iter().map(Into::into).collect())
                            .width(Fill);

                    if len >= 500 {
                        output_column = output_column.push(
                            button(Text::new("Load more..."))
                                .on_press(MainScreenMessage::FetchNext),
                        );
                    }

                    container(column![Card::new(
                        cmd_name_picker,
                        scrollable(output_column).id((*SCROLLABLE).clone())
                    )])
                    .width(Fill)
                    .height(Fill)
                    .padding(top(10).bottom(10))
                } else {
                    container(column![Card::new(
                        cmd_name_picker,
                        Text::new("Loading output...")
                    )])
                    .width(Fill)
                    .height(Fill)
                    .padding(top(10).bottom(10))
                }
            } else {
                container(text("Loading...")).height(Fill)
            }
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
                let tabs = names_opt.as_ref().map_or_else(BTreeMap::new, |names| {
                    names
                        .iter()
                        .map(|name| {
                            let key = TabKey::builder()
                                .id(Uuid::new_v4())
                                .label(name.clone())
                                .build();
                            let value = Tab::builder().build();
                            (key, value)
                        })
                        .collect::<BTreeMap<TabKey, Tab>>()
                });
                let active_tab_opt = tabs
                    .first_key_value()
                    .map(|(key, tab)| (key.clone(), tab.clone()));
                let id = active_tab_opt.clone().map_or(TabKey::none(), |(k, _v)| k);
                let message = Message::from(MainScreenMessage::TabSelected(id));
                let main_screen = MainScreen::builder()
                    .tabs(tabs)
                    .maybe_active_tab_opt(active_tab_opt)
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
            MainScreenMessage::TabSelected(tab_key) => {
                if let Some(active_tab) = self.tabs.get(&tab_key) {
                    let name = tab_key.label.clone();
                    let pool = pool_opt.clone();
                    self.active_tab_opt = Some((tab_key, active_tab.clone()));

                    Task::perform(
                        get_distinct_cmd_names_opt(pool, name),
                        MainScreenMessage::TabContent,
                    )
                } else {
                    Task::none()
                }
            }
            MainScreenMessage::TabContent(content_opt) => {
                if let Some(cmd_names) = content_opt
                    && let Some((_key, active_tab)) = &mut self.active_tab_opt
                {
                    active_tab.cmd_names = Some(cmd_names.clone());
                }
                Task::none()
            }
            MainScreenMessage::CommandSelected(name) => {
                if let Some((key, active_tab)) = &mut self.active_tab_opt {
                    let bartoc_name = key.label.clone();
                    active_tab.selected_cmd = Some(name.clone());
                    Task::perform(
                        get_cmd_output_opt(pool_opt, self.max_id, bartoc_name, name),
                        MainScreenMessage::CommandOutputLoaded,
                    )
                } else {
                    Task::none()
                }
            }
            MainScreenMessage::CommandOutputLoaded(output_opt) => {
                let max_id = output_opt
                    .as_ref()
                    .and_then(|output_map| output_map.keys().max().copied());
                if let Some(output_map) = &output_opt
                    && output_map.len() == 500
                {
                    self.max_id = max_id;
                }

                let stripped_output_opt = output_opt.as_ref().map(|output_map| {
                    output_map
                        .values()
                        .map(|line| clean_output_string(line).0.clone())
                        .collect::<Vec<String>>()
                });
                if let Some((_key, active_tab)) = &mut self.active_tab_opt {
                    active_tab.body.clone_from(&stripped_output_opt);
                }
                Task::none()
            }
            MainScreenMessage::FetchNext => {
                if let Some((key, active_tab)) = &mut self.active_tab_opt {
                    active_tab.body = None;
                    let bartoc_name = key.label.clone();
                    if let Some(cmd_name) = &active_tab.selected_cmd {
                        let name = cmd_name.clone();
                        Task::perform(
                            get_cmd_output_opt(pool_opt, self.max_id, bartoc_name, name),
                            MainScreenMessage::CommandOutputLoaded,
                        )
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
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
    name: String,
) -> Option<Vec<String>> {
    get_distinct_cmd_names(pool_opt, name).await.ok()
}

async fn get_distinct_cmd_names(pool_opt: Option<MySqlPool>, name: String) -> Result<Vec<String>> {
    Ok(if let Some(pool) = &pool_opt {
        sqlx::query!(
            "select distinct output.cmd_name from output where output.bartoc_name = ?",
            name
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|record| record.cmd_name)
        .collect::<Vec<String>>()
    } else {
        vec![]
    })
}

async fn get_cmd_output_opt(
    pool_opt: Option<MySqlPool>,
    last_id_opt: Option<u64>,
    name: String,
    cmd_name: String,
) -> Option<BTreeMap<u64, String>> {
    get_cmd_output(pool_opt, last_id_opt, name, cmd_name)
        .await
        .ok()
}

async fn get_cmd_output(
    pool_opt: Option<MySqlPool>,
    last_id_opt: Option<u64>,
    name: String,
    cmd_name: String,
) -> Result<BTreeMap<u64, String>> {
    if let Some(pool) = &pool_opt {
        Ok(if let Some(last_id) = last_id_opt {
            sqlx::query!(
                "select output.id, output.data from output where output.bartoc_name = ? and output.cmd_name = ? and output.id > ?
order by output.timestamp limit 500",
                name,
                cmd_name,
                last_id
            )
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|record| (record.id, record.data))
            .collect::<BTreeMap<u64, String>>()
        } else {
            sqlx::query!(
                "select output.id, output.data from output where output.bartoc_name = ? and output.cmd_name = ?
order by output.timestamp limit 500",
                name,
                cmd_name
            )
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|record| (record.id, record.data))
            .collect::<BTreeMap<u64, String>>()
        })
    } else {
        Ok(BTreeMap::new())
    }
}
