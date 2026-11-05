use std::ffi::{c_char, c_int, c_uint, c_void, CStr, CString};

use libloading::Library;

use crate::fbdev::egl_error;
use crate::surface::{GfxError, Surface};

const CARD: &str = "/dev/dri/card0";

const DRM_MODE_CONNECTED: c_int = 1;
const DRM_MODE_TYPE_PREFERRED: u32 = 1 << 3;
const DRM_MODE_PAGE_FLIP_EVENT: u32 = 1;

pub const CONNECTOR_LVDS: u32 = 7;
pub const CONNECTOR_EDP: u32 = 14;
pub const CONNECTOR_DSI: u32 = 16;
pub const CONNECTOR_DPI: u32 = 17;

const GBM_FORMAT_XRGB8888: u32 = 0x3432_5258;
const GBM_BO_USE_SCANOUT: u32 = 1;
const GBM_BO_USE_RENDERING: u32 = 1 << 2;

const EGL_NONE: i32 = 0x3038;
const EGL_RED_SIZE: i32 = 0x3024;
const EGL_GREEN_SIZE: i32 = 0x3023;
const EGL_BLUE_SIZE: i32 = 0x3022;
const EGL_ALPHA_SIZE: i32 = 0x3021;
const EGL_SURFACE_TYPE: i32 = 0x3033;
const EGL_RENDERABLE_TYPE: i32 = 0x3040;
const EGL_NATIVE_VISUAL_ID: i32 = 0x302E;
const EGL_CONTEXT_CLIENT_VERSION: i32 = 0x3098;
const EGL_EXTENSIONS: i32 = 0x3055;
const EGL_OPENGL_ES_API: u32 = 0x30A0;
const EGL_WINDOW_BIT: i32 = 0x0004;
const EGL_OPENGL_ES2_BIT: i32 = 0x0004;
const EGL_PLATFORM_GBM_KHR: u32 = 0x31D7;

const FLIP_TIMEOUT_MS: c_int = 100;

const PACE_MARGIN: std::time::Duration = std::time::Duration::from_micros(4000);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ModeInfo {
    pub clock: u32,
    pub hdisplay: u16,
    pub hsync_start: u16,
    pub hsync_end: u16,
    pub htotal: u16,
    pub hskew: u16,
    pub vdisplay: u16,
    pub vsync_start: u16,
    pub vsync_end: u16,
    pub vtotal: u16,
    pub vscan: u16,
    pub vrefresh: u32,
    pub flags: u32,
    pub kind: u32,
    pub name: [c_char; 32],
}

impl ModeInfo {
    pub fn blank() -> Self {
        // SAFETY: every field is a plain integer, so all-zero bytes are a valid value.
        unsafe { std::mem::zeroed() }
    }

    pub fn preferred(&self) -> bool {
        self.kind & DRM_MODE_TYPE_PREFERRED != 0
    }

    pub fn refresh_hz(&self) -> f64 {
        let dots = f64::from(self.htotal) * f64::from(self.vtotal);
        match dots > 0.0 {
            true => f64::from(self.clock) * 1000.0 / dots,
            false => f64::from(self.vrefresh),
        }
    }
}

#[repr(C)]
pub struct DrmRes {
    pub count_fbs: c_int,
    pub fbs: *mut u32,
    pub count_crtcs: c_int,
    pub crtcs: *mut u32,
    pub count_connectors: c_int,
    pub connectors: *mut u32,
    pub count_encoders: c_int,
    pub encoders: *mut u32,
    pub min_width: u32,
    pub max_width: u32,
    pub min_height: u32,
    pub max_height: u32,
}

#[repr(C)]
pub struct DrmConnector {
    pub connector_id: u32,
    pub encoder_id: u32,
    pub connector_type: u32,
    pub connector_type_id: u32,
    pub connection: c_int,
    pub mm_width: u32,
    pub mm_height: u32,
    pub subpixel: c_int,
    pub count_modes: c_int,
    pub modes: *mut ModeInfo,
    pub count_props: c_int,
    pub props: *mut u32,
    pub prop_values: *mut u64,
    pub count_encoders: c_int,
    pub encoders: *mut u32,
}

#[repr(C)]
pub struct DrmEncoder {
    pub encoder_id: u32,
    pub encoder_type: u32,
    pub crtc_id: u32,
    pub possible_crtcs: u32,
    pub possible_clones: u32,
}

