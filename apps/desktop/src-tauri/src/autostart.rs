//! Login-session host agent. Screen capture still requires an accepted invite
//! in the interactive user session; this is not a privileged OS service.
use std::env;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::{fs, path::PathBuf};

#[tauri::command]
pub fn autostart_status() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        return Ok(std::process::Command::new("reg.exe")
            .args([
                "query",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "SevenDesk",
            ])
            .output()
            .map_err(|_| "Não foi possível consultar a inicialização do Windows".to_string())?
            .status
            .success());
    }
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        Ok(autostart_path()?.is_file())
    }
}

#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let mut command = std::process::Command::new("reg.exe");
        if enabled {
            let executable = env::current_exe()
                .map_err(|_| "Não foi possível localizar o aplicativo".to_string())?;
            let value = format!("\"{}\" --background", executable.display());
            command.args([
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "SevenDesk",
                "/t",
                "REG_SZ",
                "/d",
                &value,
                "/f",
            ]);
        } else {
            command.args([
                "delete",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "SevenDesk",
                "/f",
            ]);
        }
        if !command
            .output()
            .map_err(|_| "Não foi possível alterar a inicialização do Windows".to_string())?
            .status
            .success()
        {
            return Err("O Windows recusou a alteração de inicialização".into());
        }
        return autostart_status();
    }
    #[cfg(target_os = "macos")]
    {
        let path = autostart_path()?;
        if enabled {
            let executable = env::current_exe()
                .map_err(|_| "Não foi possível localizar o aplicativo".to_string())?;
            let escaped = executable
                .to_string_lossy()
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;");
            let contents = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?><!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\"><plist version=\"1.0\"><dict><key>Label</key><string>dev.seven.desk</string><key>ProgramArguments</key><array><string>{escaped}</string><string>--background</string></array><key>RunAtLoad</key><true/></dict></plist>"
            );
            write_autostart(&path, contents.as_bytes())?;
        } else if path.exists() {
            fs::remove_file(&path)
                .map_err(|_| "Não foi possível desativar a inicialização automática".to_string())?;
        }
        return autostart_status();
    }
    #[cfg(target_os = "linux")]
    {
        let path = autostart_path()?;
        if enabled {
            let executable = env::current_exe()
                .map_err(|_| "Não foi possível localizar o aplicativo".to_string())?;
            let escaped = executable
                .to_string_lossy()
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('%', "%%");
            let contents = format!(
                "[Desktop Entry]\nType=Application\nName=Seven Desk\nExec=\"{escaped}\" --background\nTerminal=false\nX-GNOME-Autostart-enabled=true\n"
            );
            write_autostart(&path, contents.as_bytes())?;
        } else if path.exists() {
            fs::remove_file(&path)
                .map_err(|_| "Não foi possível desativar a inicialização automática".to_string())?;
        }
        autostart_status()
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn autostart_path() -> Result<PathBuf, String> {
    let home = env::var_os("HOME").ok_or_else(|| "Pasta pessoal indisponível".to_string())?;
    #[cfg(target_os = "macos")]
    {
        Ok(PathBuf::from(home).join("Library/LaunchAgents/dev.seven.desk.plist"))
    }
    #[cfg(target_os = "linux")]
    {
        let base = env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(home).join(".config"));
        Ok(base.join("autostart/seven-desk.desktop"))
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn write_autostart(path: &PathBuf, contents: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Pasta de inicialização inválida".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|_| "Não foi possível criar a pasta de inicialização".to_string())?;
    fs::write(path, contents)
        .map_err(|_| "Não foi possível salvar a inicialização automática".to_string())
}
