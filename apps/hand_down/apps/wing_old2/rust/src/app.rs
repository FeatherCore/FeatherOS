use crate::core::FixedList;
use crate::math::Size;
use crate::surface::{SurfaceDescriptor, SurfaceHandle, SurfaceId, SurfaceRequest};

pub const APP_REGISTRY_CAPACITY: usize = 32;
pub const COMMAND_ARG_CAPACITY: usize = 4;

pub const COMMAND_NSH: CommandToken = CommandToken::new("nsh", b"nsh\0");
pub const COMMAND_SH: CommandToken = CommandToken::new("sh", b"sh\0");
pub const COMMAND_DD: CommandToken = CommandToken::new("dd", b"dd\0");
pub const COMMAND_WING_SURFACE_DEMO: CommandToken =
    CommandToken::new("wing_surface_demo", b"wing_surface_demo\0");
pub const COMMAND_WING_SURFACE_RUST_DEMO: CommandToken =
    CommandToken::new("wing_surface_rust_demo", b"wing_surface_rust_demo\0");
pub const COMMAND_WING_SETTINGS: CommandToken =
    CommandToken::new("wing_settings", b"wing_settings\0");
pub const COMMAND_WING_SYSTEM: CommandToken = CommandToken::new("wing_system", b"wing_system\0");
pub const COMMAND_WING_TERMINAL: CommandToken =
    CommandToken::new("wing_terminal", b"wing_terminal\0");
