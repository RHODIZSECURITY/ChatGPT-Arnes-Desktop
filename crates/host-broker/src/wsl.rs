use serde::Serialize;
use std::fmt;

#[cfg(target_os = "windows")]
use std::{
    io::Read,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
#[cfg(target_os = "windows")]
use wait_timeout::ChildExt;

pub const DISTRO_NAME: &str = "RHODIZ-Arnes";
#[cfg(any(target_os = "windows", test))]
const SERVICE_NAME: &str = "rhodiz-arnes.service";
#[cfg(target_os = "windows")]
const COMMAND_TIMEOUT: Duration = Duration::from_secs(8);
#[cfg(target_os = "windows")]
const MAX_OUTPUT_BYTES: usize = 64 * 1024;

#[cfg(any(target_os = "windows", test))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandSpec {
    pub program: &'static str,
    pub args: Vec<String>,
}

#[cfg(any(target_os = "windows", test))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

#[cfg(any(target_os = "windows", test))]
trait CommandRunner {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput, BrokerError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrokerError {
    UnsupportedPlatform,
    SpawnFailed,
    CommandTimedOut,
    OutputReadFailed,
    OutputTooLarge,
    CommandFailed(&'static str),
}

impl fmt::Display for BrokerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => formatter.write_str("Windows host required"),
            Self::SpawnFailed => formatter.write_str("host command could not be started"),
            Self::CommandTimedOut => formatter.write_str("host command timed out"),
            Self::OutputReadFailed => formatter.write_str("host command output could not be read"),
            Self::OutputTooLarge => formatter.write_str("host command output exceeded the limit"),
            Self::CommandFailed(operation) => {
                write!(formatter, "host operation failed: {operation}")
            }
        }
    }
}

impl std::error::Error for BrokerError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceState {
    Active,
    Inactive,
    Failed,
    Unknown,
    NotInstalled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeStatus {
    pub platform_supported: bool,
    pub wsl_available: bool,
    pub distro_installed: bool,
    pub docker_ready: bool,
    pub service_state: ServiceState,
    pub detail: Option<String>,
}

#[cfg(target_os = "windows")]
struct SystemCommandRunner;

#[cfg(target_os = "windows")]
impl CommandRunner for SystemCommandRunner {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput, BrokerError> {
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        let mut child = Command::new(spec.program)
            .args(&spec.args)
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| BrokerError::SpawnFailed)?;

        let stdout = child.stdout.take().ok_or(BrokerError::OutputReadFailed)?;
        let stderr = child.stderr.take().ok_or(BrokerError::OutputReadFailed)?;

        let stdout_reader = thread::spawn(move || read_bounded(stdout));
        let stderr_reader = thread::spawn(move || read_bounded(stderr));

        let status = match child
            .wait_timeout(COMMAND_TIMEOUT)
            .map_err(|_| BrokerError::OutputReadFailed)?
        {
            Some(status) => status,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(BrokerError::CommandTimedOut);
            }
        };

        let stdout = join_reader(stdout_reader)?;
        let stderr = join_reader(stderr_reader)?;

        Ok(CommandOutput {
            success: status.success(),
            stdout: String::from_utf8_lossy(&stdout).into_owned(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
        })
    }
}

#[cfg(target_os = "windows")]
fn read_bounded(reader: impl Read) -> Result<Vec<u8>, BrokerError> {
    let mut bytes = Vec::new();
    let mut limited = reader.take((MAX_OUTPUT_BYTES + 1) as u64);
    limited
        .read_to_end(&mut bytes)
        .map_err(|_| BrokerError::OutputReadFailed)?;

    if bytes.len() > MAX_OUTPUT_BYTES {
        return Err(BrokerError::OutputTooLarge);
    }

    Ok(bytes)
}

#[cfg(target_os = "windows")]
fn join_reader(
    handle: thread::JoinHandle<Result<Vec<u8>, BrokerError>>,
) -> Result<Vec<u8>, BrokerError> {
    handle.join().map_err(|_| BrokerError::OutputReadFailed)?
}

pub fn runtime_status() -> RuntimeStatus {
    #[cfg(target_os = "windows")]
    {
        inspect_runtime_with(&SystemCommandRunner)
    }

    #[cfg(not(target_os = "windows"))]
    {
        RuntimeStatus {
            platform_supported: false,
            wsl_available: false,
            distro_installed: false,
            docker_ready: false,
            service_state: ServiceState::Unknown,
            detail: Some("Windows 11 with WSL2 is required.".to_owned()),
        }
    }
}

