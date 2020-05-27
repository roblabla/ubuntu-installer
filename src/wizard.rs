use bindings::windows::ui::xaml::hosting::{DesktopWindowXamlSource};
use bindings::windows::ui::Color;
use bindings::windows::ui::xaml::{UIElement, RoutedEventHandler, Thickness, TextWrapping};
use bindings::windows::ui::xaml::media::SolidColorBrush;
use bindings::windows::ui::xaml::controls::*;
use bindings::windows::foundation::PropertyValue;

use bindings::windows::foundation::TypedEventHandler;
use bindings::windows::devices::enumeration::{DeviceClass, DeviceInformation, DeviceWatcher};

//use bindings::windows::storage::ApplicationData;

use winrt::Object;
use winit::window::Window;
use raw_window_handle::HasRawWindowHandle;
use winit::event_loop::EventLoopProxy;
use winapi::um::fileapi::GetVolumePathNamesForVolumeNameW;
use winapi::shared::minwindef::MAX_PATH;
use winapi::um::errhandlingapi::GetLastError;
use winapi::shared::ntdef::LPWSTR;
use winapi::shared::minwindef::HLOCAL;
use winapi::um::winbase::*;
use widestring::U16String;

use std::ptr;
use std::os::windows::io::AsRawHandle;
use std::path::PathBuf;
use std::thread::JoinHandle;

use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::stream::StreamExt;

pub struct WizardUI {
    window: Window,
    desktop_source: DesktopWindowXamlSource,
    el_proxy: EventLoopProxy<WizardEvent>,
    step: WizardStep,
}

impl WizardUI {
    pub fn new(win32_window: Window, xaml_source: DesktopWindowXamlSource, el: EventLoopProxy<WizardEvent>) -> winrt::Result<WizardUI> {
        let ui = WizardUI {
            window: win32_window,
            desktop_source: xaml_source,
            el_proxy: el.clone(),
            step: WizardStep::step1(el)?,
        };

        ui.update_window()?;

        Ok(ui)
    }

    pub fn go_to_step2(&mut self) -> winrt::Result<()> {
        self.step = WizardStep::step2(self.el_proxy.clone())?;
        self.update_window()?;
        Ok(())
    }

    pub fn go_to_step3(&mut self) -> winrt::Result<()> {
        self.step = WizardStep::step3(self.el_proxy.clone())?;
        self.update_window()?;
        Ok(())
    }

    pub fn add_usb_device(&mut self, device: &DeviceNameId) -> winrt::Result<()> {
        self.step.add_usb_device(device)?;
        self.update_window()?;
        Ok(())
    }

    pub fn set_progress(&mut self, cur: u64, total: Option<u64>) -> winrt::Result<()> {
        self.step.set_progress(cur, total)?;
        self.update_window()?;
        Ok(())
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
        container: RelativePanel
    },
    Step2 {
        container: RelativePanel,
        usb_list: ListBox,
        _watcher: DeviceWatcher,
    },
    Step3 {
        container: RelativePanel,
        _handle: JoinHandle<()>,
        progress_bar: ProgressBar,
    }
}

fn make_tb(s: &str) -> winrt::Result<TextBlock> {
    let tb = TextBlock::new()?;
    tb.set_text(s)?;
    Ok(tb)
}