#[repr(C)]
pub struct DrmCrtc {
    pub crtc_id: u32,
    pub buffer_id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub mode_valid: c_int,
    pub mode: ModeInfo,
    pub gamma_size: c_int,
}

type FlipHandler = unsafe extern "C" fn(c_int, c_uint, c_uint, c_uint, *mut c_void);

#[repr(C)]
pub struct EventContext {
    pub version: c_int,
    pub vblank_handler: Option<FlipHandler>,
    pub page_flip_handler: Option<FlipHandler>,
    pub page_flip_handler2: Option<unsafe extern "C" fn()>,
    pub sequence_handler: Option<unsafe extern "C" fn()>,
}

#[derive(Clone)]
pub struct Connector {
    pub id: u32,
    pub kind: u32,
    pub connected: bool,
    pub modes: Vec<ModeInfo>,
}

pub fn internal(kind: u32) -> bool {
    matches!(
        kind,
        CONNECTOR_DSI | CONNECTOR_EDP | CONNECTOR_LVDS | CONNECTOR_DPI
    )
}

pub fn pick_connector(connectors: &[Connector]) -> Option<usize> {
    let usable = |c: &&Connector| c.connected && !c.modes.is_empty();
    let first_internal = connectors
        .iter()
        .position(|c| usable(&c) && internal(c.kind));
    first_internal.or_else(|| connectors.iter().position(|c| usable(&c)))
}

pub fn pick_mode(modes: &[ModeInfo]) -> Option<usize> {
    modes
        .iter()
        .position(ModeInfo::preferred)
        .or((!modes.is_empty()).then_some(0))
}

pub fn crtc_for(possible_crtcs: u32, crtcs: &[u32]) -> Option<u32> {
    crtcs
        .iter()
        .enumerate()
        .find(|(i, _)| *i < 32 && possible_crtcs & (1 << i) != 0)
        .map(|(_, id)| *id)
}

type Ptr = *mut c_void;

struct Drm {
    get_resources: unsafe extern "C" fn(c_int) -> *mut DrmRes,
    free_resources: unsafe extern "C" fn(*mut DrmRes),
    get_connector: unsafe extern "C" fn(c_int, u32) -> *mut DrmConnector,
    free_connector: unsafe extern "C" fn(*mut DrmConnector),
    get_encoder: unsafe extern "C" fn(c_int, u32) -> *mut DrmEncoder,
    free_encoder: unsafe extern "C" fn(*mut DrmEncoder),
    get_crtc: unsafe extern "C" fn(c_int, u32) -> *mut DrmCrtc,
    free_crtc: unsafe extern "C" fn(*mut DrmCrtc),
    set_crtc: unsafe extern "C" fn(
        c_int,
        u32,
        u32,
        u32,
        u32,
        *const u32,
        c_int,
        *const ModeInfo,
    ) -> c_int,
    add_fb: unsafe extern "C" fn(c_int, u32, u32, u8, u8, u32, u32, *mut u32) -> c_int,
    rm_fb: unsafe extern "C" fn(c_int, u32) -> c_int,
    page_flip: unsafe extern "C" fn(c_int, u32, u32, u32, Ptr) -> c_int,
    handle_event: unsafe extern "C" fn(c_int, *mut EventContext) -> c_int,
    _lib: Library,
}

struct Gbm {
    create_device: unsafe extern "C" fn(c_int) -> Ptr,
    device_destroy: unsafe extern "C" fn(Ptr),
    surface_create: unsafe extern "C" fn(Ptr, u32, u32, u32, u32) -> Ptr,
    surface_destroy: unsafe extern "C" fn(Ptr),
    lock_front_buffer: unsafe extern "C" fn(Ptr) -> Ptr,
    release_buffer: unsafe extern "C" fn(Ptr, Ptr),
    bo_get_handle: unsafe extern "C" fn(Ptr) -> u64,
    bo_get_stride: unsafe extern "C" fn(Ptr) -> u32,
    _lib: Library,
}

type GetProcAddress = unsafe extern "C" fn(*const c_char) -> *const c_void;
type GetPlatformDisplay = unsafe extern "C" fn(u32, Ptr, *const i32) -> Ptr;