pub fn start_runtime() -> Result<RuntimeStatus, BrokerError> {
    run_service_action(ServiceAction::Start)
}

pub fn stop_runtime() -> Result<RuntimeStatus, BrokerError> {
    run_service_action(ServiceAction::Stop)
}

fn run_service_action(action: ServiceAction) -> Result<RuntimeStatus, BrokerError> {
    #[cfg(target_os = "windows")]
    {
        let runner = SystemCommandRunner;
        let output = runner.run(&service_action_command(action))?;
        if !output.success {
            return Err(BrokerError::CommandFailed(action.operation_name()));
        }
        Ok(inspect_runtime_with(&runner))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = action;
        Err(BrokerError::UnsupportedPlatform)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ServiceAction {
    Start,
    Stop,
}

impl ServiceAction {
    #[cfg(target_os = "windows")]
    fn operation_name(self) -> &'static str {
        match self {
            Self::Start => "start RHODIZ-Arnes",
            Self::Stop => "stop RHODIZ-Arnes",
        }
    }

    #[cfg(any(target_os = "windows", test))]
    fn systemctl_verb(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
        }
    }
}

#[cfg(any(target_os = "windows", test))]
fn wsl_version_command() -> CommandSpec {
    wsl_command(["--version"])
}

#[cfg(any(target_os = "windows", test))]
fn list_distros_command() -> CommandSpec {
    wsl_command(["--list", "--quiet"])
}

#[cfg(any(target_os = "windows", test))]
fn docker_version_command() -> CommandSpec {
    distro_command(["docker", "version", "--format", "{{.Server.Version}}"])
}

#[cfg(any(target_os = "windows", test))]
fn service_state_command() -> CommandSpec {
    distro_command(["systemctl", "is-active", SERVICE_NAME])
}

#[cfg(any(target_os = "windows", test))]
fn service_action_command(action: ServiceAction) -> CommandSpec {
    distro_command(["systemctl", action.systemctl_verb(), SERVICE_NAME])
}

#[cfg(any(target_os = "windows", test))]
fn wsl_command<const N: usize>(args: [&str; N]) -> CommandSpec {
    CommandSpec {
        program: "wsl.exe",
        args: args.into_iter().map(str::to_owned).collect(),
    }
}

#[cfg(any(target_os = "windows", test))]
fn distro_command<const N: usize>(command: [&str; N]) -> CommandSpec {
    let mut args = vec![
        "-d".to_owned(),
        DISTRO_NAME.to_owned(),
        "-u".to_owned(),
        "root".to_owned(),
        "--exec".to_owned(),
    ];
    args.extend(command.into_iter().map(str::to_owned));

    CommandSpec {
        program: "wsl.exe",
        args,
    }
}

#[cfg(any(target_os = "windows", test))]
fn inspect_runtime_with(runner: &impl CommandRunner) -> RuntimeStatus {
    let version = match runner.run(&wsl_version_command()) {
        Ok(output) if output.success => output,
        _ => {
            return RuntimeStatus {
                platform_supported: true,
                wsl_available: false,
                distro_installed: false,
                docker_ready: false,
                service_state: ServiceState::NotInstalled,
                detail: Some("WSL is unavailable or unhealthy.".to_owned()),
            };
        }
    };

    let _ = version;

    let distros = match runner.run(&list_distros_command()) {
        Ok(output) if output.success => output,
        _ => {
            return RuntimeStatus {
                platform_supported: true,
                wsl_available: true,
                distro_installed: false,
                docker_ready: false,
                service_state: ServiceState::NotInstalled,
                detail: Some(
                    "WSL is present but its distribution list could not be read.".to_owned(),
                ),
            };
        }
    };

    let distro_installed = distros
        .stdout
        .lines()
        .map(str::trim)
        .any(|name| name.eq_ignore_ascii_case(DISTRO_NAME));

    if !distro_installed {
        return RuntimeStatus {
            platform_supported: true,
            wsl_available: true,
            distro_installed: false,
            docker_ready: false,
            service_state: ServiceState::NotInstalled,
            detail: Some("The RHODIZ-Arnes WSL2 distribution is not installed.".to_owned()),
        };
    }

    let docker_ready = runner
        .run(&docker_version_command())
        .is_ok_and(|output| output.success);

    let service_state = runner
        .run(&service_state_command())
        .map(|output| parse_service_state(&output.stdout))
        .unwrap_or(ServiceState::Unknown);

    let detail = if docker_ready && service_state == ServiceState::Active {
        None
    } else if !docker_ready {
        Some("RHODIZ-Arnes is installed but Docker Engine is not ready.".to_owned())
    } else {
        Some("Docker is ready but the ChatGPT Arnes service is not active.".to_owned())
    };

    RuntimeStatus {
        platform_supported: true,
        wsl_available: true,
        distro_installed: true,
        docker_ready,
        service_state,
        detail,
    }
}

