use std::collections::VecDeque;

pub struct ProfilerState {
    delta_times: VecDeque<std::time::Duration>,
    memory: usize,
}

impl ProfilerState {
    pub fn new(memory: usize) -> Self {
        Self {
            delta_times: VecDeque::with_capacity(memory),
            memory,
        }
    }

    pub fn memory(&self) -> usize {
        self.memory
    }

    pub fn set_memory(&mut self, memory: usize) {
        self.memory = memory;
    }

    pub fn update(&mut self, delta: std::time::Duration) {
        self.delta_times.push_front(delta);
        self.delta_times.truncate(self.memory);
    }

    pub fn immediate_ms(&self) -> f32 {
        self.delta_times[0].as_secs_f32() * 1000.0
    }

    pub fn average_ms(&self) -> f32 {
        self.delta_times
            .iter()
            .map(|d| d.as_secs_f32() * 1000.0)
            .sum::<f32>()
            / self.delta_times.len() as f32
    }

    pub fn average_fps(&self) -> f32 {
        1000.0 / self.average_ms()
    }
}