impl WizardStep {
    fn step1(el_proxy: EventLoopProxy<WizardEvent>) -> winrt::Result<WizardStep> {
        let xaml_container = winrt::factory::<RelativePanel, IRelativePanelFactory>()?.create_instance(Object::default(), &mut Object::default())?;
        let grey_brush = SolidColorBrush::new()?;
        grey_brush.set_color(Color { r: 0x5e, g: 0x27, b: 0x50, a: 255})?;
        xaml_container.set_background(grey_brush)?;

        let title = make_tb("Ubuntu Media Creation Wizard")?;
        title.set_font_size(48.)?;
        RelativePanel::set_align_horizontal_center_with_panel(&title, true)?;
        title.set_margin(Thickness {
            top: 10.,
            ..Default::default()
        })?;
        xaml_container.children()?.append(&title)?;

        let explanation = make_tb("The installer will guide you through the steps required to create a bootable Ubuntu USB flash drive, and reboot on it.")?;
        explanation.set_text_wrapping(TextWrapping::Wrap)?;
        explanation.set_margin(Thickness {
            top: 10., left: 10., right: 10., bottom: 10.,
        })?;
        RelativePanel::set_below(&explanation, Object::from(title))?;
        xaml_container.children()?.append(&explanation)?;

        let next_btn = winrt::factory::<Button, IButtonFactory>()?.create_instance(Object::default(), &mut Object::default())?;
        let next_s: Object = PropertyValue::create_string("Next")?.into();
        next_btn.set_content(next_s)?;
        next_btn.set_margin(Thickness {
            top: 0., left: 0., right: 10., bottom: 10.
        })?;
        next_btn.click(RoutedEventHandler::new(move |_, _| {
            let _ = el_proxy.send_event(WizardEvent::GoToStep2);
            Ok(())
        }))?;
        RelativePanel::set_align_bottom_with_panel(&next_btn, true)?;
        RelativePanel::set_align_right_with_panel(&next_btn, true)?;

        xaml_container.children()?.append(next_btn)?;

        xaml_container.update_layout()?;

        Ok(WizardStep::Step1 {
            container: xaml_container
        })
    }

    pub fn step2(el_proxy: EventLoopProxy<WizardEvent>) -> winrt::Result<WizardStep> {
        let xaml_container = winrt::factory::<RelativePanel, IRelativePanelFactory>()?.create_instance(Object::default(), &mut Object::default())?;
        let grey_brush = SolidColorBrush::new()?;
        grey_brush.set_color(Color { r: 0x5e, g: 0x27, b: 0x50, a: 255})?;
        xaml_container.set_background(grey_brush)?;

        let title = make_tb("Select a USB Flash Drive")?;
        title.set_font_size(48.)?;
        RelativePanel::set_align_horizontal_center_with_panel(&title, true)?;
        title.set_margin(Thickness {
            top: 10., ..Thickness::default()
        })?;
        xaml_container.children()?.append(&title)?;

        let explanation = make_tb("The files on your USB will be deleted. Please back them up to a safe location before proceeding.")?;
        explanation.set_text_wrapping(TextWrapping::Wrap)?;
        explanation.set_margin(Thickness {
            top: 10., left: 10., right: 10., bottom: 10.,
        })?;
        RelativePanel::set_below(&explanation, Object::from(title))?;
        xaml_container.children()?.append(&explanation)?;

        let usb_list = winrt::factory::<ListBox, IListBoxFactory>()?.create_instance(Object::default(), &mut Object::default())?;
        RelativePanel::set_below(&usb_list, Object::from(explanation))?;
        xaml_container.children()?.append(&usb_list)?;

        let next_btn = winrt::factory::<Button, IButtonFactory>()?.create_instance(Object::default(), &mut Object::default())?;
        let next_s: Object = PropertyValue::create_string("Next")?.into();
        next_btn.set_content(next_s)?;
        next_btn.set_margin(Thickness {
            top: 0., left: 0., right: 10., bottom: 10.
        })?;
        {
            let el_proxy = el_proxy.clone();
            next_btn.click(RoutedEventHandler::new(move |_, _| {
                el_proxy.send_event(WizardEvent::GoToStep3).unwrap();
                Ok(())
            }))?;
        }
        next_btn.set_is_enabled(false)?;
        RelativePanel::set_align_bottom_with_panel(&next_btn, true)?;
        RelativePanel::set_align_right_with_panel(&next_btn, true)?;
        xaml_container.children()?.append(&next_btn)?;

        usb_list.selection_changed(SelectionChangedEventHandler::new(move |_, _| {
            next_btn.set_is_enabled(true)?;
            Ok(())
        }))?;

        // TODO: Move to using WMI MSFT_StorageEvent.
        // BODY: Using the WinRT APIs here is a bad idea. They're too
        // BODY: restrictive. Instead, what I should be doing is using WMI to
        // BODY: list the disks, partitions, and volumes. WMI has an event
        // BODY: system as well which would allow us to see live changes to the
        // BODY: disks, similar to what we have right now.

        let watcher = DeviceInformation::create_watcher_device_class(DeviceClass::PortableStorageDevice)?;
        {
            let el_proxy = el_proxy.clone();
            watcher.added(TypedEventHandler::new(move |_, info: &DeviceInformation| {
                if let Some(device) = DeviceNameId::new(info)? {
                    el_proxy.send_event(WizardEvent::UsbDeviceFound(device)).unwrap();
                }
                Ok(())
            }))?;
        }
        watcher.updated(TypedEventHandler::new(move |_, _| {
            Ok(())
        }))?;
        watcher.removed(TypedEventHandler::new(move |_, _| {
            Ok(())
        }))?;
        watcher.start()?;

        xaml_container.update_layout()?;

        Ok(WizardStep::Step2 {
            container: xaml_container,
            usb_list, _watcher: watcher
        })
    }

