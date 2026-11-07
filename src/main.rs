use std::rc::Rc;

use anyhow::{Context, Result, anyhow};

use crate::{
    db::JiraDatabase,
    io_utils::{get_user_input, wait_for_key_press},
    models::Action,
    navigator::Navigator,
};

mod db;
mod io_utils;
mod models;
mod navigator;
mod ui;

fn main() -> Result<()> {
    let db = Rc::new(JiraDatabase::new("./data/db.json".to_string()));
    let mut nav = Navigator::new(Rc::clone(&db));

    loop {
        clearscreen::clear().unwrap();

        // 1. get current page from navigator. If there is no current page exit the loop.
        let page = match nav.get_current_page() {
            Some(page) => page,
            None => break,
        };
        // 2. render page
        page.draw_page().with_context(|| {
            anyhow!("Could not display page to user of type {}", {
                let page_any = page.as_any();
                if page_any.is::<ui::HomePage>() {
                    "HomePage"
                } else if page_any.is::<ui::EpicDetail>() {
                    "EpicDetail"
                } else if page_any.is::<ui::StoryDetail>() {
                    "StoryDetail"
                } else {
                    "Unknown"
                }
            })
        })?;
        // 3. get user input
        let response = get_user_input();
        // 4. pass input to page's input handler
        match page
            .handle_input(&response)
            .with_context(|| anyhow!("Failed to process input \"{}\"", response))
        {
            Ok(Some(action)) => {
                // 5. if the page's input handler returns an action let the navigator process the action
                nav.handle_action(action).with_context(|| {
                    anyhow!(
                        "Failed to execute action: {}",
                        match action {
                            Action::NavigateToEpicDetail { epic_id: _ } => "View Epic",
                            Action::NavigateToStoryDetail {
                                epic_id: _,
                                story_id: _,
                            } => "View Story",
                            Action::NavigateToPreviousPage => "Go Back",
                            Action::CreateEpic => "Create Epic",
                            Action::UpdateEpicStatus { epic_id: _ } => "Update Epic Status",
                            Action::DeleteEpic { epic_id: _ } => "Delete Epic",
                            Action::CreateStory { epic_id: _ } => "Create Story",
                            Action::UpdateStoryStatus { story_id: _ } => "Update Story Status",
                            Action::DeleteStory {
                                epic_id: _,
                                story_id: _,
                            } => "Delete Story",
                            Action::Exit => "Exit",
                        }
                    )
                })?;
            }
            Ok(None) => {}
            Err(error) => {
                println!(
                    "Error when handling input:{}\nPress enter to continue.",
                    error
                );
                wait_for_key_press();
                continue;
            }
        };
    }
    Ok(())
}
