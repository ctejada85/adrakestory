//! Dialog windows for file operations and confirmations.

mod events;
mod file_operations;
mod rendering;
mod window_handling;

pub use events::{AppExitEvent, FileDialogReceiver, FileSelectedEvent, MapDataChangedEvent};
pub use file_operations::{check_file_dialog_result, handle_file_operations, handle_file_selected};
pub use rendering::render_dialogs;
pub use window_handling::{handle_app_exit, handle_window_close_request};