struct Egl {
    get_display: unsafe extern "C" fn(Ptr) -> Ptr,
    initialize: unsafe extern "C" fn(Ptr, *mut i32, *mut i32) -> u32,
    bind_api: unsafe extern "C" fn(u32) -> u32,
    choose_config: unsafe extern "C" fn(Ptr, *const i32, *mut Ptr, i32, *mut i32) -> u32,
    get_config_attrib: unsafe extern "C" fn(Ptr, Ptr, i32, *mut i32) -> u32,
    create_window_surface: unsafe extern "C" fn(Ptr, Ptr, Ptr, *const i32) -> Ptr,
    destroy_surface: unsafe extern "C" fn(Ptr, Ptr) -> u32,
    create_context: unsafe extern "C" fn(Ptr, Ptr, Ptr, *const i32) -> Ptr,
    destroy_context: unsafe extern "C" fn(Ptr, Ptr) -> u32,
    make_current: unsafe extern "C" fn(Ptr, Ptr, Ptr, Ptr) -> u32,
    swap_buffers: unsafe extern "C" fn(Ptr, Ptr) -> u32,
    query_string: unsafe extern "C" fn(Ptr, i32) -> *const c_char,
    get_proc_address: GetProcAddress,
    get_error: unsafe extern "C" fn() -> i32,
    terminate: unsafe extern "C" fn(Ptr) -> u32,
    _lib: Library,
}

fn open(names: &[&str]) -> Result<Library, GfxError> {
    let mut last = String::new();
    for name in names {
        match unsafe { Library::new(name) } {
            Ok(lib) => return Ok(lib),
            Err(e) => last = format!("{name}: {e}"),
        }
    }
    Err(GfxError::Context(last))
}

unsafe fn sym<T: Copy>(lib: &Library, name: &str) -> Result<T, GfxError> {
    lib.get::<T>(name.as_bytes())
        .map(|s| *s)
        .map_err(|e| GfxError::Context(format!("{name}: {e}")))
}

impl Drm {
    fn load() -> Result<Self, GfxError> {
        let lib = open(&["libdrm.so.2", "libdrm.so"])?;
        unsafe {
            Ok(Drm {
                get_resources: sym(&lib, "drmModeGetResources")?,
                free_resources: sym(&lib, "drmModeFreeResources")?,
                get_connector: sym(&lib, "drmModeGetConnector")?,
                free_connector: sym(&lib, "drmModeFreeConnector")?,
                get_encoder: sym(&lib, "drmModeGetEncoder")?,
                free_encoder: sym(&lib, "drmModeFreeEncoder")?,
                get_crtc: sym(&lib, "drmModeGetCrtc")?,
                free_crtc: sym(&lib, "drmModeFreeCrtc")?,
                set_crtc: sym(&lib, "drmModeSetCrtc")?,
                add_fb: sym(&lib, "drmModeAddFB")?,
                rm_fb: sym(&lib, "drmModeRmFB")?,
                page_flip: sym(&lib, "drmModePageFlip")?,
                handle_event: sym(&lib, "drmHandleEvent")?,
                _lib: lib,
            })
        }
    }
}

impl Gbm {
    fn load() -> Result<Self, GfxError> {
        let lib = open(&["libgbm.so.1", "libgbm.so"])?;
        unsafe {
            Ok(Gbm {
                create_device: sym(&lib, "gbm_create_device")?,
                device_destroy: sym(&lib, "gbm_device_destroy")?,
                surface_create: sym(&lib, "gbm_surface_create")?,
                surface_destroy: sym(&lib, "gbm_surface_destroy")?,
                lock_front_buffer: sym(&lib, "gbm_surface_lock_front_buffer")?,
                release_buffer: sym(&lib, "gbm_surface_release_buffer")?,
                bo_get_handle: sym(&lib, "gbm_bo_get_handle")?,
                bo_get_stride: sym(&lib, "gbm_bo_get_stride")?,
                _lib: lib,
            })
        }
    }
}

