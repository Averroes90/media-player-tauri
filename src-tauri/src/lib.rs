use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;
use std::sync::Mutex;
use tauri::{Manager, Window};

// Video area coordinates from frontend
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoArea {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

// MPV player state with native rendering support
pub struct MpvPlayer {
    handle: Option<*mut libmpv_sys::mpv_handle>,
    video_area: Option<VideoArea>,
    // We'll use this field when we implement more advanced native rendering
    #[allow(dead_code)]
    #[cfg(target_os = "macos")]
    child_window: Option<*mut std::os::raw::c_void>,
}

unsafe impl Send for MpvPlayer {}
unsafe impl Sync for MpvPlayer {}

impl MpvPlayer {
    fn new() -> Self {
        Self {
            handle: None,
            video_area: None,
            #[cfg(target_os = "macos")]
            child_window: None,
        }
    }
}

// Native rendering setup for macOS
#[cfg(target_os = "macos")]
fn setup_macos_native_rendering(
    window: &Window,
    mpv_handle: *mut libmpv_sys::mpv_handle,
    _video_area: &VideoArea,
) -> Result<(), String> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    unsafe {
        // Get the raw window handle from Tauri
        let raw_handle = window
            .window_handle()
            .map_err(|e| format!("Failed to get window handle: {}", e))?
            .as_raw();

        match raw_handle {
            RawWindowHandle::AppKit(appkit_handle) => {
                // Get the native view pointer
                let ns_view = appkit_handle.ns_view.as_ptr() as i64;

                // Set the window ID for MPV to render into
                let wid_prop = CString::new("wid").unwrap();
                let ret = libmpv_sys::mpv_set_property(
                    mpv_handle,
                    wid_prop.as_ptr(),
                    libmpv_sys::mpv_format_MPV_FORMAT_INT64,
                    &ns_view as *const i64 as *mut c_void,
                );

                if ret != 0 {
                    return Err(format!("Failed to set window ID: error code {}", ret));
                }

                // Force MPV to use the embedded view
                let force_window_prop = CString::new("force-window").unwrap();
                let force_window_value = CString::new("yes").unwrap();
                libmpv_sys::mpv_set_property_string(
                    mpv_handle,
                    force_window_prop.as_ptr(),
                    force_window_value.as_ptr(),
                );

                println!("MPV configured to render into view: {}", ns_view);
                Ok(())
            }
            _ => Err("Unsupported window handle type".to_string()),
        }
    }
}

// Tauri commands
#[tauri::command]
fn init_mpv_player(state: tauri::State<Mutex<MpvPlayer>>) -> Result<String, String> {
    let mut player = state.lock().unwrap();

    unsafe {
        let handle = libmpv_sys::mpv_create();
        if handle.is_null() {
            return Err("Failed to create MPV handle".to_string());
        }

        // CRITICAL: Set up MPV for embedding BEFORE initialization
        let vo_prop = CString::new("vo").unwrap();
        let vo_value = CString::new("libmpv").unwrap();
        libmpv_sys::mpv_set_option_string(handle, vo_prop.as_ptr(), vo_value.as_ptr());

        // Disable video output initially (no separate window)
        let vid_prop = CString::new("vid").unwrap();
        let vid_value = CString::new("no").unwrap();
        libmpv_sys::mpv_set_option_string(handle, vid_prop.as_ptr(), vid_value.as_ptr());

        // Initialize MPV
        let ret = libmpv_sys::mpv_initialize(handle);
        if ret != 0 {
            libmpv_sys::mpv_destroy(handle);
            return Err(format!("Failed to initialize MPV: {}", ret));
        }

        // Basic configuration
        let hwdec_prop = CString::new("hwdec").unwrap();
        let hwdec_value = CString::new("auto").unwrap();
        libmpv_sys::mpv_set_option_string(handle, hwdec_prop.as_ptr(), hwdec_value.as_ptr());

        player.handle = Some(handle);
        Ok("MPV initialized for embedding - no popup windows".to_string())
    }
}

#[tauri::command]
fn setup_video_rendering(
    window: Window,
    video_area: VideoArea,
    state: tauri::State<Mutex<MpvPlayer>>,
) -> Result<String, String> {
    let mut player = state.lock().unwrap();

    if let Some(handle) = player.handle {
        player.video_area = Some(video_area.clone());

        unsafe {
            // Enable video output now that we have a target
            let vid_prop = CString::new("vid").unwrap();
            let vid_value = CString::new("auto").unwrap();
            libmpv_sys::mpv_set_property_string(handle, vid_prop.as_ptr(), vid_value.as_ptr());

            #[cfg(target_os = "macos")]
            {
                match setup_macos_native_rendering(&window, handle, &video_area) {
                    Ok(_) => Ok(format!(
                        "✅ Video will render in app (no popup) at {}x{}",
                        video_area.width, video_area.height
                    )),
                    Err(e) => Err(format!("Failed to setup embedding: {}", e)),
                }
            }

            #[cfg(not(target_os = "macos"))]
            {
                Ok("Video rendering not yet implemented for this platform".to_string())
            }
        }
    } else {
        Err("MPV not initialized - call init_mpv_player first".to_string())
    }
}

#[tauri::command]
fn load_video(file_path: String, state: tauri::State<Mutex<MpvPlayer>>) -> Result<String, String> {
    let player = state.lock().unwrap();

    if let Some(handle) = player.handle {
        unsafe {
            let cmd = CString::new("loadfile").unwrap();
            let path = CString::new(file_path.clone()).unwrap();
            let mut args = [cmd.as_ptr(), path.as_ptr(), ptr::null()];

            let ret = libmpv_sys::mpv_command(handle, args.as_mut_ptr());
            if ret != 0 {
                return Err(format!("Failed to load file: {}", ret));
            }
            Ok(format!("🎬 Loading video: {}", file_path))
        }
    } else {
        Err("MPV not initialized".to_string())
    }
}

