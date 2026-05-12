// Export the correct platform implementation
#[cfg(target_arch = "wasm32")]
pub use wasm_audio::AudioPlayer;

#[cfg(not(target_arch = "wasm32"))]
pub use native_audio::AudioPlayer;

// ==========================================
// WebAssembly Implementation (JavaScript)
// ==========================================
#[cfg(target_arch = "wasm32")]
mod wasm_audio {
    extern "C" {
        // C-FFI bound functions provided by our JavaScript plugin
        fn play_sound_from_file(path_ptr: *const u8, path_len: usize, volume: f32, pan: f32, looping: u32) -> u32;
        fn play_sound_from_memory(data_ptr: *const u8, data_len: usize, volume: f32, pan: f32, looping: u32) -> u32;
        fn stop_sound(id: u32);
        fn set_sound_volume(id: u32, volume: f32);
        fn set_sound_pan(id: u32, pan: f32);
    }

    pub struct AudioPlayer {}

    impl AudioPlayer {
        pub fn new() -> Self {
            Self {}
        }

        pub fn play_file(&mut self, path: &str, volume: f32, pan: f32, looping: bool) -> u32 {
            unsafe {
                play_sound_from_file(
                    path.as_ptr(), path.len(),
                    volume, pan,
                    if looping { 1 } else { 0 }
                )
            }
        }

        pub fn play_memory(&mut self, data: &[u8], volume: f32, pan: f32, looping: bool) -> u32 {
            unsafe {
                play_sound_from_memory(
                    data.as_ptr(), data.len(),
                    volume, pan,
                    if looping { 1 } else { 0 }
                )
            }
        }

        pub fn stop(&mut self, id: u32) {
            unsafe { stop_sound(id) }
        }

        pub fn set_volume(&mut self, id: u32, volume: f32) {
            unsafe { set_sound_volume(id, volume) }
        }

        pub fn set_pan(&mut self, id: u32, pan: f32) {
            unsafe { set_sound_pan(id, pan) }
        }
    }
}

// ==========================================
// Native Implementation (Kira)
// ==========================================
#[cfg(not(target_arch = "wasm32"))]
mod native_audio {
    use kira::{
        AudioManager, AudioManagerSettings, DefaultBackend,
        sound::static_sound::{StaticSoundData, StaticSoundSettings, StaticSoundHandle},
        Tween,
    };
    use std::collections::HashMap;

    pub struct AudioPlayer {
        manager: AudioManager,
        sounds: HashMap<u32, StaticSoundHandle>,
        next_id: u32,
    }

    impl AudioPlayer {
        pub fn new() -> Self {
            let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
                .expect("Failed to initialize Kira AudioManager");
            Self {
                manager,
                sounds: HashMap::new(),
                next_id: 1,
            }
        }

        pub fn play_file(&mut self, path: &str, volume: f32, pan: f32, looping: bool) -> u32 {
            let mut settings = StaticSoundSettings::new()
                .volume(volume as f64)
                .panning(pan as f64);

            if looping {
                settings = settings.loop_region(..); // standard way to loop entire track
            }

            if let Ok(data) = StaticSoundData::from_file(path, settings) {
                if let Ok(handle) = self.manager.play(data) {
                    let id = self.next_id;
                    self.sounds.insert(id, handle);
                    self.next_id += 1;
                    return id;
                }
            }
            0
        }

        pub fn play_memory(&mut self, data: &[u8], volume: f32, pan: f32, looping: bool) -> u32 {
            let cursor = std::io::Cursor::new(data.to_vec());
            let mut settings = StaticSoundSettings::new()
                .volume(volume as f64)
                .panning(pan as f64);

            if looping {
                settings = settings.loop_region(..);
            }

            if let Ok(data) = StaticSoundData::from_cursor(cursor, settings) {
                if let Ok(handle) = self.manager.play(data) {
                    let id = self.next_id;
                    self.sounds.insert(id, handle);
                    self.next_id += 1;
                    return id;
                }
            }
            0
        }

        pub fn stop(&mut self, id: u32) {
            if let Some(mut handle) = self.sounds.remove(&id) {
                let _ = handle.stop(Tween::default());
            }
        }

        pub fn set_volume(&mut self, id: u32, volume: f32) {
            if let Some(handle) = self.sounds.get_mut(&id) {
                let _ = handle.set_volume(volume as f64, Tween::default());
            }
        }

        pub fn set_pan(&mut self, id: u32, pan: f32) {
            if let Some(handle) = self.sounds.get_mut(&id) {
                let _ = handle.set_panning(pan as f64, Tween::default());
            }
        }
    }
}