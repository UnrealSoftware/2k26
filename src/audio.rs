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
        fn play_sound_from_file(path_ptr: *const u8, path_len: usize, volume: f32, x: f32, y: f32, looping: u32) -> u32;
        fn play_sound_from_memory(data_ptr: *const u8, data_len: usize, volume: f32, x: f32, y: f32, looping: u32) -> u32;
        fn stop_sound(id: u32);
        fn set_sound_volume(id: u32, volume: f32);
        fn set_sound_position(id: u32, x: f32, y: f32);
    }

    pub struct AudioPlayer {}

    impl AudioPlayer {
        pub fn new() -> Self { Self {} }

        pub fn play_file(&mut self, path: &str, volume: f32, pos: [f32; 2], looping: bool) -> u32 {
            unsafe { play_sound_from_file(path.as_ptr(), path.len(), volume, pos[0], pos[1], if looping { 1 } else { 0 }) }
        }

        pub fn play_memory(&mut self, data: &[u8], volume: f32, pos: [f32; 2], looping: bool) -> u32 {
            unsafe { play_sound_from_memory(data.as_ptr(), data.len(), volume, pos[0], pos[1], if looping { 1 } else { 0 }) }
        }

        pub fn stop(&mut self, id: u32) { unsafe { stop_sound(id) } }
        pub fn set_volume(&mut self, id: u32, volume: f32) { unsafe { set_sound_volume(id, volume) } }
        pub fn set_position(&mut self, id: u32, pos: [f32; 2]) { unsafe { set_sound_position(id, pos[0], pos[1]) } }
    }
}

// ==========================================
// Native Implementation (Kira)
// ==========================================
#[cfg(not(target_arch = "wasm32"))]
mod native_audio {
    use kira::{
        AudioManager, AudioManagerSettings, DefaultBackend, Decibels, Tween,
        sound::static_sound::{StaticSoundData, StaticSoundHandle},
        track::{SpatialTrackBuilder, SpatialTrackHandle, SpatialTrackDistances},
        listener::ListenerHandle,
    };
    use std::collections::HashMap;

    pub struct AudioPlayer {
        manager: AudioManager,
        listener: ListenerHandle,
        sounds: HashMap<u32, (StaticSoundHandle, SpatialTrackHandle)>,
        next_id: u32,
    }

    impl AudioPlayer {
        pub fn new() -> Self {
            let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
                .expect("Failed to initialize Kira");

            // Listener sits at the origin. We use arrays that implicitly convert into glam/mint types.
            let listener = manager.add_listener([0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0]).unwrap();

            Self { manager, listener, sounds: HashMap::new(), next_id: 1 }
        }

        fn map_to_3d(pos: [f32; 2]) -> [f32; 3] {
            [pos[0], 0.0, pos[1]]
        }

        fn amp_to_db(amplitude: f32) -> Decibels {
            if amplitude <= 0.0001 {
                Decibels::SILENCE
            } else {
                Decibels(20.0 * amplitude.log10())
            }
        }

        pub fn play_track(&mut self, data: StaticSoundData, pos: [f32; 2]) -> u32 {
            let distances = SpatialTrackDistances {
                min_distance: 32.0,
                max_distance: 10000.0,
            };
            let builder = SpatialTrackBuilder::new().distances(distances);
            if let Ok(mut track) = self.manager.add_spatial_sub_track(&self.listener, Self::map_to_3d(pos), builder) {
                if let Ok(handle) = track.play(data) {
                    let id = self.next_id;
                    self.sounds.insert(id, (handle, track));
                    self.next_id += 1;
                    return id;
                }
            }
            0
        }

        pub fn play_file(&mut self, path: &str, volume: f32, pos: [f32; 2], looping: bool) -> u32 {
            if let Ok(mut data) = StaticSoundData::from_file(path) {
                data = data.volume(Self::amp_to_db(volume));
                if looping { data = data.loop_region(..); }
                Self::play_track(self, data, pos);
            }
            0
        }

        pub fn play_memory(&mut self, data: &[u8], volume: f32, pos: [f32; 2], looping: bool) -> u32 {
            let cursor = std::io::Cursor::new(data.to_vec());

            if let Ok(mut data) = StaticSoundData::from_cursor(cursor) {
                data = data.volume(Self::amp_to_db(volume));
                if looping { data = data.loop_region(..); }
                Self::play_track(self, data, pos);
            }
            0
        }

        pub fn stop(&mut self, id: u32) {
            if let Some((mut handle, _)) = self.sounds.remove(&id) {
                let _ = handle.stop(Tween::default());
            }
        }

        pub fn set_volume(&mut self, id: u32, volume: f32) {
            if let Some((handle, _)) = self.sounds.get_mut(&id) {
                let _ = handle.set_volume(Self::amp_to_db(volume), Tween::default());
            }
        }

        pub fn set_position(&mut self, id: u32, pos: [f32; 2]) {
            if let Some((_, track)) = self.sounds.get_mut(&id) {
                let _ = track.set_position(Self::map_to_3d(pos), Tween::default());
            }
        }
    }
}