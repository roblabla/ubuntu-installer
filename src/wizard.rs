use bindings::windows::ui::xaml::hosting::{DesktopWindowXamlSource};
use bindings::windows::ui::Color;
use bindings::windows::ui::xaml::{HorizontalAlignment, VerticalAlignment, UIElement, RoutedEventHandler};
use bindings::windows::ui::xaml::media::SolidColorBrush;
use bindings::windows::ui::xaml::controls::{StackPanel, IStackPanelFactory, TextBlock, Button, IButtonFactory};
use bindings::windows::foundation::PropertyValue;

use winrt::Object;
use winit::window::Window;
use raw_window_handle::HasRawWindowHandle;
use winit::event_loop::EventLoopProxy;

pub struct WizardUI {
    window: Window,
    desktop_source: DesktopWindowXamlSource,
    el_proxy: EventLoopProxy<()>,
    step: WizardStep,
}

impl WizardUI {
    pub fn new(win32_window: Window, xaml_source: DesktopWindowXamlSource, el: EventLoopProxy<()>) -> winrt::Result<WizardUI> {
        let ui = WizardUI {
            window: win32_window,
            desktop_source: xaml_source,
            el_proxy: el.clone(),
            step: WizardStep::new(el)?,
        };

        ui.update_window()?;

        Ok(ui)
    }

    fn update_window(&self) -> winrt::Result<()> {
        self.desktop_source.set_content(self.step.top_level())?;
        let hwnd = match self.window.raw_window_handle() {
            raw_window_handle::RawWindowHandle::Windows(window_handle) => window_handle.hwnd,
            _ => panic!("Unsupported platform!"),
        };

        unsafe { UpdateWindow(hwnd); }

        Ok(())
    }
}

enum WizardStep {
    Step1 {
        container: StackPanel
    },
}

fn make_tb(s: &str) -> winrt::Result<TextBlock> {
    let tb = TextBlock::new()?;
    tb.set_text(s)?;
    tb.set_vertical_alignment(VerticalAlignment::Top)?;
    tb.set_horizontal_alignment(HorizontalAlignment::Center)?;
    Ok(tb)
}

impl WizardStep {
    fn new(el_proxy: EventLoopProxy<()>) -> winrt::Result<WizardStep> {
        let xaml_container = winrt::factory::<StackPanel, IStackPanelFactory>()?.create_instance(Object::default(), &mut Object::default())?;
        let grey_brush = SolidColorBrush::new()?;
        grey_brush.set_color(Color { r: 0x33, g: 0x33, b: 0x33, a: 255})?;
        xaml_container.set_background(grey_brush)?;

        let title = make_tb("Ubuntu Media Creation Wizard")?;
        title.set_font_size(48.)?;

        xaml_container.children()?.append(title)?;

        let next_btn = winrt::factory::<Button, IButtonFactory>()?.create_instance(Object::default(), &mut Object::default())?;
        let next_s: Object = PropertyValue::create_string("Next")?.into();
        next_btn.set_content(next_s)?;
        next_btn.set_vertical_alignment(VerticalAlignment::Bottom)?;
        next_btn.set_horizontal_alignment(HorizontalAlignment::Center)?;
        next_btn.click(RoutedEventHandler::new(|_, _| {
            let _ = el_proxy.send_event(());
            Ok(())
        }))?;

        xaml_container.children()?.append(next_btn)?;

        xaml_container.update_layout()?;

        Ok(WizardStep::Step1 {
            container: xaml_container
        })
    }

    fn top_level(&self) -> UIElement {
        match self {
            WizardStep::Step1 { ref container } => container.into()
        }
    }
}

#[link(name = "user32")]
extern "stdcall" {
    fn UpdateWindow(
        hwnd: *mut core::ffi::c_void,
    ) -> i32;
}