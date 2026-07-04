//! Contains all types and implementations related to the plugin's GUI
slint::include_modules!();

use crate::{params::GainParamsShared, GainPluginMainThread, GainPluginShared};
use baseview::{WindowHandle as BaseviewWindowHandle, WindowOpenOptions, WindowScalePolicy};
use clack_extensions::gui::*;
use clack_plugin::prelude::*;
use slint_baseview::slint_window::SlintWindow;
use std::sync::Arc;

/// The slint application state
struct AppState {
    /// A handle to the shared params state
    shared_params: Arc<GainParamsShared>,
}

impl AppState {
    /// Initializes a new [`AppState`] from the given shared params state handle
    pub fn new(shared_params: &Arc<GainParamsShared>) -> Self {
        Self {
            shared_params: Arc::clone(shared_params),
        }
    }
}

/// GUI state that can be accessed directly by the main thread
pub struct GainPluginGui {
    /// The handle to the baseview plugin window.
    handle: BaseviewWindowHandle,
}

impl GainPluginGui {
    /// Creates a new GUI window, and embeds it into the given `parent`.
    pub fn new(parent: Window<'_>, state: &GainPluginShared) -> Self {
        let settings = WindowOpenOptions {
            title: "Gain Plugin".to_string(),
            size: baseview::Size {
                width: 400.0,
                height: 400.0,
            },
            scale: WindowScalePolicy::SystemScaleFactor,
            gl_config: Some(Default::default()),
        };

        let handle = SlintWindow::open_parented(
            &parent,
            settings,
            AppState::new(&state.params),
            |_, _| {},
            |state| {
                let params = state.shared_params.clone();
                let component = AppWindow::new().unwrap();
                component.on_begin_drag(|| eprintln!("on_gain_begin_drag"));
                component.on_changed(move |value| {
                    params.volume.swap(value);
                });
                component.on_end_drag(|| eprintln!("on_gain_end_drag"));
                component
            },
        );

        Self { handle }
    }

    /// Requests the UI to repaint itself, e.g. in response to events or parameter changes
    pub fn request_repaint(&self) {}
}

impl Drop for GainPluginGui {
    fn drop(&mut self) {
        self.handle.close();
    }
}

impl<'a> PluginGuiImpl for GainPluginMainThread<'a> {
    fn is_api_supported(&mut self, configuration: GuiConfiguration) -> bool {
        configuration.api_type
            == GuiApiType::default_for_current_platform().expect("Unsupported platform")
            && !configuration.is_floating
    }

    fn get_preferred_api(&mut self) -> Option<GuiConfiguration<'_>> {
        Some(GuiConfiguration {
            api_type: GuiApiType::default_for_current_platform().expect("Unsupported platform"),
            is_floating: false,
        })
    }

    fn create(&mut self, configuration: GuiConfiguration) -> Result<(), PluginError> {
        if configuration.is_floating {
            return Err(PluginError::Message(
                "Invalid GUI configuration: this plugin does not support floating mode",
            ));
        }

        let supported_type =
            GuiApiType::default_for_current_platform().expect("Unsupported platform");

        if configuration.api_type != supported_type {
            return Err(PluginError::Message(
                "Invalid GUI configuration: unsupported API type",
            ));
        }

        Ok(())
    }

    fn destroy(&mut self) {
        let _ = self.gui.take();
    }

    fn set_scale(&mut self, _scale: f64) -> Result<(), PluginError> {
        Ok(())
    }

    fn get_size(&mut self) -> Option<GuiSize> {
        Some(GuiSize {
            width: 400,
            height: 360,
        })
    }

    fn set_size(&mut self, _size: GuiSize) -> Result<(), PluginError> {
        Ok(())
    }

    fn set_parent(&mut self, window: Window) -> Result<(), PluginError> {
        self.gui = Some(GainPluginGui::new(window, self.shared));
        Ok(())
    }

    fn set_transient(&mut self, _window: Window) -> Result<(), PluginError> {
        Ok(())
    }

    fn show(&mut self) -> Result<(), PluginError> {
        if let Some(gui) = &self.gui {
            gui.request_repaint()
        }
        Ok(())
    }

    fn hide(&mut self) -> Result<(), PluginError> {
        if let Some(gui) = &self.gui {
            gui.request_repaint()
        }

        Ok(())
    }
}
