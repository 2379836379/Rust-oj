use tauri::{Emitter, Manager};

#[tauri::command]
fn get_app_state() -> Result<oj_client::config::AppState, String> {
    oj_client::config::get_app_state()
}

#[tauri::command]
fn set_app_state(state: oj_client::config::AppStateInput) -> Result<(), String> {
    oj_client::config::set_app_state(state)
}

#[tauri::command]
fn get_openai_config() -> Result<oj_client::config::OpenAiConfig, String> {
    oj_client::config::get_openai_config()
}

#[tauri::command]
fn set_openai_config(config: oj_client::config::OpenAiConfigInput) -> Result<(), String> {
    oj_client::config::set_openai_config(config)
}
#[tauri::command]
fn oj_aiconfig_get_text() -> Result<oj_client::config::OpenAiConfigText, String> {
    oj_client::config::get_openai_config_text()
}

#[tauri::command]
fn oj_aiconfig_save_text(content: String) -> Result<String, String> {
    oj_client::config::set_openai_config_text(content)
}

#[tauri::command]
async fn oj_ai_chat(
    messages: Vec<oj_client::ai::ChatMessage>,
) -> Result<String, String> {
    oj_client::ai::ai_chat(messages).await
}

#[tauri::command]
fn oj_favorite_list_folders() -> Result<Vec<oj_client::favorite::FavoriteFolderInfo>, String> {
    oj_client::favorite::FavoriteRepository::list_folders()
}

#[tauri::command]
fn oj_favorite_create_folder(name: String) -> Result<i64, String> {
    oj_client::favorite::FavoriteRepository::create_folder(name)
}

#[tauri::command]
fn oj_favorite_list_folder_items(
    folder_id: i64,
) -> Result<Vec<oj_client::favorite::FavoriteProblemRow>, String> {
    oj_client::favorite::FavoriteRepository::list_folder_items(folder_id)
}

#[tauri::command]
fn oj_favorite_save_to_folder(
    folder_id: i64,
    problem: oj_client::parser::ProblemPageInfo,
) -> Result<(), String> {
    oj_client::favorite::FavoriteRepository::save_to_folder(folder_id, problem)
}

#[tauri::command]
fn oj_favorite_load_problem(
    problem_url: String,
) -> Result<Option<oj_client::parser::ProblemPageInfo>, String> {
    oj_client::favorite::FavoriteRepository::load_problem(problem_url)
}
#[tauri::command]
fn oj_favorite_delete_folder(folder_id: i64) -> Result<(), String> {
    oj_client::favorite::FavoriteRepository::delete_folder(folder_id)
}

#[tauri::command]
fn oj_favorite_remove_item(folder_id: i64, problem_url: String) -> Result<(), String> {
    oj_client::favorite::FavoriteRepository::remove_item(folder_id, problem_url)
}

#[tauri::command]
fn oj_storage_get_sizes() -> Result<oj_client::storage::StorageSizes, String> {
    oj_client::storage::get_storage_sizes()
}

#[tauri::command]
fn oj_storage_clear_cache() -> Result<oj_client::storage::StorageSizes, String> {
    oj_client::storage::clear_all_caches()
}
#[tauri::command]
fn oj_requires_email_verification(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
    email: String,
) -> bool {
    ctx.requires_email_verification(&email)
}

#[tauri::command]
fn oj_login_cache_last(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
) -> Result<Option<oj_client::storage::LoginRecord>, String> {
    ctx.login_cache_last()
}

#[tauri::command]
fn oj_login_cache_lookup(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
    email: String,
) -> Result<Option<oj_client::storage::LoginRecord>, String> {
    ctx.login_cache_lookup(email)
}

#[tauri::command]
async fn oj_email_send_code(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
    email: String,
) -> Result<String, String> {
    ctx.email_send_code(email).await
}

#[tauri::command]
async fn oj_email_verify_code(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
    email: String,
    code: String,
) -> Result<(), String> {
    ctx.email_verify_code(email, code).await
}

#[tauri::command]
async fn oj_login(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
    email: String,
    password: String,
) -> Result<oj_client::state::OpenJudgeLoginResult, String> {
    ctx.login(email, password).await
}

#[tauri::command]
fn oj_logout(ctx: tauri::State<'_, oj_client::state::AppCtx>) -> Result<(), String> {
    ctx.logout()
}

#[tauri::command]
async fn oj_get_joined_classes(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
) -> Result<Vec<oj_client::parser::JoinedClassInfo>, String> {
    ctx.get_joined_classes().await
}

#[tauri::command]
async fn oj_debug_get_joined_classes_html(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
) -> Result<oj_client::state::DebugJoinedClassesHtml, String> {
    ctx.debug_get_joined_classes_html().await
}

#[tauri::command]
async fn oj_due_soon_reminders(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
    classes: Vec<oj_client::parser::JoinedClassInfo>,
) -> Result<Vec<oj_client::state::DeadlineReminder>, String> {
    ctx.due_soon_reminders(classes).await
}

