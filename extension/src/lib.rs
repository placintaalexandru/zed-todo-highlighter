use zed_extension_api::{self as zed};

struct MyExtension;

impl zed::Extension for MyExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    // TODO
    fn language_server_command(
        &mut self,
        _language_server_id: &zed_extension_api::LanguageServerId,
        _worktree: &zed_extension_api::Worktree,
    ) -> zed_extension_api::Result<zed_extension_api::Command> {
        Ok(zed::Command {
            command:
                "/Users/aplacinta/Documents/Code/Personal/todo-highlight/target/release/todo-highlight-lsp"
                    .to_owned(),
            args: vec![],
            env: vec![],
        })
    }
}

zed::register_extension!(MyExtension);

// grcov-excl-start
#[cfg(test)]
mod tests {
    use super::*;
    use zed::Extension;

    #[test]
    fn test_language_server_command() {
        let _ = MyExtension::new();
    }
}
// grcov-excl-stop
