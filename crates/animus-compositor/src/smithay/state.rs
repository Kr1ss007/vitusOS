#![cfg(target_os = "linux")]

use smithay::{
    delegate_compositor, delegate_shm, delegate_xdg_shell, delegate_seat, delegate_output,
    backend::renderer::utils::on_commit_buffer_handler,
    input::{Seat, SeatState, SeatHandler, pointer::CursorImageStatus},
    wayland::{
        buffer::BufferHandler,
        compositor::{CompositorHandler, CompositorState, CompositorClientState},
        output::OutputHandler,
        shm::{ShmHandler, ShmState},
        shell::xdg::{XdgShellHandler, XdgShellState, ToplevelSurface, PopupSurface, PositionerState},
    },
};
use wayland_server::backend::ClientData;
use wayland_server::protocol::{wl_surface::WlSurface, wl_buffer::WlBuffer};

pub struct ClientState {
    pub compositor_state: CompositorClientState,
}
impl ClientData for ClientState {
    fn initialized(&self, _client_id: wayland_server::backend::ClientId) {}
    fn disconnected(&self, _client_id: wayland_server::backend::ClientId, _reason: wayland_server::backend::DisconnectReason) {}
}

pub struct SmithayState {
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<SmithayState>,
}

impl SmithayState {
    pub fn new(display: &mut wayland_server::Display<Self>) -> Self {
        let dh = display.handle();
        let compositor_state = CompositorState::new::<Self>(&dh);
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);
        let seat_state = SeatState::new();

        Self {
            compositor_state,
            xdg_shell_state,
            shm_state,
            seat_state,
        }
    }
}

impl CompositorHandler for SmithayState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }
    fn client_compositor_state<'a>(&self, client: &'a wayland_server::Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }
    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
    }
}

impl ShmHandler for SmithayState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl BufferHandler for SmithayState {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl SeatHandler for SmithayState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}
    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}
}

impl OutputHandler for SmithayState {}

impl XdgShellHandler for SmithayState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }
    fn new_toplevel(&mut self, _surface: ToplevelSurface) {
        tracing::info!("Wayland client mapped a new toplevel window");
    }
    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {}
    fn reposition_request(&mut self, _surface: PopupSurface, _positioner: PositionerState, _token: u32) {}
    fn grab(&mut self, _surface: PopupSurface, _seat: wayland_server::protocol::wl_seat::WlSeat, _serial: smithay::utils::Serial) {}
}

delegate_compositor!(SmithayState);
delegate_shm!(SmithayState);
delegate_xdg_shell!(SmithayState);
delegate_seat!(SmithayState);
delegate_output!(SmithayState);
