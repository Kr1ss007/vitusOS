//! vitusOS Live ISO Setup Assistant & Installer Binary.

use animus_core::event_bus::EventBus;
use tracing::info;
use vitusos_installer::SetupWizard;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("================================================================================");
    info!("             vitusOS macOS-Grade Setup Assistant & Installer Engine            ");
    info!("================================================================================");

    let bus = EventBus::new();
    let mut wizard = SetupWizard::new(bus);

    wizard.activate();
    info!("Wizard active on step: {:?} ('{}')", wizard.current_step, wizard.current_step.title());
    info!("Subtext: '{}'", wizard.current_step.subtitle());
    info!("Detected Storage Targets: {} device(s)", wizard.available_disks.len());

    for disk in &wizard.available_disks {
        info!(" -> Target Drive: {} ({}) [{:?}]", disk.model, disk.formatted_size(), disk.transport);
    }

    // Step forward demonstration
    wizard.advance();
    info!("Stepped forward to: {:?}", wizard.current_step);

    wizard.advance();
    info!("Stepped forward to: {:?}", wizard.current_step);

    wizard.password_input = "VitusOS!2026MasterKey".to_string();
    info!("Evaluated Password Strength: {:?}", wizard.password_strength());

    info!("Installer initialized and ready for compositor surface presentation.");
    Ok(())
}
