// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! bartos - barto server

// rustc lints
#![cfg_attr(
    all(feature = "unstable", nightly),
    feature(
        multiple_supertrait_upcastable,
        must_not_suspend,
        non_exhaustive_omitted_patterns_lint,
        rustdoc_missing_doc_code_examples,
        strict_provenance_lints,
        supertrait_item_shadowing,
        unqualified_local_imports,
    )
)]
#![cfg_attr(nightly, allow(single_use_lifetimes))]
#![cfg_attr(
    nightly,
    deny(
        absolute_paths_not_starting_with_crate,
        ambiguous_glob_imports,
        ambiguous_glob_reexports,
        ambiguous_negative_literals,
        ambiguous_wide_pointer_comparisons,
        anonymous_parameters,
        array_into_iter,
        asm_sub_register,
        async_fn_in_trait,
        bad_asm_style,
        bare_trait_objects,
        boxed_slice_into_iter,
        break_with_label_and_loop,
        clashing_extern_declarations,
        closure_returning_async_block,
        coherence_leak_check,
        confusable_idents,
        const_evaluatable_unchecked,
        const_item_mutation,
        dangling_pointers_from_temporaries,
        dead_code,
        dependency_on_unit_never_type_fallback,
        deprecated,
        deprecated_in_future,
        deprecated_safe_2024,
        deprecated_where_clause_location,
        deref_into_dyn_supertrait,
        deref_nullptr,
        double_negations,
        drop_bounds,
        dropping_copy_types,
        dropping_references,
        duplicate_macro_attributes,
        dyn_drop,
        edition_2024_expr_fragment_specifier,
        elided_lifetimes_in_paths,
        ellipsis_inclusive_range_patterns,
        explicit_outlives_requirements,
        exported_private_dependencies,
        ffi_unwind_calls,
        forbidden_lint_groups,
        forgetting_copy_types,
        forgetting_references,
        for_loops_over_fallibles,
        function_item_references,
        hidden_glob_reexports,
        if_let_rescope,
        impl_trait_overcaptures,
        impl_trait_redundant_captures,
        improper_ctypes,
        improper_ctypes_definitions,
        inline_no_sanitize,
        internal_features,
        invalid_from_utf8,
        invalid_macro_export_arguments,
        invalid_nan_comparisons,
        invalid_value,
        irrefutable_let_patterns,
        keyword_idents_2018,
        keyword_idents_2024,
        large_assignments,
        late_bound_lifetime_arguments,
        legacy_derive_helpers,
        let_underscore_drop,
        macro_use_extern_crate,
        map_unit_fn,
        meta_variable_misuse,
        mismatched_lifetime_syntaxes,
        missing_abi,
        missing_copy_implementations,
        missing_debug_implementations,
        missing_docs,
        missing_unsafe_on_extern,
        mixed_script_confusables,
        named_arguments_used_positionally,
        never_type_fallback_flowing_into_unsafe,
        no_mangle_generic_items,
        non_ascii_idents,
        non_camel_case_types,
        non_contiguous_range_endpoints,
        non_fmt_panics,
        non_local_definitions,
        non_shorthand_field_patterns,
        non_snake_case,
        non_upper_case_globals,
        noop_method_call,
        opaque_hidden_inferred_bound,
        out_of_scope_macro_calls,
        overlapping_range_endpoints,
        path_statements,
        private_bounds,
        private_interfaces,
        ptr_to_integer_transmute_in_consts,
        redundant_imports,
        redundant_lifetimes,
        redundant_semicolons,
        refining_impl_trait_internal,
        refining_impl_trait_reachable,
        renamed_and_removed_lints,
        repr_transparent_non_zst_fields,
        rust_2021_incompatible_closure_captures,
        rust_2021_incompatible_or_patterns,
        rust_2021_prefixes_incompatible_syntax,
        rust_2021_prelude_collisions,
        rust_2024_guarded_string_incompatible_syntax,
        rust_2024_incompatible_pat,
        rust_2024_prelude_collisions,
        self_constructor_from_outer_item,
        semicolon_in_expressions_from_macros,
        single_use_lifetimes,
        special_module_name,
        stable_features,
        static_mut_refs,
        suspicious_double_ref_op,
        tail_expr_drop_order,
        trivial_bounds,
        trivial_casts,
        trivial_numeric_casts,
        type_alias_bounds,
        tyvar_behind_raw_pointer,
        uncommon_codepoints,
        unconditional_recursion,
        uncovered_param_in_projection,
        unexpected_cfgs,
        unfulfilled_lint_expectations,
        ungated_async_fn_track_caller,
        uninhabited_static,
        unit_bindings,
        unknown_lints,
        unknown_or_malformed_diagnostic_attributes,
        unnameable_test_items,
        unnameable_types,
        unpredictable_function_pointer_comparisons,
        unreachable_code,
        unreachable_patterns,
        unreachable_pub,
        unsafe_attr_outside_unsafe,
        unsafe_code,
        unsafe_op_in_unsafe_fn,
        unstable_name_collisions,
        unstable_syntax_pre_expansion,
        unused_allocation,
        unused_assignments,
        unused_associated_type_bounds,
        unused_attributes,
        unused_braces,
        unused_comparisons,
        unused_crate_dependencies,
        unused_doc_comments,
        unused_extern_crates,
        unused_features,
        unused_import_braces,
        unused_imports,
        unused_labels,
        unused_lifetimes,
        unused_macro_rules,
        unused_macros,
        unused_must_use,
        unused_mut,
        unused_parens,
        unused_qualifications,
        unused_results,
        unused_unsafe,
        unused_variables,
        useless_ptr_null_checks,
        uses_power_alignment,
        variant_size_differences,
        while_true,
    )
)]
// If nightly and unstable, allow `incomplete_features` and `unstable_features`
#![cfg_attr(
    all(feature = "unstable", nightly),
    allow(incomplete_features, unstable_features)
)]
// If nightly and not unstable, deny `incomplete_features` and `unstable_features`
#![cfg_attr(
    all(not(feature = "unstable"), nightly),
    deny(incomplete_features, unstable_features)
)]
// The unstable lints
#![cfg_attr(
    all(feature = "unstable", nightly),
    deny(
        fuzzy_provenance_casts,
        lossy_provenance_casts,
        multiple_supertrait_upcastable,
        must_not_suspend,
        non_exhaustive_omitted_patterns,
        supertrait_item_shadowing_definition,
        supertrait_item_shadowing_usage,
        unqualified_local_imports,
    )
)]
// clippy lints
#![cfg_attr(nightly, deny(clippy::all, clippy::pedantic))]
// rustdoc lints
#![cfg_attr(
    nightly,
    deny(
        rustdoc::bare_urls,
        rustdoc::broken_intra_doc_links,
        rustdoc::invalid_codeblock_attributes,
        rustdoc::invalid_html_tags,
        rustdoc::missing_crate_level_docs,
        rustdoc::private_doc_tests,
        rustdoc::private_intra_doc_links,
    )
)]
#![cfg_attr(
    all(nightly, feature = "unstable"),
    deny(rustdoc::missing_doc_code_examples)
)]
#![cfg_attr(all(docsrs), feature(doc_cfg))]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use std::{path::Path, sync::Arc};

