use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_float, c_int, c_uchar, c_uint, c_ulong, c_ushort};

use crate::error::{check, LibRawError};

// --- FFI declarations for libraw C API ---

#[repr(C)]
pub struct LibRawDataT {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct LibRawProcessedImageT {
    _opaque: [u8; 0],
}

extern "C" {
    // lifecycle
    fn libraw_init(flags: c_uint) -> *mut LibRawDataT;
    fn libraw_close(lr: *mut LibRawDataT);
    fn libraw_recycle(lr: *mut LibRawDataT);

    // I/O
    fn libraw_open_file(lr: *mut LibRawDataT, path: *const c_char) -> c_int;

    // processing
    fn libraw_unpack(lr: *mut LibRawDataT) -> c_int;
    fn libraw_dcraw_process(lr: *mut LibRawDataT) -> c_int;
    fn libraw_dcraw_make_mem_image(
        lr: *mut LibRawDataT,
        errc: *mut c_int,
    ) -> *mut LibRawProcessedImageT;
    fn libraw_dcraw_clear_mem(img: *mut LibRawProcessedImageT);

    // --- wrapper.c shims: param setters ---
    fn lrnif_set_output_bps(lr: *mut LibRawDataT, bps: c_int);
    fn lrnif_set_use_camera_wb(lr: *mut LibRawDataT, v: c_int);
    fn lrnif_set_no_auto_bright(lr: *mut LibRawDataT, v: c_int);
    fn lrnif_set_gamm(lr: *mut LibRawDataT, g0: c_double, g1: c_double);

    // --- wrapper.c shims: metadata getters ---
    fn lrnif_get_make(lr: *mut LibRawDataT) -> *const c_char;
    fn lrnif_get_model(lr: *mut LibRawDataT) -> *const c_char;
    fn lrnif_get_timestamp(lr: *mut LibRawDataT) -> c_ulong;
    fn lrnif_get_iso(lr: *mut LibRawDataT) -> c_float;
    fn lrnif_get_shutter(lr: *mut LibRawDataT) -> c_float;
    fn lrnif_get_aperture(lr: *mut LibRawDataT) -> c_float;
    fn lrnif_get_flip(lr: *mut LibRawDataT) -> c_int;

    // --- wrapper.c shims: processed image accessors ---
    fn lrnif_image_height(img: *mut LibRawProcessedImageT) -> c_ushort;
    fn lrnif_image_width(img: *mut LibRawProcessedImageT) -> c_ushort;
    fn lrnif_image_colors(img: *mut LibRawProcessedImageT) -> c_ushort;
    fn lrnif_image_bits(img: *mut LibRawProcessedImageT) -> c_ushort;
    fn lrnif_image_data_size(img: *mut LibRawProcessedImageT) -> c_uint;
    fn lrnif_image_data(img: *mut LibRawProcessedImageT) -> *mut c_uchar;
}

/// RAII guard that calls `libraw_recycle` + `libraw_close` on drop.
pub struct RawHandle {
    ptr: *mut LibRawDataT,
}

// libraw_data_t owns no BEAM-managed resources and is used only from a single
// dirty-CPU scheduler thread per NIF call, so Send is safe here.
unsafe impl Send for RawHandle {}

impl RawHandle {
    /// Allocate a new libraw handle.
    pub fn new() -> Result<Self, LibRawError> {
        let ptr = unsafe { libraw_init(0) };
        if ptr.is_null() {
            Err(LibRawError::NullPointer)
        } else {
            Ok(RawHandle { ptr })
        }
    }

    /// Open a file by path.
    pub fn open(&self, path: &str) -> Result<(), LibRawError> {
        let cpath = CString::new(path).map_err(|_| LibRawError::InvalidPath)?;
        check(unsafe { libraw_open_file(self.ptr, cpath.as_ptr()) })
    }