#[tauri::command]
fn play_pause(state: tauri::State<Mutex<MpvPlayer>>) -> Result<String, String> {
    let player = state.lock().unwrap();

    if let Some(handle) = player.handle {
        unsafe {
            let prop = CString::new("pause").unwrap();
            let mut paused: i64 = 0;
            let ret = libmpv_sys::mpv_get_property(
                handle,
                prop.as_ptr(),
                libmpv_sys::mpv_format_MPV_FORMAT_FLAG,
                &mut paused as *mut _ as *mut c_void,
            );

            if ret == 0 {
                let new_pause = if paused != 0 { 0i64 } else { 1i64 };
                let ret = libmpv_sys::mpv_set_property(
                    handle,
                    prop.as_ptr(),
                    libmpv_sys::mpv_format_MPV_FORMAT_FLAG,
                    &new_pause as *const _ as *mut c_void,
                );

                if ret == 0 {
                    Ok(if paused != 0 {
                        "▶️ Playing"
                    } else {
                        "⏸️ Paused"
                    }
                    .to_string())
                } else {
                    Err(format!("Failed to toggle pause: {}", ret))
                }
            } else {
                Err(format!("Failed to get pause state: {}", ret))
            }
        }
    } else {
        Err("MPV not initialized".to_string())
    }
}

#[tauri::command]
fn stop_video(state: tauri::State<Mutex<MpvPlayer>>) -> Result<String, String> {
    let player = state.lock().unwrap();

    if let Some(handle) = player.handle {
        unsafe {
            let cmd = CString::new("stop").unwrap();
            let mut args = [cmd.as_ptr(), ptr::null()];

            let ret = libmpv_sys::mpv_command(handle, args.as_mut_ptr());
            if ret != 0 {
                return Err(format!("Failed to stop: {}", ret));
            }
            Ok("⏹️ Stopped".to_string())
        }
    } else {
        Err("MPV not initialized".to_string())
    }
}

#[tauri::command]
fn set_playback_speed(speed: f64, state: tauri::State<Mutex<MpvPlayer>>) -> Result<String, String> {
    let player = state.lock().unwrap();

    if let Some(handle) = player.handle {
        unsafe {
            // Validate speed range
            if !(0.25..=4.0).contains(&speed) {
                return Err("Speed must be between 0.25x and 4.0x".to_string());
            }

            let prop = CString::new("speed").unwrap();
            let ret = libmpv_sys::mpv_set_property(
                handle,
                prop.as_ptr(),
                libmpv_sys::mpv_format_MPV_FORMAT_DOUBLE,
                &speed as *const f64 as *mut c_void,
            );

            if ret == 0 {
                Ok(format!("🏃 Speed set to {:.2}x", speed))
            } else {
                Err(format!("Failed to set speed: {}", ret))
            }
        }
    } else {
        Err("MPV not initialized".to_string())
    }
}

#[tauri::command]
fn get_playback_speed(state: tauri::State<Mutex<MpvPlayer>>) -> Result<f64, String> {
    let player = state.lock().unwrap();

    if let Some(handle) = player.handle {
        unsafe {
            let prop = CString::new("speed").unwrap();
            let mut speed: f64 = 1.0;
            let ret = libmpv_sys::mpv_get_property(
                handle,
                prop.as_ptr(),
                libmpv_sys::mpv_format_MPV_FORMAT_DOUBLE,
                &mut speed as *mut f64 as *mut c_void,
            );

            if ret == 0 {
                Ok(speed)
            } else {
                Err(format!("Failed to get speed: {}", ret))
            }
        }
    } else {
        Err("MPV not initialized".to_string())
    }
}

#[tauri::command]
fn speed_preset(speed: f64, state: tauri::State<Mutex<MpvPlayer>>) -> Result<String, String> {
    set_playback_speed(speed, state)
}

// Window positioning commands (from your existing code)
#[tauri::command]
fn move_to_monitor(window: tauri::Window, monitor_index: u32) -> Result<String, String> {
    use tauri::{LogicalPosition, Position};

    let (x, y) = match monitor_index {
        0 => (0, 100),
        1 => (1920, 100),
        2 => (3840, 100),
        _ => return Err("Invalid monitor index. Use 0, 1, or 2.".to_string()),
    };

    window
        .set_position(Position::Logical(LogicalPosition {
            x: x as f64,
            y: y as f64,
        }))
        .map_err(|e| format!("Failed to move window: {}", e))?;

    Ok(format!("Moved window to monitor {}", monitor_index))
}

fn setup_window_position(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let window = app.get_webview_window("main").unwrap();
    let preferred_monitor = 2;

    let (x, y) = match preferred_monitor {
        0 => (0, 100),
        1 => (1920, 100),
        2 => (3840, 100),
        _ => (0, 100),
    };

    use tauri::{LogicalPosition, Position};
    window.set_position(Position::Logical(LogicalPosition {
        x: x as f64,
        y: y as f64,
    }))?;

    println!("Window moved to monitor {}", preferred_monitor);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(MpvPlayer::new()))
        .invoke_handler(tauri::generate_handler![
            init_mpv_player,
            setup_video_rendering,
            load_video,
            play_pause,
            stop_video,
            set_playback_speed,
            get_playback_speed,
            speed_preset,
            move_to_monitor
        ])
        .setup(|app| {
            if let Err(e) = setup_window_position(app) {
                eprintln!("Failed to position window: {}", e);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