use anyhow::Result;
use bon::Builder;
use getset::WithSetters;
use iced::{
    Element,
    Length::Fill,
    Task, Theme,
    padding::{bottom, top},
    widget::{PickList, button, column, container, horizontal_space, row, text, text_editor},
};
use rfd::AsyncFileDialog;
use sqlx::MySqlPool;
use tokio::fs::read_to_string;

use crate::error::Error;

use iced_aw::{TabBar, TabLabel};
use iced_fonts::REQUIRED_FONT_BYTES;

mod error;

#[derive(Debug, WithSetters)]
struct State {
    screen: Screen,
    content: text_editor::Content,
    theme: Theme,
    #[getset(set_with)]
    main_screen: MainScreen,
    #[getset(set_with)]
    db: Option<MySqlPool>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            screen: Screen::Main,
            content: text_editor::Content::new(),
            theme: Theme::CatppuccinMocha,
            main_screen: MainScreen::default(),
            db: None,
        }
    }
}

#[derive(Builder, Debug, Default)]
struct MainScreen {
    tabs: Vec<(usize, String)>,
    active_tab: Option<(usize, String)>,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
enum Screen {
    #[default]
    Main,
}

#[derive(Clone, Debug)]
enum Message {
    Initialized,
    Edit(text_editor::Action),
    Open,
    FileOpened(Arc<String>),
    ThemeChanged(Theme),
    TabSelected((usize, String)),
    TabClosed((usize, String)),
}

#[tokio::main]
async fn main() -> Result<()> {
    let pool = MySqlPool::connect("mysql://barto:barto@localhost/barto").await?;
    let mut names = get_distinct_names(&pool).await?;
    names.sort();

    Ok(iced::application("A cool application", update, view)
        .theme(theme)
        .font(REQUIRED_FONT_BYTES)
        .run_with(|| {
            let tabs = names
                .into_iter()
                .enumerate()
                .collect::<Vec<(usize, String)>>();
            let active_tab = tabs.first().cloned();
            let main_screen = MainScreen::builder()
                .tabs(tabs)
                .maybe_active_tab(active_tab)
                .build();
            let state = State::default()
                .with_db(Some(pool))
                .with_main_screen(main_screen);
            (state, Task::none())
        })?)
}

fn theme(state: &State) -> Theme {
    state.theme.clone()
}

fn view(state: &State) -> Element<'_, Message> {
    match state.screen {
        Screen::Main => main_screen(state),
    }
}