impl Egl {
    fn load() -> Result<Self, GfxError> {
        let lib = open(&["libEGL.so.1", "libEGL.so"])?;
        unsafe {
            Ok(Egl {
                get_display: sym(&lib, "eglGetDisplay")?,
                initialize: sym(&lib, "eglInitialize")?,
                bind_api: sym(&lib, "eglBindAPI")?,
                choose_config: sym(&lib, "eglChooseConfig")?,
                get_config_attrib: sym(&lib, "eglGetConfigAttrib")?,
                create_window_surface: sym(&lib, "eglCreateWindowSurface")?,
                destroy_surface: sym(&lib, "eglDestroySurface")?,
                create_context: sym(&lib, "eglCreateContext")?,
                destroy_context: sym(&lib, "eglDestroyContext")?,
                make_current: sym(&lib, "eglMakeCurrent")?,
                swap_buffers: sym(&lib, "eglSwapBuffers")?,
                query_string: sym(&lib, "eglQueryString")?,
                get_proc_address: sym(&lib, "eglGetProcAddress")?,
                get_error: sym(&lib, "eglGetError")?,
                terminate: sym(&lib, "eglTerminate")?,
                _lib: lib,
            })
        }
    }

    fn fail(&self, what: &str) -> GfxError {
        egl_error(what, unsafe { (self.get_error)() })
    }

    fn client_extensions(&self) -> String {
        let s = unsafe { (self.query_string)(std::ptr::null_mut(), EGL_EXTENSIONS) };
        match s.is_null() {
            true => String::new(),
            false => unsafe { CStr::from_ptr(s) }.to_string_lossy().into_owned(),
        }
    }

    unsafe fn display_for(&self, gbm_device: Ptr) -> Ptr {
        let ext = self.client_extensions();
        let gbm_platform = ext
            .split_whitespace()
            .any(|e| e == "EGL_KHR_platform_gbm" || e == "EGL_MESA_platform_gbm");
        if gbm_platform {
            let name = c"eglGetPlatformDisplayEXT";
            let f = (self.get_proc_address)(name.as_ptr());
            if !f.is_null() {
                let get: GetPlatformDisplay = std::mem::transmute(f);
                let display = get(EGL_PLATFORM_GBM_KHR, gbm_device, std::ptr::null());
                if !display.is_null() {
                    return display;
                }
            }
        }
        (self.get_display)(gbm_device)
    }
}

pub fn libraries_resolve() -> Result<(), GfxError> {
    Drm::load()?;
    Gbm::load()?;
    Egl::load()?;
    open(&["libGLESv2.so.2", "libGLESv2.so"])?;
    Ok(())
}

const STATS_EVERY: u32 = 600;

#[derive(Default)]
pub struct FlipStats {
    refresh_us: u64,
    flips: u32,
    skipped: u32,
    late: u32,
    swap_sum_us: u64,
    swap_max_us: u64,
    lead_sum_us: u64,
    lead_max_us: u64,
    last_vblank_us: Option<u64>,
}

impl FlipStats {
    pub fn new(refresh_hz: f64) -> Self {
        FlipStats {
            refresh_us: (1_000_000.0 / refresh_hz.max(1.0)) as u64,
            ..FlipStats::default()
        }
    }

    pub fn record(&mut self, swap_us: u64, submit_us: u64, vblank_us: u64) -> Option<String> {
        if let Some(last) = self.last_vblank_us.replace(vblank_us) {
            if vblank_us.saturating_sub(last) > self.refresh_us * 3 / 2 {
                self.skipped += 1;
                if submit_us > last + self.refresh_us {
                    self.late += 1;
                }
            }
        } else {
            return None;
        }
        let lead = vblank_us.saturating_sub(submit_us);
        self.flips += 1;
        self.swap_sum_us += swap_us;
        self.swap_max_us = self.swap_max_us.max(swap_us);
        self.lead_sum_us += lead;
        self.lead_max_us = self.lead_max_us.max(lead);
        (self.flips == STATS_EVERY).then(|| self.report())
    }

    fn report(&mut self) -> String {
        let n = u64::from(self.flips.max(1));
        let ms = |us: u64| us as f64 / 1000.0;
        let line = format!(
            "slot: kms: {} flips, {} skipped a vblank ({} submitted after the vblank they were meant for, {} submitted in time), \
             eglSwapBuffers avg {:.2} max {:.2} ms, submit to scanout avg {:.2} max {:.2} ms",
            self.flips,
            self.skipped,
            self.late,
            self.skipped - self.late,
            ms(self.swap_sum_us / n),
            ms(self.swap_max_us),
            ms(self.lead_sum_us / n),
            ms(self.lead_max_us),
        );
        let (refresh_us, last) = (self.refresh_us, self.last_vblank_us);
        *self = FlipStats {
            refresh_us,
            last_vblank_us: last,
            ..FlipStats::default()
        };
        line
    }
}

