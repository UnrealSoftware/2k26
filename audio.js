const audio_ctx = new (window.AudioContext || window.webkitAudioContext)();
let audio_sources = {};
let next_audio_id = 1;

miniquad_add_plugin({
	register_plugin: function (importObject) {

		importObject.env.play_sound_from_file = function (path_ptr, path_len, volume, x, y, looping) {
			let bytes = new Uint8Array(wasm_memory.buffer, path_ptr, path_len);
			let path_str = new TextDecoder('utf-8').decode(bytes);
			let id = next_audio_id++;

			fetch(path_str)
				.then(response => response.arrayBuffer())
				.then(buffer => audio_ctx.decodeAudioData(buffer))
				.then(decoded => play_decoded(id, decoded, volume, x, y, looping))
				.catch(e => console.error("Audio Load Error:", e));
			return id;
		};

		importObject.env.play_sound_from_memory = function (data_ptr, data_len, volume, x, y, looping) {
			let bytes = new Uint8Array(wasm_memory.buffer, data_ptr, data_len);
			let buffer = bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
			let id = next_audio_id++;

			audio_ctx.decodeAudioData(buffer)
				.then(decoded => play_decoded(id, decoded, volume, x, y, looping))
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

		importObject.env.set_sound_position = function (id, x, y) {
			if (audio_sources[id] && audio_sources[id].panner) {
				audio_sources[id].panner.positionX.value = x;
				audio_sources[id].panner.positionZ.value = y; // Game Y -> Audio Z
			}
		};
	},
	on_init: function () {},
	name: "spatial_audio_plugin",
	version: "1.0.0"
});

function play_decoded(id, buffer, volume, x, y, looping) {
	let source = audio_ctx.createBufferSource();
	source.buffer = buffer;
	source.loop = (looping !== 0);

	let gainNode = audio_ctx.createGain();
	gainNode.gain.value = volume;

	let pannerNode = audio_ctx.createPanner();
	pannerNode.panningModel = 'HRTF';
	pannerNode.distanceModel = 'inverse';
	pannerNode.refDistance = 1.0;
	pannerNode.maxDistance = 10000.0;
	pannerNode.rolloffFactor = 1.0;

	pannerNode.positionX.value = x;
	pannerNode.positionY.value = 0.0; // Flat 2D Elevation
	pannerNode.positionZ.value = y;   // Game Y -> Audio Z

	source.connect(pannerNode);
	pannerNode.connect(gainNode);
	gainNode.connect(audio_ctx.destination);
	source.start();

	source.onended = () => {
		if (audio_sources[id] && !source.loop) delete audio_sources[id];
	};

	audio_sources[id] = { source: source, gain: gainNode, panner: pannerNode };
}