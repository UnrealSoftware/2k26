const audio_ctx = new (window.AudioContext || window.webkitAudioContext)();
let audio_sources = {};
let next_audio_id = 1;

// Create the Macroquad JS Plugin to intercept the extern C calls
miniquad_add_plugin({
	register_plugin: function (importObject) {

		// Play from File (Fetches over HTTP)
		importObject.env.play_sound_from_file = function (path_ptr, path_len, volume, pan, looping) {
			let bytes = new Uint8Array(wasm_memory.buffer, path_ptr, path_len);
			let path_str = new TextDecoder('utf-8').decode(bytes);
			let id = next_audio_id++;

			fetch(path_str)
				.then(response => response.arrayBuffer())
				.then(buffer => audio_ctx.decodeAudioData(buffer))
				.then(decoded => play_decoded(id, decoded, volume, pan, looping))
				.catch(e => console.error("Audio Load Error:", e));
			return id;
		};

		// Play from Memory (Reads directly from WASM memory)
		importObject.env.play_sound_from_memory = function (data_ptr, data_len, volume, pan, looping) {
			let bytes = new Uint8Array(wasm_memory.buffer, data_ptr, data_len);
			// We must slice to copy the buffer, as `decodeAudioData` requires an unshared buffer,
			// and wasm_memory can't be detached.
			let buffer = bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
			let id = next_audio_id++;

			audio_ctx.decodeAudioData(buffer)
				.then(decoded => play_decoded(id, decoded, volume, pan, looping))
				.catch(e => console.error("Audio Decode Error:", e));
			return id;
		};

		importObject.env.stop_sound = function (id) {
			if (audio_sources[id]) {
				audio_sources[id].source.stop();
				delete audio_sources[id];
			}
		};

		importObject.env.set_sound_volume = function (id, volume) {
			if (audio_sources[id]) audio_sources[id].gain.gain.value = volume;
		};

		importObject.env.set_sound_pan = function (id, pan) {
			if (audio_sources[id] && audio_sources[id].panner) {
				audio_sources[id].panner.pan.value = pan;
			}
		};
	},
	on_init: function () {},
	name: "spatial_audio_plugin",
	version: "1.0.0"
});

// Core WebAudio pipeline
function play_decoded(id, buffer, volume, pan, looping) {
	let source = audio_ctx.createBufferSource();
	source.buffer = buffer;
	source.loop = (looping !== 0);

	let gainNode = audio_ctx.createGain();
	gainNode.gain.value = volume;

	let pannerNode = audio_ctx.createStereoPanner();
	pannerNode.pan.value = pan;

	// Connect nodes: Source -> Panner -> Gain -> Output
	source.connect(pannerNode);
	pannerNode.connect(gainNode);
	gainNode.connect(audio_ctx.destination);
	source.start();

	source.onended = () => {
		if (audio_sources[id] && !source.loop) delete audio_sources[id];
	};

	audio_sources[id] = { source: source, gain: gainNode, panner: pannerNode };
}

// Web Browsers block audio until the user interacts with the page!
document.addEventListener('click', function() {
	if (audio_ctx.state === 'suspended') audio_ctx.resume();
});