use gpu_bytes_derive::{AsStd140, AsStd430};

use crate::engine::profiler_state::ProfilerState;

use super::{DynamicBuffer, UpdateFromSource, MIN_DYNAMIC_BUFFER_CAPACITY};

pub const PROFILER_STEP_SIZE: usize = 10;
pub const PROFILER_STEP_COUNT: usize = 60;

#[derive(AsStd140, AsStd430)]
pub struct ProfilerUniform {
    pub num_frametimes: u32,
    pub list: Vec<f32>,
}

impl UpdateFromSource<ProfilerState> for ProfilerUniform {
    fn update(&mut self, source: &ProfilerState) {
        self.list.insert(0, source.immediate_ms());
        self.list.truncate(PROFILER_STEP_COUNT);
        self.num_frametimes = self.list.len() as u32;
    }
}

impl Default for ProfilerUniform {
    fn default() -> Self {
        Self {
            num_frametimes: 0,
            list: Vec::with_capacity(PROFILER_STEP_COUNT),
        }
    }
}

pub type ProfilerBuffer = DynamicBuffer<ProfilerUniform, ProfilerState>;