#[repr(C)]
struct FlipState {
    pending: bool,
    sec: c_uint,
    usec: c_uint,
}

unsafe extern "C" fn flipped(_fd: c_int, _seq: c_uint, sec: c_uint, usec: c_uint, data: Ptr) {
    if !data.is_null() {
        let state = &mut *(data as *mut FlipState);
        state.pending = false;
        state.sec = sec;
        state.usec = usec;
    }
}

#[repr(C)]
struct Timespec {
    sec: i64,
    nsec: i64,
}

const CLOCK_MONOTONIC: c_int = 1;

extern "C" {
    fn clock_gettime(clock: c_int, ts: *mut Timespec) -> c_int;
}

fn now_us() -> u64 {
    let mut ts = Timespec { sec: 0, nsec: 0 };
    unsafe { clock_gettime(CLOCK_MONOTONIC, &mut ts) };
    ts.sec as u64 * 1_000_000 + ts.nsec as u64 / 1000
}

fn env_on(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|v| v != "0")
}

struct Output {
    connector: u32,
    crtc: u32,
    mode: ModeInfo,
}

pub struct KmsSurface {
    drm: Drm,
    gbm: Gbm,
    egl: Egl,
    gles: Library,
    card: std::fs::File,
    output: Output,
    saved_crtc: *mut DrmCrtc,
    gbm_device: Ptr,
    gbm_surface: Ptr,
    display: Ptr,
    surface: Ptr,
    context: Ptr,
    size: (u32, u32),
    on_screen: Option<Ptr>,
    framebuffers: Vec<(Ptr, u32)>,
    crtc_set: bool,
    flip: Box<FlipState>,
    flip_failed: bool,
    finish: bool,
    stats: Option<FlipStats>,
    swap_us: u64,
}

