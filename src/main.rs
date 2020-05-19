mod interop;
use interop::{ro_initialize, RoInitType};

mod window_target;

mod desktopwindowxamlsource;
use desktopwindowxamlsource::IDesktopWindowXamlSourceNative;

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use bindings::windows::ui::Color;
use bindings::windows::ui::xaml::{HorizontalAlignment, VerticalAlignment};
use bindings::windows::ui::xaml::controls::{StackPanel, IStackPanelFactory, TextBlock};
use bindings::windows::ui::xaml::hosting::{DesktopWindowXamlSource, IDesktopWindowXamlSourceFactory, WindowsXamlManager};
use bindings::windows::ui::xaml::media::SolidColorBrush;

use winrt::Object;

use raw_window_handle::HasRawWindowHandle;
use core::ptr;

fn main() -> winrt::Result<()> {
    ro_initialize(RoInitType::MultiThreaded)?;

    // Initialize XAML and create desktop source **before** making the
    // event_loop, otherwise winit panics with a weird error: thread 'main'
    // panicked at 'either event handler is re-entrant (likely), or no event
    // handler is registered (very unlikely)',
    // winit-0.22.2\src\platform_impl\windows\event_loop\runner.rs:235:37.
    let _win_xaml_mgr = WindowsXamlManager::initialize_for_current_thread();
    let desktop_source = winrt::factory::<DesktopWindowXamlSource, IDesktopWindowXamlSourceFactory>()?.create_instance(Object::default(), &mut Object::default())?;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("lol");

    let idesktop_source: IDesktopWindowXamlSourceNative = desktop_source.clone().into();
    idesktop_source.attach_to_window(&window)?;

    let hwnd_xaml_island = idesktop_source.get_window_handle()?;

    // By default, the xaml_island will have a size of 0, 0. That's bad, we want
    // to fix it!
    let size = window.inner_size();
    unsafe { SetWindowPos(hwnd_xaml_island, ptr::null_mut(), 0, 0, size.width as i32, size.height as i32, /*SWP_SHOWWINDOW*/ 0x40); }

    let xaml_container = winrt::factory::<StackPanel, IStackPanelFactory>()?.create_instance(Object::default(), &mut Object::default())?;
    let grey_brush = SolidColorBrush::new()?;
    grey_brush.set_color(Color { r: 0x33, g: 0x33, b: 0x33, a: 255})?;
    xaml_container.set_background(grey_brush)?;

    let tb = TextBlock::new()?;
    tb.set_text("Hello World from XAML Islands!")?;
    tb.set_vertical_alignment(VerticalAlignment::Center)?;
    tb.set_horizontal_alignment(HorizontalAlignment::Center)?;
    tb.set_font_size(48.)?;

    xaml_container.children()?.append(tb)?;
    xaml_container.update_layout()?;
    desktop_source.set_content(xaml_container)?;

    let hwnd = match window.raw_window_handle() {
        raw_window_handle::RawWindowHandle::Windows(window_handle) => window_handle.hwnd,
        _ => panic!("Unsupported platform!"),
    };

    unsafe { UpdateWindow(hwnd); }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                window_id
            } if window_id == window.id() => {
                unsafe { SetWindowPos(hwnd_xaml_island, ptr::null_mut(), 0, 0, size.width as i32, size.height as i32, /*SWP_SHOWWINDOW*/ 0x40); }
            }
            _ => (),
        }
    });
}

#[link(name = "user32")]
extern "stdcall" {
    fn UpdateWindow(
        hwnd: *mut core::ffi::c_void,
    ) -> i32;
    fn SetWindowPos(
        hwnd: *mut core::ffi::c_void,
        hwnd_insert_after: *mut core::ffi::c_void,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        flags: u32
    ) -> i32;
}