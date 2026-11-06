//! Slot²: the test surface on Linux. Mesa's EGL with no window (`EGL_MESA_platform_surfaceless`)
//! and a GLES2 context, as on the device; the compositor draws into its own framebuffers,
//! as it does under the macOS surface. It runs on WSL's software renderer, so the render
//! tests run on the machine Slot² is built on.

use std::ffi::{c_char, c_void, CString};

use libloading::Library;

use crate::fbdev::egl_error;
use crate::surface::{GfxError, Surface, OUT_H, OUT_W};

type Ptr = *mut c_void;

const EGL_PLATFORM_SURFACELESS_MESA: u32 = 0x31DD;
const EGL_NONE: i32 = 0x3038;
const EGL_RENDERABLE_TYPE: i32 = 0x3040;
const EGL_OPENGL_ES2_BIT: i32 = 0x0004;
const EGL_SURFACE_TYPE: i32 = 0x3033;
const EGL_PBUFFER_BIT: i32 = 0x0001;
const EGL_RED_SIZE: i32 = 0x3024;
const EGL_GREEN_SIZE: i32 = 0x3023;
const EGL_BLUE_SIZE: i32 = 0x3022;
const EGL_ALPHA_SIZE: i32 = 0x3021;
const EGL_CONTEXT_CLIENT_VERSION: i32 = 0x3098;
const EGL_OPENGL_ES_API: u32 = 0x30A0;

type GetPlatformDisplay = unsafe extern "C" fn(u32, Ptr, *const isize) -> Ptr;
type Initialize = unsafe extern "C" fn(Ptr, *mut i32, *mut i32) -> u32;
type BindApi = unsafe extern "C" fn(u32) -> u32;
type ChooseConfig = unsafe extern "C" fn(Ptr, *const i32, *mut Ptr, i32, *mut i32) -> u32;
type CreateContext = unsafe extern "C" fn(Ptr, Ptr, Ptr, *const i32) -> Ptr;
type DestroyContext = unsafe extern "C" fn(Ptr, Ptr) -> u32;
type MakeCurrent = unsafe extern "C" fn(Ptr, Ptr, Ptr, Ptr) -> u32;
type GetProcAddress = unsafe extern "C" fn(*const c_char) -> *const c_void;
type GetError = unsafe extern "C" fn() -> i32;
type Terminate = unsafe extern "C" fn(Ptr) -> u32;

struct Egl {
    make_current: MakeCurrent,
    destroy_context: DestroyContext,
    get_proc_address: GetProcAddress,
    get_error: GetError,
    terminate: Terminate,
    _lib: Library,
}

pub struct HeadlessSurface {
    egl: Egl,
    display: Ptr,
    context: Ptr,
}

unsafe fn sym<T: Copy>(lib: &Library, name: &str) -> Result<T, GfxError> {
    lib.get::<T>(name.as_bytes())
        .map(|s| *s)
        .map_err(|e| GfxError::Context(format!("{name}: {e}")))
}

impl HeadlessSurface {
    pub fn new() -> Result<Self, GfxError> {
        let lib = unsafe { Library::new("libEGL.so.1") }
            .map_err(|e| GfxError::Context(format!("libEGL: {e}")))?;
        unsafe {
            let get_proc_address: GetProcAddress = sym(&lib, "eglGetProcAddress")?;
            let get_error: GetError = sym(&lib, "eglGetError")?;
            let fail = |what: &str| egl_error(what, get_error());
            let platform_display = get_proc_address(c"eglGetPlatformDisplayEXT".as_ptr());
            if platform_display.is_null() {
                return Err(GfxError::Context("no eglGetPlatformDisplayEXT".into()));
            }
            let platform_display: GetPlatformDisplay = std::mem::transmute(platform_display);
            let display = platform_display(
                EGL_PLATFORM_SURFACELESS_MESA,
                std::ptr::null_mut(),
                std::ptr::null(),
            );
            if display.is_null() {
                return Err(fail("eglGetPlatformDisplayEXT"));
            }
            let initialize: Initialize = sym(&lib, "eglInitialize")?;
            let (mut major, mut minor) = (0, 0);
            if initialize(display, &mut major, &mut minor) == 0 {
                return Err(fail("eglInitialize"));
            }
            let bind_api: BindApi = sym(&lib, "eglBindAPI")?;
            if bind_api(EGL_OPENGL_ES_API) == 0 {
                return Err(fail("eglBindAPI"));
            }
            let choose_config: ChooseConfig = sym(&lib, "eglChooseConfig")?;
            let attrs = [
                EGL_SURFACE_TYPE,
                EGL_PBUFFER_BIT,
                EGL_RENDERABLE_TYPE,
                EGL_OPENGL_ES2_BIT,
                EGL_RED_SIZE,
                8,
                EGL_GREEN_SIZE,
                8,
                EGL_BLUE_SIZE,
                8,
                EGL_ALPHA_SIZE,
                8,
                EGL_NONE,
            ];
            let (mut config, mut count): (Ptr, i32) = (std::ptr::null_mut(), 0);
            if choose_config(display, attrs.as_ptr(), &mut config, 1, &mut count) == 0
                || count == 0
            {
                return Err(GfxError::Context("no GLES2 config".into()));
            }
            let create_context: CreateContext = sym(&lib, "eglCreateContext")?;
            let context_attrs = [EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE];
            let context = create_context(
                display,
                config,
                std::ptr::null_mut(),
                context_attrs.as_ptr(),
            );
            if context.is_null() {
                return Err(fail("eglCreateContext"));
            }
            let egl = Egl {
                make_current: sym(&lib, "eglMakeCurrent")?,
                destroy_context: sym(&lib, "eglDestroyContext")?,
                get_proc_address,
                get_error,
                terminate: sym(&lib, "eglTerminate")?,
                _lib: lib,
            };
            let mut surface = HeadlessSurface {
                egl,
                display,
                context,
            };
            surface.make_current()?;
            Ok(surface)
        }
    }
}

impl Surface for HeadlessSurface {
    fn make_current(&mut self) -> Result<(), GfxError> {
        let none = std::ptr::null_mut();
        match unsafe { (self.egl.make_current)(self.display, none, none, self.context) } {
            0 => Err(egl_error("eglMakeCurrent", unsafe {
                (self.egl.get_error)()
            })),
            _ => Ok(()),
        }
    }

    fn window_size(&self) -> (u32, u32) {
        (OUT_W, OUT_H)
    }

    fn swap(&mut self) -> Result<(), GfxError> {
        unsafe { gl::Flush() };
        Ok(())
    }

    fn proc_address(&self, name: &str) -> *const c_void {
        let Ok(name) = CString::new(name) else {
            return std::ptr::null();
        };
        unsafe { (self.egl.get_proc_address)(name.as_ptr()) }
    }
}

impl Drop for HeadlessSurface {
    fn drop(&mut self) {
        let none = std::ptr::null_mut();
        unsafe {
            (self.egl.make_current)(self.display, none, none, none);
            (self.egl.destroy_context)(self.display, self.context);
            (self.egl.terminate)(self.display);
        }
    }
}
