use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;
use std::sync::Mutex;

// Simple MPV player state using libmpv-sys
pub struct MpvPlayer {
    handle: Option<*mut libmpv_sys::mpv_handle>,
}

unsafe impl Send for MpvPlayer {}
unsafe impl Sync for MpvPlayer {}

impl MpvPlayer {
    fn new() -> Self {
        Self { handle: None }
    }
}

// Tauri commands for MPV control
#[tauri::command]
fn init_mpv_player(state: tauri::State<Mutex<MpvPlayer>>) -> Result<String, String> {
    let mut player = state.lock().unwrap();

    unsafe {
        let handle = libmpv_sys::mpv_create();
        if handle.is_null() {
            return Err("Failed to create MPV handle".to_string());
        }

        // Initialize MPV
        let ret = libmpv_sys::mpv_initialize(handle);
        if ret != 0 {
            libmpv_sys::mpv_destroy(handle);
            return Err(format!("Failed to initialize MPV: {}", ret));
        }

        // Set basic properties for embedding
        let vo = CString::new("vo").unwrap();
        let vo_val = CString::new("libmpv").unwrap();
        libmpv_sys::mpv_set_option_string(handle, vo.as_ptr(), vo_val.as_ptr());

        player.handle = Some(handle);
        Ok("MPV initialized successfully".to_string())
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
            Ok(format!("Loading video: {}", file_path))
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
                    Ok(if paused != 0 { "Playing" } else { "Paused" }.to_string())
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
            Ok("Stopped".to_string())
        }
    } else {
        Err("MPV not initialized".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(MpvPlayer::new()))
        .invoke_handler(tauri::generate_handler![
            init_mpv_player,
            load_video,
            play_pause,
            stop_video
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