#[cfg(any(target_os = "windows", test))]
fn parse_service_state(output: &str) -> ServiceState {
    match output.trim() {
        "active" => ServiceState::Active,
        "inactive" | "activating" | "deactivating" => ServiceState::Inactive,
        "failed" => ServiceState::Failed,
        _ => ServiceState::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, collections::VecDeque};

    struct ScriptedRunner {
        outputs: RefCell<VecDeque<Result<CommandOutput, BrokerError>>>,
    }

    impl ScriptedRunner {
        fn new(outputs: impl IntoIterator<Item = Result<CommandOutput, BrokerError>>) -> Self {
            Self {
                outputs: RefCell::new(outputs.into_iter().collect()),
            }
        }
    }

    impl CommandRunner for ScriptedRunner {
        fn run(&self, _spec: &CommandSpec) -> Result<CommandOutput, BrokerError> {
            self.outputs
                .borrow_mut()
                .pop_front()
                .expect("scripted command result")
        }
    }

    fn success(stdout: &str) -> Result<CommandOutput, BrokerError> {
        Ok(CommandOutput {
            success: true,
            stdout: stdout.to_owned(),
            stderr: String::new(),
        })
    }

    #[test]
    fn all_host_operations_are_fixed_wsl_exe_specs() {
        let specs = [
            wsl_version_command(),
            list_distros_command(),
            docker_version_command(),
            service_state_command(),
            service_action_command(ServiceAction::Start),
            service_action_command(ServiceAction::Stop),
        ];

        for spec in specs {
            assert_eq!(spec.program, "wsl.exe");
            assert!(!spec.args.iter().any(|arg| {
                let lower = arg.to_ascii_lowercase();
                lower.contains("powershell")
                    || lower.contains("cmd.exe")
                    || lower == "sh"
                    || lower == "bash"
                    || lower == "-c"
            }));
        }
    }

    #[test]
    fn lifecycle_commands_target_only_the_fixed_service() {
        let start = service_action_command(ServiceAction::Start);
        let stop = service_action_command(ServiceAction::Stop);

        assert_eq!(
            start.args,
            [
                "-d",
                DISTRO_NAME,
                "-u",
                "root",
                "--exec",
                "systemctl",
                "start",
                SERVICE_NAME
            ]
            .map(str::to_owned)
        );
        assert_eq!(
            stop.args,
            [
                "-d",
                DISTRO_NAME,
                "-u",
                "root",
                "--exec",
                "systemctl",
                "stop",
                SERVICE_NAME
            ]
            .map(str::to_owned)
        );
    }

    #[test]
    fn inspection_reports_a_ready_runtime() {
        let runner = ScriptedRunner::new([
            success("WSL version: 2.6.1"),
            success("Ubuntu\nRHODIZ-Arnes\n"),
            success("28.5.1\n"),
            success("active\n"),
        ]);

        assert_eq!(
            inspect_runtime_with(&runner),
            RuntimeStatus {
                platform_supported: true,
                wsl_available: true,
                distro_installed: true,
                docker_ready: true,
                service_state: ServiceState::Active,
                detail: None,
            }
        );
    }

    #[test]
    fn inspection_fails_closed_when_wsl_is_unavailable() {
        let runner = ScriptedRunner::new([Err(BrokerError::SpawnFailed)]);

        let status = inspect_runtime_with(&runner);
        assert!(!status.wsl_available);
        assert!(!status.distro_installed);
        assert!(!status.docker_ready);
        assert_eq!(status.service_state, ServiceState::NotInstalled);
    }

    #[test]
    fn service_state_parser_is_bounded_to_known_states() {
        assert_eq!(parse_service_state("active\n"), ServiceState::Active);
        assert_eq!(parse_service_state("failed\n"), ServiceState::Failed);
        assert_eq!(parse_service_state("surprise\n"), ServiceState::Unknown);
    }
}