pub const WING_MANAGED_APP_PRIORITY: i32 = 110;
pub const WING_MANAGED_APP_STACK_BYTES: u32 = 65536;
pub const EMPTY_COMMAND_ARGS: &[CommandToken] = &[];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinAppId {
    Settings,
    SystemInfo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandToken {
    pub text: &'static str,
    pub cstr: &'static [u8],
}

impl CommandToken {
    pub const fn new(text: &'static str, cstr: &'static [u8]) -> Self {
        Self { text, cstr }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: CommandToken,
    pub args: &'static [CommandToken],
}

impl CommandSpec {
    pub const fn new(program: CommandToken) -> Self {
        Self {
            program,
            args: EMPTY_COMMAND_ARGS,
        }
    }

    pub const fn with_args(program: CommandToken, args: &'static [CommandToken]) -> Self {
        Self { program, args }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskStdio {
    Inherit,
    Null,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskPriority {
    BuiltinDefault,
    Value(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskStackSize {
    BuiltinDefault,
    Bytes(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskSurface {
    Detached,
    WingManaged(SurfaceRequest),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskLaunchError {
    InvalidProgram,
    InvalidArgument,
    TooManyArguments,
    UnsupportedStdio,
    UnsupportedPriority,
    UnsupportedStackSize,
    UnsupportedSurface,
    InvalidSurface,
    SurfaceCapacity,
    SpawnFailed(i32),
}

impl TaskLaunchError {
    pub const fn code(self) -> i32 {
        match self {
            Self::InvalidProgram => -2,
            Self::InvalidArgument => -3,
            Self::UnsupportedStdio => -4,
            Self::UnsupportedPriority => -5,
            Self::UnsupportedStackSize => -6,
            Self::TooManyArguments => -7,
            Self::UnsupportedSurface => -8,
            Self::InvalidSurface => -9,
            Self::SurfaceCapacity => -10,
            Self::SpawnFailed(code) => code,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::InvalidProgram => "BAD PROGRAM",
            Self::InvalidArgument => "BAD ARG",
            Self::TooManyArguments => "TOO MANY ARGS",
            Self::UnsupportedStdio => "STDIO",
            Self::UnsupportedPriority => "PRIORITY",
            Self::UnsupportedStackSize => "STACK",
            Self::UnsupportedSurface => "SURFACE",
            Self::InvalidSurface => "BAD SURFACE",
            Self::SurfaceCapacity => "SURFACE FULL",
            Self::SpawnFailed(_) => "SPAWN",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformTaskCapabilities {
    pub max_args: u8,
    pub inherit_stdio: bool,
    pub null_stdio: bool,
    pub builtin_default_priority: bool,
    pub custom_priority: bool,
    pub builtin_default_stack: bool,
    pub custom_stack: bool,
    pub detached_surface: bool,
    pub wing_managed_surface: bool,
}

impl PlatformTaskCapabilities {
    pub const NUTTX_EXEC_BUILTIN: Self = Self {
        max_args: COMMAND_ARG_CAPACITY as u8,
        inherit_stdio: true,
        null_stdio: false,
        builtin_default_priority: true,
        custom_priority: false,
        builtin_default_stack: true,
        custom_stack: false,
        detached_surface: true,
        wing_managed_surface: false,
    };

    pub const NUTTX_TASK_RUNNER: Self = Self {
        max_args: COMMAND_ARG_CAPACITY as u8,
        inherit_stdio: true,
        null_stdio: true,
        builtin_default_priority: true,
        custom_priority: true,
        builtin_default_stack: true,
        custom_stack: true,
        detached_surface: true,
        wing_managed_surface: false,
    };

    pub fn validate(self, task: PlatformTask) -> Result<(), TaskLaunchError> {
        if task.command.args.len() > self.max_args as usize {
            return Err(TaskLaunchError::TooManyArguments);
        }

        match task.stdio {
            TaskStdio::Inherit if self.inherit_stdio => {}
            TaskStdio::Null if self.null_stdio => {}
            _ => return Err(TaskLaunchError::UnsupportedStdio),
        }

        match task.priority {
            TaskPriority::BuiltinDefault if self.builtin_default_priority => {}
            TaskPriority::Value(_) if self.custom_priority => {}
            _ => return Err(TaskLaunchError::UnsupportedPriority),
        }

        match task.stack_size {
            TaskStackSize::BuiltinDefault if self.builtin_default_stack => {}
            TaskStackSize::Bytes(_) if self.custom_stack => {}
            _ => return Err(TaskLaunchError::UnsupportedStackSize),
        }

        match task.surface {
            TaskSurface::Detached if self.detached_surface => {}
            TaskSurface::WingManaged(_) if self.wing_managed_surface => {}
            _ => return Err(TaskLaunchError::UnsupportedSurface),
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformTask {
    pub command: CommandSpec,
    pub stdio: TaskStdio,
    pub priority: TaskPriority,
    pub stack_size: TaskStackSize,
    pub surface: TaskSurface,
}

impl PlatformTask {
    pub const fn new(command: CommandSpec) -> Self {
        Self {
            command,
            stdio: TaskStdio::Inherit,
            priority: TaskPriority::BuiltinDefault,
            stack_size: TaskStackSize::BuiltinDefault,
            surface: TaskSurface::Detached,
        }
    }

    pub const fn with_policy(
        command: CommandSpec,
        stdio: TaskStdio,
        priority: TaskPriority,
        stack_size: TaskStackSize,
        surface: TaskSurface,
    ) -> Self {
        Self {
            command,
            stdio,
            priority,
            stack_size,
            surface,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchKind {
    Builtin(BuiltinAppId),
    PlatformTask(PlatformTask),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppManifest {
    pub id: AppId,
    pub name: &'static str,
    pub icon: u16,
    pub launch: LaunchKind,
}

#[derive(Default)]
pub struct AppRegistry {
    entries: FixedList<AppManifest, APP_REGISTRY_CAPACITY>,
}

impl AppRegistry {
    pub fn with_builtin_apps() -> Self {
        let mut registry = Self::default();
        registry.register(AppManifest {
            id: AppId(1),
            name: "Settings",
            icon: 16,
            launch: LaunchKind::PlatformTask(PlatformTask::with_policy(
                CommandSpec::new(COMMAND_WING_SETTINGS),
                TaskStdio::Null,
                TaskPriority::Value(WING_MANAGED_APP_PRIORITY),
                TaskStackSize::Bytes(WING_MANAGED_APP_STACK_BYTES),
                TaskSurface::WingManaged(SurfaceRequest::new(AppId(1), Size::new(320, 360))),
            )),
        });
        registry.register(AppManifest {
            id: AppId(2),
            name: "System",
            icon: 20,
            launch: LaunchKind::PlatformTask(PlatformTask::with_policy(
                CommandSpec::new(COMMAND_WING_SYSTEM),
                TaskStdio::Null,
                TaskPriority::Value(WING_MANAGED_APP_PRIORITY),
                TaskStackSize::Bytes(WING_MANAGED_APP_STACK_BYTES),
                TaskSurface::WingManaged(SurfaceRequest::new(AppId(2), Size::new(320, 360))),
            )),
        });
        registry.register(AppManifest {
            id: AppId(16),
            name: "Terminal",
            icon: 17,
            launch: LaunchKind::PlatformTask(PlatformTask::with_policy(
                CommandSpec::new(COMMAND_WING_TERMINAL),
                TaskStdio::Null,
                TaskPriority::Value(WING_MANAGED_APP_PRIORITY),
                TaskStackSize::Bytes(WING_MANAGED_APP_STACK_BYTES),
                TaskSurface::WingManaged(SurfaceRequest::new(AppId(16), Size::new(320, 360))),
            )),
        });
        registry.register(AppManifest {
            id: AppId(17),
            name: "Surface",
            icon: 21,
            launch: LaunchKind::PlatformTask(PlatformTask::with_policy(
                CommandSpec::new(COMMAND_WING_SURFACE_RUST_DEMO),
                TaskStdio::Null,
                TaskPriority::Value(WING_MANAGED_APP_PRIORITY),
                TaskStackSize::Bytes(WING_MANAGED_APP_STACK_BYTES),
                TaskSurface::WingManaged(SurfaceRequest::new(AppId(17), Size::new(320, 360))),
            )),
        });
        registry
    }

    pub fn register(&mut self, manifest: AppManifest) -> bool {
        for entry in self.entries.as_mut_slice() {
            if entry.id == manifest.id {
                *entry = manifest;
                return true;
            }
        }

        self.entries.push(manifest)
    }

    pub fn get(&self, id: AppId) -> Option<AppManifest> {
        self.entries
            .as_slice()
            .iter()
            .copied()
            .find(|entry| entry.id == id)
    }

    pub fn entries(&self) -> &[AppManifest] {
        self.entries.as_slice()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    pub fn overflowed(&self) -> bool {
        self.entries.overflowed()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppLaunchRequest {
    Builtin(BuiltinAppId),
    PlatformTask(PlatformTask),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformTaskExit {
    pub pid: i32,
    pub status: i32,
}

#[derive(Default)]
pub struct AppManager {
    focused: Option<AppId>,
    pending_request: Option<AppLaunchRequest>,
    last_request: Option<AppLaunchRequest>,
    last_pid: Option<i32>,
    focused_surface: Option<SurfaceId>,
    active_task: Option<ActiveAppTask>,
    last_error: Option<TaskLaunchError>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActiveAppTask {
    pub pid: i32,
    pub surface: Option<SurfaceDescriptor>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppRenderState {
    pub focused: Option<AppId>,
    pub last_pid: Option<i32>,
    pub focused_surface: Option<SurfaceId>,
    pub last_error: Option<TaskLaunchError>,
}

impl AppManager {
    pub fn launch(&mut self, registry: &AppRegistry, id: AppId) -> Option<AppLaunchRequest> {
        let manifest = registry.get(id)?;
        self.focused = Some(id);
        self.focused_surface = None;
        let request = match manifest.launch {
            LaunchKind::Builtin(app) => AppLaunchRequest::Builtin(app),
            LaunchKind::PlatformTask(task) => AppLaunchRequest::PlatformTask(task),
        };
        self.pending_request = Some(request);
        self.last_request = Some(request);
        Some(request)
    }

    pub fn take_launch_request(&mut self) -> Option<AppLaunchRequest> {
        let request = self.pending_request;
        self.pending_request = None;
        request
    }

    pub fn complete_launch(&mut self, pid: i32, surface: Option<SurfaceDescriptor>) {
        self.last_pid = Some(pid);
        self.focused_surface = surface.map(|descriptor| descriptor.id);
        self.active_task = surface.map(|descriptor| ActiveAppTask {
            pid,
            surface: Some(descriptor),
        });
        self.last_error = None;
    }

    pub fn fail_launch(&mut self, error: TaskLaunchError) {
        self.last_pid = None;
        self.focused_surface = None;
        self.active_task = None;
        self.last_error = Some(error);
    }

    pub fn take_active_task(&mut self) -> Option<ActiveAppTask> {
        let active = self.active_task;
        self.active_task = None;
        self.focused = None;
        self.focused_surface = None;
        self.last_pid = None;
        active
    }

    pub fn handle_task_exit(&mut self, exit: PlatformTaskExit) -> Option<ActiveAppTask> {
        if self
            .active_task
            .map(|active| active.pid == exit.pid)
            .unwrap_or(false)
        {
            return self.take_active_task();
        }

        if self.last_pid == Some(exit.pid) {
            self.last_pid = None;
        }

        None
    }

    pub fn handle_surface_closed(
        &mut self,
        handle: SurfaceHandle,
        token: u32,
    ) -> Option<ActiveAppTask> {
        let matches_active_surface = self
            .active_task
            .and_then(|active| active.surface)
            .map(|surface| surface.transport.dirty == handle && surface.transport.token == token)
            .unwrap_or(false);

        if matches_active_surface {
            self.take_active_task()
        } else {
            None
        }
    }

    pub fn has_active_task(&self) -> bool {
        self.active_task.is_some()
    }

    pub fn clear_focus(&mut self) {
        self.focused = None;
        self.focused_surface = None;
        self.active_task = None;
        self.last_pid = None;
    }

    pub fn focused(&self) -> Option<AppId> {
        self.focused
    }

    pub fn last_request(&self) -> Option<AppLaunchRequest> {
        self.last_request
    }

    pub fn last_pid(&self) -> Option<i32> {
        self.last_pid
    }

    pub fn focused_surface(&self) -> Option<SurfaceId> {
        self.focused_surface
    }

    pub fn last_error(&self) -> Option<TaskLaunchError> {
        self.last_error
    }

    pub fn render_state(&self) -> AppRenderState {
        AppRenderState {
            focused: self.focused,
            last_pid: self.last_pid,
            focused_surface: self.focused_surface,
            last_error: self.last_error,
        }
    }
}