impl KmsSurface {
    pub fn new() -> Result<Self, GfxError> {
        use std::os::unix::fs::OpenOptionsExt;
        let card = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc_cloexec())
            .open(CARD)
            .map_err(|e| GfxError::Context(format!("{CARD}: {e}")))?;
        let drm = Drm::load()?;
        let gbm = Gbm::load()?;
        let egl = Egl::load()?;
        let gles = open(&["libGLESv2.so.2", "libGLESv2.so"])?;
        let fd = raw(&card);
        let output = unsafe { find_output(&drm, fd)? };
        let mode = output.mode;
        let size = (u32::from(mode.hdisplay), u32::from(mode.vdisplay));
        eprintln!(
            "slot: kms connector {} crtc {} mode {}x{} at {:.2} Hz",
            output.connector,
            output.crtc,
            size.0,
            size.1,
            mode.refresh_hz()
        );
        let saved_crtc = unsafe { (drm.get_crtc)(fd, output.crtc) };
        let mut surface = KmsSurface {
            drm,
            gbm,
            egl,
            gles,
            card,
            output,
            saved_crtc,
            gbm_device: std::ptr::null_mut(),
            gbm_surface: std::ptr::null_mut(),
            display: std::ptr::null_mut(),
            surface: std::ptr::null_mut(),
            context: std::ptr::null_mut(),
            size,
            on_screen: None,
            framebuffers: Vec::new(),
            crtc_set: false,
            flip: Box::new(FlipState {
                pending: false,
                sec: 0,
                usec: 0,
            }),
            flip_failed: false,
            finish: std::env::var_os("SLOT_KMS_FINISH").is_none_or(|v| v != "0"),
            stats: env_on("SLOT_TRACE_LATENCY").then(|| FlipStats::new(mode.refresh_hz())),
            swap_us: 0,
        };
        if surface.finish {
            eprintln!("slot: kms: waiting for the GPU before each flip");
        }
        unsafe { surface.start_gl()? };
        Ok(surface)
    }

    unsafe fn start_gl(&mut self) -> Result<(), GfxError> {
        let fd = raw(&self.card);
        self.gbm_device = (self.gbm.create_device)(fd);
        if self.gbm_device.is_null() {
            return Err(GfxError::Context("gbm_create_device failed".into()));
        }
        self.gbm_surface = (self.gbm.surface_create)(
            self.gbm_device,
            self.size.0,
            self.size.1,
            GBM_FORMAT_XRGB8888,
            GBM_BO_USE_SCANOUT | GBM_BO_USE_RENDERING,
        );
        if self.gbm_surface.is_null() {
            return Err(GfxError::Context("gbm_surface_create failed".into()));
        }
        let egl = &self.egl;
        self.display = egl.display_for(self.gbm_device);
        if self.display.is_null() {
            return Err(egl.fail("eglGetDisplay"));
        }
        if (egl.initialize)(self.display, std::ptr::null_mut(), std::ptr::null_mut()) == 0 {
            return Err(egl.fail("eglInitialize"));
        }
        if (egl.bind_api)(EGL_OPENGL_ES_API) == 0 {
            return Err(egl.fail("eglBindAPI"));
        }
        let config = self.pick_config()?;
        self.surface =
            (egl.create_window_surface)(self.display, config, self.gbm_surface, std::ptr::null());
        if self.surface.is_null() {
            return Err(egl.fail("eglCreateWindowSurface"));
        }
        let context_attrs = [EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE];
        self.context = (egl.create_context)(
            self.display,
            config,
            std::ptr::null_mut(),
            context_attrs.as_ptr(),
        );
        if self.context.is_null() {
            return Err(egl.fail("eglCreateContext"));
        }
        if (egl.make_current)(self.display, self.surface, self.surface, self.context) == 0 {
            return Err(egl.fail("eglMakeCurrent"));
        }
        Ok(())
    }

    unsafe fn pick_config(&self) -> Result<Ptr, GfxError> {
        let egl = &self.egl;
        let attrs = [
            EGL_SURFACE_TYPE,
            EGL_WINDOW_BIT,
            EGL_RED_SIZE,
            8,
            EGL_GREEN_SIZE,
            8,
            EGL_BLUE_SIZE,
            8,
            EGL_ALPHA_SIZE,
            0,
            EGL_RENDERABLE_TYPE,
            EGL_OPENGL_ES2_BIT,
            EGL_NONE,
        ];
        let mut configs = [std::ptr::null_mut(); 64];
        let mut found = 0;
        if (egl.choose_config)(
            self.display,
            attrs.as_ptr(),
            configs.as_mut_ptr(),
            configs.len() as i32,
            &mut found,
        ) == 0
        {
            return Err(egl.fail("eglChooseConfig"));
        }
        let configs = &configs[..found.clamp(0, 64) as usize];
        let visual = |c: Ptr| {
            let mut id = 0;
            (egl.get_config_attrib)(self.display, c, EGL_NATIVE_VISUAL_ID, &mut id);
            id as u32
        };
        configs
            .iter()
            .copied()
            .find(|c| visual(*c) == GBM_FORMAT_XRGB8888)
            .or_else(|| configs.first().copied())
            .ok_or_else(|| GfxError::Context("eglChooseConfig: no es2 window config".into()))
    }

    fn framebuffer(&mut self, bo: Ptr) -> Result<u32, GfxError> {
        if let Some((_, fb)) = self.framebuffers.iter().find(|(b, _)| *b == bo) {
            return Ok(*fb);
        }
        let handle = unsafe { (self.gbm.bo_get_handle)(bo) } as u32;
        let stride = unsafe { (self.gbm.bo_get_stride)(bo) };
        let mut fb = 0;
        let rc = unsafe {
            (self.drm.add_fb)(
                raw(&self.card),
                self.size.0,
                self.size.1,
                24,
                32,
                stride,
                handle,
                &mut fb,
            )
        };
        if rc != 0 {
            return Err(GfxError::Context(format!("drmModeAddFB: {rc}")));
        }
        self.framebuffers.push((bo, fb));
        Ok(fb)
    }

    fn set_crtc(&self, fb: u32) -> Result<(), GfxError> {
        let rc = unsafe {
            (self.drm.set_crtc)(
                raw(&self.card),
                self.output.crtc,
                fb,
                0,
                0,
                &self.output.connector,
                1,
                &self.output.mode,
            )
        };
        match rc {
            0 => Ok(()),
            _ => Err(GfxError::Context(format!(
                "drmModeSetCrtc: {}",
                std::io::Error::last_os_error()
            ))),
        }
    }

    fn flip(&mut self, fb: u32) -> Result<(), GfxError> {
        let fd = raw(&self.card);
        self.flip.pending = true;
        let data = &mut *self.flip as *mut FlipState as Ptr;
        let submitted = now_us();
        let rc = unsafe {
            (self.drm.page_flip)(fd, self.output.crtc, fb, DRM_MODE_PAGE_FLIP_EVENT, data)
        };
        if rc != 0 {
            self.flip.pending = false;
            if !std::mem::replace(&mut self.flip_failed, true) {
                eprintln!(
                    "slot: kms page flip refused ({}), setting the crtc each frame",
                    std::io::Error::last_os_error()
                );
            }
            return self.set_crtc(fb);
        }
        let mut ctx = EventContext {
            version: 2,
            vblank_handler: None,
            page_flip_handler: Some(flipped),
            page_flip_handler2: None,
            sequence_handler: None,
        };
        while self.flip.pending {
            let mut poll = PollFd {
                fd,
                events: POLLIN,
                revents: 0,
            };
            let ready = unsafe { poll_fds(&mut poll, 1, FLIP_TIMEOUT_MS) };
            if ready < 0 && std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            if ready <= 0 {
                self.flip.pending = false;
                return Err(GfxError::Context("kms page flip timed out".into()));
            }
            unsafe { (self.drm.handle_event)(fd, &mut ctx) };
        }
        let vblank = u64::from(self.flip.sec) * 1_000_000 + u64::from(self.flip.usec);
        if let Some(stats) = &mut self.stats {
            if let Some(line) = stats.record(self.swap_us, submitted, vblank) {
                eprintln!("{line}");
            }
        }
        Ok(())
    }
}

