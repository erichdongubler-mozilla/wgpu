use crate::com::ComPtr;
#[cfg(any(feature = "libloading", feature = "implicit-link"))]
use winapi::Interface as _;
use winapi::{
    shared::{minwindef::TRUE, winerror::S_OK},
    um::d3d12sdklayers,
};

pub type Debug = ComPtr<d3d12sdklayers::ID3D12Debug>;

#[cfg(feature = "libloading")]
impl crate::D3D12Lib {
    pub fn get_debug_interface(&self) -> Result<crate::D3DResult<Debug>, libloading::Error> {
        type Fun = extern "system" fn(
            winapi::shared::guiddef::REFIID,
            *mut *mut winapi::ctypes::c_void,
        ) -> crate::HRESULT;

        let mut debug = std::ptr::null_mut();
        let hr = unsafe {
            let func: libloading::Symbol<Fun> = self.lib.get(b"D3D12GetDebugInterface")?;
            func(&d3d12sdklayers::ID3D12Debug::uuidof(), &mut debug)
        };
        let debug = unsafe { ComPtr::from_reffed(debug.cast()) };

        Ok((debug, hr))
    }
}

impl Debug {
    #[cfg(feature = "implicit-link")]
    pub fn get_interface() -> crate::D3DResult<Self> {
        let mut debug = std::ptr::null_mut();
        let hr = unsafe {
            winapi::um::d3d12::D3D12GetDebugInterface(
                &d3d12sdklayers::ID3D12Debug::uuidof(),
                &mut debug,
            )
        };
        let debug = unsafe { ComPtr::from_reffed(debug.cast()) };

        (debug, hr)
    }

    pub fn enable_layer(&self) {
        unsafe { self.EnableDebugLayer() }
    }

    pub fn enable_gpu_based_validation(&self) -> bool {
        let (ptr, hr) = unsafe { self.cast::<d3d12sdklayers::ID3D12Debug1>() };
        if hr == S_OK {
            unsafe { ptr.SetEnableGPUBasedValidation(TRUE) };
            true
        } else {
            false
        }
    }
}
