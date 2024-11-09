use super::render_state::RenderState;

use egui_wgpu::RenderState as EguiRenderState;

pub struct UiState {}

// impl UiState {
//     pub fn init(render_state: &RenderState) -> Self {
//         let egui_state = pollster::block_on(EguiRenderState::create(
//             &egui_wgpu::WgpuConfiguration::default(),
//             &render_state.instance,
//             &render_state.surface,
//             None,
//             1,
//             false,
//         ))
//         .unwrap();
//     }
// }