#[repr(C)]
struct PollFd {
    fd: c_int,
    events: i16,
    revents: i16,
}

const POLLIN: i16 = 1;

extern "C" {
    #[link_name = "poll"]
    fn poll_fds(fds: *mut PollFd, nfds: u64, timeout: c_int) -> c_int;
}

fn libc_cloexec() -> i32 {
    0o2_000_000
}

fn raw(file: &std::fs::File) -> c_int {
    use std::os::unix::io::AsRawFd;
    file.as_raw_fd()
}

unsafe fn find_output(drm: &Drm, fd: c_int) -> Result<Output, GfxError> {
    let res = (drm.get_resources)(fd);
    if res.is_null() {
        return Err(GfxError::Context(format!(
            "drmModeGetResources: {}",
            std::io::Error::last_os_error()
        )));
    }
    let r = &*res;
    let ids = slice(r.connectors, r.count_connectors);
    let crtcs = slice(r.crtcs, r.count_crtcs).to_vec();
    let mut found = Vec::new();
    for id in ids {
        let c = (drm.get_connector)(fd, *id);
        if c.is_null() {
            continue;
        }
        let conn = &*c;
        let encoders = slice(conn.encoders, conn.count_encoders).to_vec();
        found.push((
            Connector {
                id: conn.connector_id,
                kind: conn.connector_type,
                connected: conn.connection == DRM_MODE_CONNECTED,
                modes: slice(conn.modes, conn.count_modes).to_vec(),
            },
            conn.encoder_id,
            encoders,
        ));
        (drm.free_connector)(c);
    }
    (drm.free_resources)(res);
    let list: Vec<Connector> = found.iter().map(|(c, _, _)| c.clone()).collect();
    let pick = pick_connector(&list)
        .ok_or_else(|| GfxError::Context("kms: no connected display".into()))?;
    let (conn, current, encoders) = &found[pick];
    let mode = conn.modes[pick_mode(&conn.modes).unwrap_or(0)];
    let crtc = crtc_of(drm, fd, *current)
        .or_else(|| {
            encoders.iter().find_map(|e| {
                let enc = (drm.get_encoder)(fd, *e);
                if enc.is_null() {
                    return None;
                }
                let possible = (*enc).possible_crtcs;
                (drm.free_encoder)(enc);
                crtc_for(possible, &crtcs)
            })
        })
        .ok_or_else(|| GfxError::Context("kms: no crtc for the display".into()))?;
    Ok(Output {
        connector: conn.id,
        crtc,
        mode,
    })
}

