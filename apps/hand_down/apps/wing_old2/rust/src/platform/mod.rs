use alloc::vec::Vec;

use crate::app::{PlatformTask, PlatformTaskCapabilities, PlatformTaskExit, TaskLaunchError};
use crate::input::InputState;
use crate::render::{DirtyRegion, ResourcePartition};
use crate::resource_file::{
    RuntimeResourceId, RuntimeResourceInfo, RuntimeResourceSummary, RUNTIME_RESOURCE_IDS,
};
use crate::settings::{SettingsEvent, SettingsSnapshot, SettingsStorageSource};
use crate::surface::{
    SurfaceCapabilities, SurfaceDescriptor, SurfaceEvent, SurfaceFrame, SurfaceHandle,
    SurfacePointerEvent,
};

pub mod nuttx;

pub trait Platform {
    fn dimensions(&self) -> (u16, u16);
    fn poll_input(&mut self, input: &mut InputState);
    fn present(&mut self, framebuffer: &[u32], dirty: DirtyRegion);
    fn poll_surface_event(&mut self) -> Option<SurfaceEvent>;
    fn poll_settings_event(&mut self) -> Option<SettingsEvent>;
    fn load_settings_snapshot(&mut self) -> Option<SettingsSnapshot> {
        None
    }
    fn save_settings_snapshot(&mut self, _snapshot: SettingsSnapshot) -> bool {
        true
    }
    fn settings_storage_source(&self) -> SettingsStorageSource {
        SettingsStorageSource::Memory
    }
    fn resource_partition(&self) -> ResourcePartition {
        ResourcePartition::builtin()
    }
    fn inspect_runtime_resource(&mut self, id: RuntimeResourceId) -> RuntimeResourceInfo {
        RuntimeResourceInfo::missing(id)
    }
    fn load_runtime_resource(&mut self, _id: RuntimeResourceId) -> Option<Vec<u8>> {
        None
    }
    fn load_runtime_resource_file(
        &mut self,
        _path: &'static [u8],
        _capacity: usize,
    ) -> Option<Vec<u8>> {
        None
    }
    fn inspect_runtime_resources(&mut self) -> RuntimeResourceSummary {
        let mut summary = RuntimeResourceSummary::default();
        for id in RUNTIME_RESOURCE_IDS {
            summary.include(self.inspect_runtime_resource(id));
        }
        summary
    }
    fn send_surface_input(&mut self, event: SurfacePointerEvent) -> bool;
    fn poll_task_exit(&mut self) -> Option<PlatformTaskExit>;
    fn resolve_surface_frame(&mut self, handle: SurfaceHandle, token: u32) -> Option<SurfaceFrame>;
    fn task_capabilities(&self) -> PlatformTaskCapabilities;
    fn surface_capabilities(&self) -> SurfaceCapabilities;
    fn launch_task(
        &mut self,
        task: PlatformTask,
        surface: Option<SurfaceDescriptor>,
        settings: Option<SettingsSnapshot>,
    ) -> Result<i32, TaskLaunchError>;
    fn close_task(&mut self, pid: i32, surface: Option<SurfaceDescriptor>);
    fn is_running(&self) -> bool;
    fn sleep_ms(&mut self, ms: u32);
}
