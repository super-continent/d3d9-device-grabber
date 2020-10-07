use std::error;
use std::mem;
use std::ptr;

use winapi::shared::{d3d9::*, d3d9types::*, minwindef::*, windef::*};
use winapi::um::{processthreadsapi::GetCurrentProcessId, winuser::*};

use thiserror::Error;

/// Get the D3D9 device pointer
///
/// Example usage:
///
/// ```
/// // This would be part of the lib.rs inside of a library that is of the type `cdylib`
/// // init() would be called from a new thread spawned in DllMain
///
/// unsafe fn init() {
///    AllocConsole();
///    let device_result = get_d3d9_device();
///
///    if let Err(e) = device_result {
///        println!("error getting d3d9 device: {}", e);
///        return
///    }
///
///    let device = device_result.unwrap();
///    // do whatever you want with device
/// }
/// ```
pub unsafe fn get_d3d9_device() -> Result<&'static mut IDirect3DDevice9, Box<dyn error::Error>> {
    let window = match get_process_window() {
        Some(hwnd) => hwnd,
        None => return Err(Box::new(D3D9GrabError::GetProcessWindowFailed)),
    };

    let d3d9 = Direct3DCreate9(D3D_SDK_VERSION);

    if d3d9.is_null() {
        return Err(Box::new(D3D9GrabError::D3DCreate9Null));
    }

    let mut present_params = D3DPRESENT_PARAMETERS {
        BackBufferWidth: 0,
        BackBufferHeight: 0,
        BackBufferFormat: 0,
        BackBufferCount: 0,
        MultiSampleType: 0,
        MultiSampleQuality: 0,
        SwapEffect: D3DSWAPEFFECT_DISCARD,
        hDeviceWindow: window,
        Windowed: FALSE,
        EnableAutoDepthStencil: 0,
        AutoDepthStencilFormat: 0,
        Flags: 0,
        FullScreen_RefreshRateInHz: 0,
        PresentationInterval: 0,
    };

    let d3d9_device: *mut IDirect3DDevice9 = ptr::null_mut();

    let result_device_err = (*d3d9).CreateDevice(
        D3DADAPTER_DEFAULT,
        D3DDEVTYPE_HAL,
        present_params.hDeviceWindow,
        D3DCREATE_SOFTWARE_VERTEXPROCESSING,
        &mut present_params,
        mem::transmute(&d3d9_device),
    );

    if result_device_err != 0 {
        present_params.Windowed = !present_params.Windowed;
        let result_device_err = (*d3d9).CreateDevice(
            D3DADAPTER_DEFAULT,
            D3DDEVTYPE_HAL,
            present_params.hDeviceWindow,
            D3DCREATE_SOFTWARE_VERTEXPROCESSING,
            mem::transmute(&present_params),
            mem::transmute(&d3d9_device),
        );
        if result_device_err != 0 {
            return Err(Box::new(D3D9GrabError::CreateDeviceError(
                result_device_err,
            )));
        }
    }

    match d3d9_device.as_mut() {
        None => return Err(Box::new(D3D9GrabError::AsMutError)),
        Some(device_ref) => return Ok(device_ref),
    }
}

unsafe fn get_process_window() -> Option<HWND> {
    extern "system" fn enum_windows_callback(hwnd: HWND, l_param: LPARAM) -> BOOL {
        let mut wnd_proc_id: DWORD = 0;
        unsafe {
            GetWindowThreadProcessId(hwnd, &mut wnd_proc_id as *mut DWORD);
            if GetCurrentProcessId() != wnd_proc_id {
                return TRUE;
            }
            *(l_param as *mut HWND) = hwnd;
        }
        return FALSE;
    }

    let mut hwnd: HWND = ptr::null_mut();
    EnumWindows(
        Some(enum_windows_callback),
        &mut hwnd as *mut HWND as LPARAM,
    );
    return if hwnd.is_null() { None } else { Some(hwnd) };
}

#[derive(Debug, Error)]
pub enum D3D9GrabError {
    #[error("d3d9_device.as_mut() failed to return an instance of &mut IDirect3D9Device")]
    AsMutError,
    #[error("d3d9.CreateDevice call returned with an error code `{0:#X}`")]
    CreateDeviceError(i32),
    #[error("D3DCreate9 call returned null")]
    D3DCreate9Null,
    #[error("Could not get current process window handle")]
    GetProcessWindowFailed,
}