    pub fn step3(el_proxy: EventLoopProxy<WizardEvent>) -> winrt::Result<WizardStep> {
        let xaml_container = winrt::factory::<RelativePanel, IRelativePanelFactory>()?.create_instance(Object::default(), &mut Object::default())?;
        let grey_brush = SolidColorBrush::new()?;
        grey_brush.set_color(Color { r: 0x5e, g: 0x27, b: 0x50, a: 255})?;
        xaml_container.set_background(grey_brush)?;

        let title = make_tb("Downloading ISO")?;
        title.set_font_size(48.)?;
        RelativePanel::set_align_horizontal_center_with_panel(&title, true)?;
        title.set_margin(Thickness {
            top: 10., ..Thickness::default()
        })?;
        xaml_container.children()?.append(&title)?;

        let progress_bar = winrt::factory::<ProgressBar, IProgressBarFactory>()?.create_instance(Object::default(), &mut Object::default())?;
        RelativePanel::set_below(&progress_bar, Object::from(title))?;
        RelativePanel::set_align_left_with_panel(&progress_bar, true)?;
        RelativePanel::set_align_right_with_panel(&progress_bar, true)?;
        progress_bar.set_is_indeterminate(false)?;
        progress_bar.set_margin(Thickness {
            top: 10., left: 10., right: 10., ..Thickness::default()
        })?;
        let join_handle = {
            let el_proxy = el_proxy.clone();
            download_iso(move |cur_prog, total_bytes| {
                el_proxy.send_event(WizardEvent::SetProgress(cur_prog, total_bytes)).unwrap();
            }, move |_res| {
                // Whatever.
            })
        };

        xaml_container.children()?.append(&progress_bar)?;


        Ok(WizardStep::Step3 {
            container: xaml_container,
            progress_bar: progress_bar,
            _handle: join_handle,
        })
    }

    pub fn add_usb_device(&self, device: &DeviceNameId) -> winrt::Result<()> {
        if let WizardStep::Step2 { container, usb_list, .. } = self {
            usb_list.items()?.append(Object::from(make_tb(&format!("{} ({})", device.path, device.name))?))?;
            container.update_layout()?;
        }
        Ok(())
    }

    pub fn set_progress(&self, cur: u64, total: Option<u64>) -> winrt::Result<()> {
        if let WizardStep::Step3 { container, progress_bar, .. } = self {
            progress_bar.set_value(cur as f64)?;
            if let Some(v) = total {
                progress_bar.set_maximum(v as f64)?;
            }
            container.update_layout()?;
        }
        Ok(())
    }

    fn top_level(&self) -> UIElement {
        match self {
            WizardStep::Step1 { ref container } => container.into(),
            WizardStep::Step2 { ref container, .. } => container.into(),
            WizardStep::Step3 { ref container, .. } => container.into(),
        }
    }
}

#[derive(Debug)]
pub struct DeviceNameId {
    id: String,
    name: String,
    path: String,
}