unsafe fn crtc_of(drm: &Drm, fd: c_int, encoder: u32) -> Option<u32> {
    if encoder == 0 {
        return None;
    }
    let enc = (drm.get_encoder)(fd, encoder);
    if enc.is_null() {
        return None;
    }
    let crtc = (*enc).crtc_id;
    (drm.free_encoder)(enc);
    (crtc != 0).then_some(crtc)
}

unsafe fn slice<'a, T>(ptr: *const T, count: c_int) -> &'a [T] {
    match (ptr.is_null(), usize::try_from(count)) {
        (false, Ok(n)) if n > 0 => std::slice::from_raw_parts(ptr, n),
        _ => &[],
    }
}

impl Surface for KmsSurface {
    fn make_current(&mut self) -> Result<(), GfxError> {
        let ok = unsafe {
            (self.egl.make_current)(self.display, self.surface, self.surface, self.context)
        };
        match ok {
            0 => Err(self.egl.fail("eglMakeCurrent")),
            _ => Ok(()),
        }
    }

    fn window_size(&self) -> (u32, u32) {
        self.size
    }

    fn settle(&mut self) {
        if self.finish {
            unsafe { ::gl::Finish() };
        }
    }

    fn pace_margin(&self) -> Option<std::time::Duration> {
        Some(PACE_MARGIN)
    }

    fn swap(&mut self) -> Result<(), GfxError> {
        let began = now_us();
        if unsafe { (self.egl.swap_buffers)(self.display, self.surface) } == 0 {
            return Err(self.egl.fail("eglSwapBuffers"));
        }
        self.swap_us = now_us().saturating_sub(began);
        let bo = unsafe { (self.gbm.lock_front_buffer)(self.gbm_surface) };
        if bo.is_null() {
            return Err(GfxError::Context(
                "gbm_surface_lock_front_buffer failed".into(),
            ));
        }
        let shown = self.framebuffer(bo).and_then(|fb| match self.crtc_set {
            true => self.flip(fb),
            false => self.set_crtc(fb).map(|()| self.crtc_set = true),
        });
        if let Err(e) = shown {
            unsafe { (self.gbm.release_buffer)(self.gbm_surface, bo) };
            return Err(e);
        }
        if let Some(old) = self.on_screen.replace(bo) {
            unsafe { (self.gbm.release_buffer)(self.gbm_surface, old) };
        }
        Ok(())
    }

    fn proc_address(&self, name: &str) -> *const c_void {
        let Ok(c) = CString::new(name) else {
            return std::ptr::null();
        };
        let exported = unsafe { self.gles.get::<unsafe extern "C" fn()>(name.as_bytes()) };
        match exported {
            Ok(f) => *f as *const c_void,
            Err(_) => unsafe { (self.egl.get_proc_address)(c.as_ptr()) },
        }
    }
}

impl Drop for KmsSurface {
    fn drop(&mut self) {
        let fd = raw(&self.card);
        unsafe {
            if !self.saved_crtc.is_null() {
                let saved = &*self.saved_crtc;
                if saved.mode_valid != 0 && saved.buffer_id != 0 {
                    (self.drm.set_crtc)(
                        fd,
                        saved.crtc_id,
                        saved.buffer_id,
                        saved.x,
                        saved.y,
                        &self.output.connector,
                        1,
                        &saved.mode,
                    );
                }
                (self.drm.free_crtc)(self.saved_crtc);
            }
            if !self.display.is_null() {
                (self.egl.make_current)(
                    self.display,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
                if !self.context.is_null() {
                    (self.egl.destroy_context)(self.display, self.context);
                }
                if !self.surface.is_null() {
                    (self.egl.destroy_surface)(self.display, self.surface);
                }
                (self.egl.terminate)(self.display);
            }
            for (_, fb) in self.framebuffers.drain(..) {
                (self.drm.rm_fb)(fd, fb);
            }
            if let Some(bo) = self.on_screen.take() {
                (self.gbm.release_buffer)(self.gbm_surface, bo);
            }
            if !self.gbm_surface.is_null() {
                (self.gbm.surface_destroy)(self.gbm_surface);
            }
            if !self.gbm_device.is_null() {
                (self.gbm.device_destroy)(self.gbm_device);
            }
        }
    }
}
