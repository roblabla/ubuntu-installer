use raw_window_handle::HasRawWindowHandle;
use core::ffi::c_void;
use core::ptr;
use bindings::windows::ui::xaml::hosting::DesktopWindowXamlSource;

#[repr(transparent)]
#[derive(PartialEq)]
pub struct IDesktopWindowXamlSourceNative {
    ptr: ::winrt::ComPtr<IDesktopWindowXamlSourceNative>,
}

impl IDesktopWindowXamlSourceNative {
    pub fn attach_to_window<T: HasRawWindowHandle>(&self, hnd: &T) -> winrt::Result<()> {
        let this = <::winrt::ComPtr<Self> as ::winrt::ComInterface>::as_raw(&self.ptr);
        if this.is_null() {
            panic!("The `this` pointer was null when calling method");
        }

        let hwnd = match hnd.raw_window_handle() {
            raw_window_handle::RawWindowHandle::Windows(window_handle) => window_handle.hwnd,
            _ => panic!("Unsupported platform!"),
        };

        unsafe {
            ((*(*(this))).attach_to_window)(
                this,
                hwnd
            ).ok()
        }
    }

    pub fn get_window_handle(&self) -> winrt::Result<*mut c_void> {
        let this = <::winrt::ComPtr<Self> as ::winrt::ComInterface>::as_raw(&self.ptr);
        if this.is_null() {
            panic!("The `this` pointer was null when calling method");
        }

        let mut hwnd = ptr::null_mut();

        unsafe {
            ((*(*(this))).get_window_handle)(
                this,
                &mut hwnd,
            )
            .and_then(|| hwnd)
        }
    }
}

unsafe impl ::winrt::ComInterface for IDesktopWindowXamlSourceNative {
    type VTable = abi_IDesktopWindowXamlSourceNative;
    fn iid() -> ::winrt::Guid {
        ::winrt::Guid::from_values(0x3cbcf1bf, 0x2f76, 0x4e9c, [0x96, 0xab, 0xe8, 0x4b, 0x37, 0x97, 0x25, 0x54])
    }
}

impl std::clone::Clone for IDesktopWindowXamlSourceNative {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr.clone()
        }
    }
}

#[repr(C)]
pub struct abi_IDesktopWindowXamlSourceNative {
    pub unknown_query_interface: extern "system" fn(::winrt::RawComPtr<::winrt::IUnknown>, &::winrt::Guid, *mut ::winrt::RawPtr) -> ::winrt::ErrorCode,
    pub unknown_add_ref: extern "system" fn(::winrt::RawComPtr<::winrt::IUnknown>) -> u32,
    pub unknown_release: extern "system" fn(::winrt::RawComPtr<::winrt::IUnknown>) -> u32,
    pub attach_to_window: extern "system" fn(*const *const abi_IDesktopWindowXamlSourceNative, *mut c_void) -> ::winrt::ErrorCode,
    pub get_window_handle: extern "system" fn(*const *const abi_IDesktopWindowXamlSourceNative, *mut *mut c_void) -> ::winrt::ErrorCode,
}

impl From<&DesktopWindowXamlSource> for IDesktopWindowXamlSourceNative {
    fn from(value: &DesktopWindowXamlSource) -> IDesktopWindowXamlSourceNative {
        <DesktopWindowXamlSource as ::winrt::ComInterface>::query(value)
    }
}

impl From<DesktopWindowXamlSource> for IDesktopWindowXamlSourceNative {
    fn from(value: DesktopWindowXamlSource) -> IDesktopWindowXamlSourceNative {
        std::convert::From::from(&value)
    }
}