fn main_screen(state: &State) -> Element<'_, Message> {
    let controls = row![button("Open").on_press(Message::Open)];

    let tab_bar = state
        .main_screen
        .tabs
        .iter()
        .fold(
            TabBar::new(Message::TabSelected),
            |tab_bar, (idx, tab_label)| {
                tab_bar.push(
                    (*idx, tab_label.clone()),
                    TabLabel::Text(tab_label.to_owned()),
                )
            },
        )
        .on_close(Message::TabClosed);
    let tab_bar = if let Some(active_tab) = &state.main_screen.active_tab {
        tab_bar.set_active_tab(active_tab)
    } else {
        tab_bar
    };

    // let tab_bar_row = row![
    //     tab_bar.width(FillPortion(1)),
    //     horizontal_space().width(FillPortion(1))
    // ];
    let position = {
        let (line, column) = state.content.cursor_position();
        let line = line + 1;
        let column = column + 1;
        text(format!("Line: {line}, Column: {column}"))
    };

    let status_bar = row![
        PickList::new(Theme::ALL, Some(&state.theme), Message::ThemeChanged),
        horizontal_space(),
        position
    ]
    .padding(5);

    let tab_content = match state.main_screen.active_tab {
        Some((0, _)) => tab1(state),
        Some((1, _)) => tab2(state),
        Some((2, _)) => tab3(state),
        _ => container(text("No tab selected")).height(Fill).into(),
    };
    container(
        column![
            controls.padding(bottom(10)),
            tab_bar,
            tab_content,
            status_bar.padding(top(10))
        ]
        .height(Fill),
    )
    .padding(10)
    .height(Fill)
    .into()
}

fn tab1(state: &State) -> Element<'_, Message> {
    text_editor(&state.content)
        .placeholder("This is your first editor")
        .on_action(Message::Edit)
        .height(Fill)
        .into()
}

fn tab2(_state: &State) -> Element<'_, Message> {
    container(text("Tab 2")).height(Fill).into()
}

fn tab3(_state: &State) -> Element<'_, Message> {
    container(text("Tab 3")).height(Fill).into()
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Edit(action) => {
            state.content.perform(action);
            Task::none()
        }
        Message::FileOpened(content) => {
            state.content = text_editor::Content::with_text(&content);
            Task::none()
        }
        Message::Open => {
            Task::perform(pick_file(), |res| match res {
                Ok(content) => Message::FileOpened(content),
                Err(_) => Message::Initialized, // Handle error appropriately
            })
        }
        Message::Initialized => {
            // Handle open action
            Task::none()
        }
        Message::TabSelected(active_tab) => {
            state.main_screen.active_tab = Some(active_tab);
            Task::none()
        }
        Message::ThemeChanged(theme) => {
            state.theme = theme;
            Task::none()
        }
        Message::TabClosed(_screen) => Task::none(),
    }
}

async fn load_file(path: impl AsRef<Path>) -> Result<Arc<String>> {
    let content = read_to_string(path).await?;
    Ok(Arc::new(content))
}

async fn pick_file() -> Result<Arc<String>> {
    let file = AsyncFileDialog::new()
        .set_title("Open a file")
        .pick_file()
        .await
        .ok_or(Error::FilePickerClosed)?;

    load_file(file.path()).await
}

async fn get_distinct_names(pool: &MySqlPool) -> Result<Vec<String>> {
    let names = sqlx::query!("select distinct output.bartoc_name from output")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|record| record.bartoc_name)
        .collect::<Vec<String>>();
    Ok(names)
}