#[tauri::command]
fn oj_alarm_process_reminders(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
    reminders: Vec<oj_client::state::DeadlineReminder>,
) -> Result<Vec<oj_client::state::AlarmTrigger>, String> {
    ctx.alarm_process_reminders(reminders)
}
#[tauri::command]
async fn oj_open_class(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
    class_page_url: String,
) -> Result<oj_client::state::OpenClassResult, String> {
    ctx.open_class(class_page_url).await
}

#[tauri::command]
async fn oj_open_contest(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
    contest_page_url: String,
) -> Result<oj_client::parser::ContestPageInfo, String> {
    ctx.open_contest(contest_page_url).await
}

#[tauri::command]
async fn oj_open_problem(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
    problem_url: String,
) -> Result<oj_client::parser::ProblemPageInfo, String> {
    ctx.open_problem(problem_url).await
}

#[tauri::command]
async fn oj_open_submit(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
    submit_page_url: String,
) -> Result<oj_client::parser::SubmitPageInfo, String> {
    ctx.open_submit(submit_page_url).await
}

#[tauri::command]
async fn oj_submit_solution(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
    submit_page: oj_client::parser::SubmitPageInfo,
    language: String,
    source_text: String,
) -> Result<oj_client::state::SubmitResponse, String> {
    ctx.submit_solution(submit_page, language, source_text).await
}

#[tauri::command]
async fn oj_open_result(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
    result_page_url: String,
) -> Result<oj_client::parser::ResultPageInfo, String> {
    ctx.open_result(result_page_url).await
}

#[tauri::command]
fn oj_result_is_waiting(result: oj_client::parser::ResultPageInfo) -> bool {
    oj_client::state::AppCtx::result_is_waiting(&result)
}

#[tauri::command]
async fn oj_judge_source(
    ctx: tauri::State<'_, oj_client::state::AppCtx>,
    language: String,
    file_name: String,
    source_code: String,
    stdin_text: String,
) -> Result<oj_client::state::JudgeResponse, String> {
    ctx.judge_source(language, file_name, source_code, stdin_text)
        .await
}

fn main() {
    tauri::Builder::default()
                .manage(oj_client::state::AppCtx::default())
        .setup(|app| {
            let handle = app.handle();
            let menu = tauri::menu::Menu::new(handle)?;
            let restore = tauri::menu::MenuItem::with_id(handle, "tray_restore", "Restore", true, None::<&str>)?;
            let exit = tauri::menu::MenuItem::with_id(handle, "tray_exit", "Exit", true, None::<&str>)?;
            menu.append_items(&[&restore, &exit])?;

            let mut builder = tauri::tray::TrayIconBuilder::new().menu(&menu).tooltip("oj-client");
            if let Some(icon) = app.default_window_icon().cloned() {
                builder = builder.icon(icon);
            }

            builder
                .on_menu_event(|app, event| {
                    let id = event.id().as_ref();
                    if id == "tray_restore" {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    } else if id == "tray_exit" {
                        app.exit(0);
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(handle)?;

            // first-run init (Qt installer behavior)
            let st = oj_client::config::get_app_state()?;
            let source_exists = std::path::Path::new(&st.source_path).is_file();
            if !source_exists {
                let ring = handle
                    .path()
                    .resolve("alarm.mp3", tauri::path::BaseDirectory::Resource)
                    .map_err(|e| format!("resolve alarm.mp3: {e}"))?;

                let _ = oj_client::config::set_app_state(oj_client::config::AppStateInput {
                    ring_path: Some(ring.display().to_string()),
                    alarm_enabled: false,
                });
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();

                let ctx = window.state::<oj_client::state::AppCtx>();
                if ctx.tray_take_first_close_notification() {
                    let _ = window.emit("oj_tray_first_close", ());
                }
            }
        })        .invoke_handler(tauri::generate_handler![
            get_app_state,
            set_app_state,
            get_openai_config,
            set_openai_config,
            oj_aiconfig_get_text,
            oj_aiconfig_save_text,
            oj_ai_chat,
            oj_favorite_list_folders,
            oj_favorite_create_folder,
            oj_favorite_list_folder_items,
            oj_favorite_save_to_folder,
            oj_favorite_load_problem,
            oj_favorite_delete_folder,
            oj_favorite_remove_item,
            oj_storage_get_sizes,
            oj_storage_clear_cache,
            oj_requires_email_verification,
            oj_login_cache_last,
            oj_login_cache_lookup,
            oj_email_send_code,
            oj_email_verify_code,
            oj_login,
            oj_logout,
            oj_get_joined_classes,
            oj_debug_get_joined_classes_html,
            oj_due_soon_reminders,
            oj_alarm_process_reminders,
            oj_open_class,
            oj_open_contest,
            oj_open_problem,
            oj_open_submit,
            oj_submit_solution,
            oj_open_result,
            oj_result_is_waiting,
            oj_judge_source
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