    /// Apply decode parameters.
    pub fn set_params(&self, use_camera_wb: i32, no_auto_bright: i32, output_bps: i32, g0: f64, g1: f64) {
        unsafe {
            lrnif_set_use_camera_wb(self.ptr, use_camera_wb);
            lrnif_set_no_auto_bright(self.ptr, no_auto_bright);
            lrnif_set_output_bps(self.ptr, output_bps);
            lrnif_set_gamm(self.ptr, g0, g1);
        }
    }

    /// Unpack the raw data from the file.
    pub fn unpack(&self) -> Result<(), LibRawError> {
        check(unsafe { libraw_unpack(self.ptr) })
    }

    /// Run dcraw processing pipeline.
    pub fn dcraw_process(&self) -> Result<(), LibRawError> {
        check(unsafe { libraw_dcraw_process(self.ptr) })
    }

    /// Convert to an in-memory bitmap and return a `ProcessedImage` guard.
    pub fn make_mem_image(&self) -> Result<ProcessedImage, LibRawError> {
        let mut errc: c_int = 0;
        let img = unsafe { libraw_dcraw_make_mem_image(self.ptr, &mut errc) };
        if img.is_null() {
            Err(if errc != 0 {
                LibRawError::from_code(errc)
            } else {
                LibRawError::NullPointer
            })
        } else {
            Ok(ProcessedImage { ptr: img })
        }
    }

    // --- metadata accessors (safe wrappers around the C shims) ---

    pub fn make(&self) -> String {
        unsafe { cstr_to_string(lrnif_get_make(self.ptr)) }
    }

    pub fn model(&self) -> String {
        unsafe { cstr_to_string(lrnif_get_model(self.ptr)) }
    }

    /// Unix timestamp as reported by libraw. 0 means "not present".
    pub fn timestamp(&self) -> u64 {
        unsafe { lrnif_get_timestamp(self.ptr) as u64 }
    }

    pub fn iso(&self) -> f64 {
        unsafe { lrnif_get_iso(self.ptr) as f64 }
    }

    pub fn shutter(&self) -> f64 {
        unsafe { lrnif_get_shutter(self.ptr) as f64 }
    }

    pub fn aperture(&self) -> f64 {
        unsafe { lrnif_get_aperture(self.ptr) as f64 }
    }

    pub fn flip(&self) -> i32 {
        unsafe { lrnif_get_flip(self.ptr) as i32 }
    }
}

impl Drop for RawHandle {
    fn drop(&mut self) {
        unsafe {
            libraw_recycle(self.ptr);
            libraw_close(self.ptr);
        }
    }
}

/// RAII guard for a `libraw_processed_image_t` allocated by libraw.
pub struct ProcessedImage {
    ptr: *mut LibRawProcessedImageT,
}

impl ProcessedImage {
    pub fn height(&self) -> u32 {
        unsafe { lrnif_image_height(self.ptr) as u32 }
    }

    pub fn width(&self) -> u32 {
        unsafe { lrnif_image_width(self.ptr) as u32 }
    }

    pub fn colors(&self) -> u32 {
        unsafe { lrnif_image_colors(self.ptr) as u32 }
    }

    pub fn bits(&self) -> u32 {
        unsafe { lrnif_image_bits(self.ptr) as u32 }
    }

    /// Copy the pixel data into an owned `Vec<u8>`.
    pub fn pixel_bytes(&self) -> Vec<u8> {
        let size = unsafe { lrnif_image_data_size(self.ptr) } as usize;
        let data_ptr = unsafe { lrnif_image_data(self.ptr) };
        unsafe { std::slice::from_raw_parts(data_ptr, size).to_vec() }
    }
}

impl Drop for ProcessedImage {
    fn drop(&mut self) {
        unsafe { libraw_dcraw_clear_mem(self.ptr) }
    }
}

// --- helpers ---

unsafe fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr)
        .to_str()
        .unwrap_or("")
        .trim_end_matches('\0')
        .to_owned()
}
