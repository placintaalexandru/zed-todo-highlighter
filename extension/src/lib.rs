use std::fs;

use zed_extension_api::{self as zed, Architecture, DownloadedFileType, Os, Result};

enum Status {
    None,
    Downloading,
    Failed(String),
}

const LSP_NAME_UNIX: &str = "todo-highlight-lsp";
const LSP_NAME_WINDOWS: &str = "todo-highlight-lsp.ext";

#[inline]
const fn bin_name(platform: (Os, Architecture)) -> &'static str {
    match platform {
        (Os::Windows, _) => LSP_NAME_WINDOWS,
        _ => LSP_NAME_UNIX,
    }
}

fn update_status(id: &zed::LanguageServerId, status: Status) {
    match status {
        Status::None => zed::set_language_server_installation_status(
            id,
            &zed::LanguageServerInstallationStatus::None,
        ),
        Status::Downloading => zed::set_language_server_installation_status(
            id,
            &zed::LanguageServerInstallationStatus::Downloading,
        ),
        Status::Failed(msg) => zed::set_language_server_installation_status(
            id,
            &zed::LanguageServerInstallationStatus::Failed(msg),
        ),
    }
}

#[derive(Debug, Default)]
struct TodoHighlightExtension {
    cached_binary_path: Option<String>,
}

impl TodoHighlightExtension {
    fn check_to_update(id: &zed::LanguageServerId) -> Result<String> {
        const GITHUB_REPO: &str = "placintaalexandru/todo-highlighter";

        let platform = zed::current_platform();
        let (os, arch) = platform;
        let release = zed::latest_github_release(
            GITHUB_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let asset_name = format!(
            "{LSP_NAME_UNIX}-{os}-{arch}.{ext}",
            arch = match arch {
                Architecture::Aarch64 => "arm64",
                Architecture::X86 => "amd64",
                Architecture::X8664 => "amd64",
            },
            os = match os {
                Os::Mac => "darwin",
                Os::Linux => "linux",
                Os::Windows => "windows",
            },
            ext = match os {
                Os::Windows => "zip",
                _ => "tar.gz",
            }
        );

        let file_type = match os {
            Os::Windows => DownloadedFileType::Zip,
            _ => DownloadedFileType::GzipTar,
        };

        let version_dir = format!("{LSP_NAME_UNIX}-{}", release.version);
        let bin_name = bin_name(platform);
        let version_binary_path = format!("{version_dir}/{bin_name}");

        if !fs::metadata(&version_binary_path).map_or(false, |stat| stat.is_file()) {
            update_status(id, Status::Downloading);

            let asset = release
                .assets
                .iter()
                .find(|asset| asset.name == asset_name)
                .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;
            zed::download_file(&asset.download_url, &version_dir, file_type)
                .map_err(|e| format!("failed to download file: {e}"))?;

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }

            update_status(id, Status::None);
        }

        Ok(version_binary_path)
    }

    fn language_server_binary_path(
        &mut self,
        id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        let platform = zed::current_platform();
        let bin_name = bin_name(platform);

        // Check if the binary is already installed by manually checking the path
        if let Some(path) = worktree.which(bin_name) {
            return Ok(path);
        }

        if let Some(ref path) = self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                update_status(id, Status::None);
                return Ok(path.clone());
            }
        }

        if let Some(binary_path) = Self::check_installed() {
            // silent to check for update.
            let _ = Self::check_to_update(&id);
            return Ok(binary_path);
        }

        let version_binary_path = Self::check_to_update(id)?;
        self.cached_binary_path = Some(version_binary_path.clone());
        Ok(version_binary_path)
    }

    fn check_installed() -> Option<String> {
        let entries = fs::read_dir(".").ok()?;
        for entry in entries.flatten().filter(|entry| entry.path().is_dir()) {
            let binary_path = entry.path().join(bin_name(zed::current_platform()));
            if fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
                return binary_path.to_str().map(|s| s.to_string());
            }
        }

        None
    }
}

impl zed::Extension for TodoHighlightExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed_extension_api::LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> zed_extension_api::Result<zed_extension_api::Command> {
        let command = self
            .language_server_binary_path(language_server_id, worktree)
            .inspect_err(|err| {
                update_status(language_server_id, Status::Failed(err.to_string()));
            })?;

        Ok(zed::Command {
            command,
            args: vec![],
            env: Default::default(),
        })
    }
}

zed::register_extension!(TodoHighlightExtension);

// grcov-excl-start
#[cfg(test)]
mod tests {
    use super::*;
    use zed::Extension;

    #[test]
    fn test_language_server_command() {
        let _ = TodoHighlightExtension::new();
    }
}
// grcov-excl-stop