impl DeviceNameId {
    fn new(info: &DeviceInformation) -> winrt::Result<Option<DeviceNameId>> {
        let id = info.id()?;
        let mut id_with_backslash = Vec::from(id.as_wide());
        id_with_backslash.extend(&*'\\'.encode_utf16(&mut [0; 2]));
        id_with_backslash.extend(&*'\0'.encode_utf16(&mut [0; 2]));
        println!("id: {:?}", id);
        // TODO: My own error.
        let file = std::fs::File::open(String::from_utf16_lossy(id.as_wide())).unwrap();
        unsafe {
            let volume_name = &mut [0; MAX_PATH + 1];
            to_win32_err(winapi::um::fileapi::GetVolumeInformationByHandleW(
                file.as_raw_handle(),
                volume_name.as_mut_ptr(),
                volume_name.len() as u32,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
            )).unwrap();
            let name = String::from_utf16_lossy(&volume_name[..volume_name.iter().position(|v| *v == 0).unwrap_or(volume_name.len())]);
            println!("Name: {}", name);

            to_win32_err(winapi::um::fileapi::GetVolumeNameForVolumeMountPointW(
                id_with_backslash.as_ptr(),
                volume_name.as_mut_ptr(),
                volume_name.len() as u32,
            )).unwrap();
            println!("GUID: {:?}", String::from_utf16_lossy(&volume_name[..volume_name.iter().position(|v| *v == 0).unwrap_or(volume_name.len())]));

            let names = &mut [0; MAX_PATH * 4];
            let mut char_count = 0;
            to_win32_err(GetVolumePathNamesForVolumeNameW(
                volume_name.as_ptr(),
                names.as_mut_ptr(),
                names.len() as u32,
                &mut char_count
            )).unwrap();

            let names = &names[..char_count as usize];
            while !names.is_empty() {
                let end_pos = names.iter().position(|v| *v == 0).unwrap_or(names.len());
                let drive_letter = String::from_utf16_lossy(&names[..end_pos]);
                if drive_letter != "" {
                    return Ok(Some(DeviceNameId {
                        id: id.to_string(), name, path: drive_letter
                    }));
                }
            }
            Ok(None)
        }
    }
}

fn to_win32_err_arg(err: u32) -> Result<u32, String> {
    if err == 0 {
        unsafe {
            let err = GetLastError();
            let mut buffer: LPWSTR = ptr::null_mut();
            let strlen = FormatMessageW(FORMAT_MESSAGE_FROM_SYSTEM |
                FORMAT_MESSAGE_ALLOCATE_BUFFER |
                FORMAT_MESSAGE_IGNORE_INSERTS,
                ptr::null(),
                err,
                0,
                (&mut buffer as *mut LPWSTR) as LPWSTR,
                0,
                ptr::null_mut());

            // Get the buffer as a wide string
            let msg = U16String::from_ptr(buffer, strlen as usize);

            // Since U16String creates an owned copy, it's safe to free original buffer now
            // If you didn't want an owned copy, you could use &U16Str.
            LocalFree(buffer as HLOCAL);

            Err(msg.to_string_lossy())
        }
    } else {
        Ok(err)
    }
}

// TODO: Move to WinRT BackgroundDownloader when built for UWP
fn download_iso<ProgCb, ComplCb>(mut progress_cb: ProgCb, mut complete_cb: ComplCb) -> JoinHandle<()>
where
    ProgCb: FnMut(u64, Option<u64>) + Send + 'static,
    ComplCb: FnMut(Result<PathBuf, ()>) + Send + 'static,
{
    // TODO: Find an URL through the RSS feed https://launchpad.net/ubuntu/+cdmirrors-rss
    const URL: &'static str = "https://mirrors.melbourne.co.uk/ubuntu-releases/20.04/ubuntu-20.04-desktop-amd64.iso";
    std::thread::spawn(move || {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut file = File::from_std(tempfile::tempfile().unwrap());
            let resp = reqwest::get(URL)
                .await
                .unwrap();

            if resp.status().is_success() {
                let content_len = resp.content_length();
                let mut current_len = 0;
                let mut resp = resp.bytes_stream();
                while let Some(val) = resp.next().await {
                    let val = val.unwrap();
                    file.write_all(&*val).await.unwrap();
                    current_len += val.len();
                    progress_cb(current_len as u64, content_len);
                }
                //complete_cb(Ok(file))
            } else {
                complete_cb(Err(()))
            }
        });
    })
}

fn to_win32_err(err: i32) -> Result<(), String> {
    to_win32_err_arg(err as u32).map(|_| ())
}

#[link(name = "user32")]
extern "stdcall" {
    fn UpdateWindow(
        hwnd: *mut core::ffi::c_void,
    ) -> i32;
}

#[derive(Debug)]
pub enum WizardEvent {
    GoToStep2,
    UsbDeviceFound(DeviceNameId),
    // TODO: Selected Device
    GoToStep3,
    SetProgress(u64, Option<u64>),
}