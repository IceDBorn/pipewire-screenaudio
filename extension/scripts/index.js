(() => {
	let sessionType = null;
	let titleWatcher = null;

	const instanceIdentifier = `pipewire-screenaudio-${createRandomString(16)}`;

	const getTitleWithInstanceIdentifier = (title) =>
		`${title} | ${instanceIdentifier}`;

	function createRandomString(
		length,
		chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
	) {
		const array = new Uint8Array(length);
		crypto.getRandomValues(array);
		return Array.from(array, (byte) => chars[byte % chars.length]).join("");
	}

	function overrideGetDisplayMedia() {
		navigator.mediaDevices.chromiumGetDisplayMedia =
			navigator.mediaDevices.getDisplayMedia;

		const getAudioDevice = async (nameOfAudioDevice) => {
			await navigator.mediaDevices.getUserMedia({
				audio: true,
			});

			await new Promise((resolve, reject) => setTimeout(resolve, 1000));
			const devices = await navigator.mediaDevices.enumerateDevices();

			const audioDevice = devices.find(
				({ label }) => label === nameOfAudioDevice,
			);

			return audioDevice;
		};

		const getDisplayMedia = async () => {
			let micId;

			try {
				micId = await getAudioDevice("pipewire-screenaudio").then(
					({ deviceId }) => deviceId,
				);
			} catch {
				return await navigator.mediaDevices.chromiumGetDisplayMedia({
					video: true,
					audio: false,
				});
			}

			const capturePipewireScreenaudioMic =
				await navigator.mediaDevices.getUserMedia({
					audio: {
						deviceId: {
							exact: micId,
						},

						// We want auto gain control, noise cancellation and noise suppression disabled so that our stream won't sound bad
						autoGainControl: false,
						echoCancellation: false,
						noiseSuppression: false,

						// We can set more audio constraints here, bellow are some examples
						// channelCount: 2,
						// latency: 0,
						// sampleRate: 48000,
						// sampleSize: 16,
						// volume: 1.0
					},
				});

			const [track] = capturePipewireScreenaudioMic.getAudioTracks();

			const displayMedia = await navigator.mediaDevices.chromiumGetDisplayMedia(
				{
					video: true,
					audio: true,
				},
			);

			displayMedia.addTrack(track);
			watchTitle();

			// Trigger title change to append the identifier
			document.title = document.title;

			// Send the node name to exclude for All Desktop Audio
			window.postMessage({
				message: "instance-identifier",
				instanceIdentifier,
			});

			// Watch track and clear instance when ended
			// Workaround solution for firefox, because it does not support MediaStream's inactive event
			const trackWatcher = setInterval(() => {
				if (track.readyState !== "ended") return;

				// TODO: Add instance clearing native logic when implementing multiple instances
				console.log("track ended");

				clearInterval(trackWatcher);
			}, 50);

			return displayMedia;
		};

		navigator.mediaDevices.getDisplayMedia = getDisplayMedia;
	}

	// Watch the title element for changes and append our identifier if missing
	function watchTitle() {
		if (titleWatcher) return;

		const titleEl = document.querySelector("title");
		if (!titleEl) return;

		titleWatcher = new MutationObserver((mutations) => {
			for (const mut of mutations) {
				if (["childList", "characterData"].includes(mut.type)) {
					if (document.title.includes(instanceIdentifier)) break;
					document.title = getTitleWithInstanceIdentifier(document.title);
					break;
				}
			}
		});

		titleWatcher.observe(titleEl, {
			childList: true,
			characterData: true,
			subtree: true,
		});
	}

	overrideGetDisplayMedia();

	const onMessageHooks = {};

	// Store the session type we get (either "x11" or "wayland") into sessionType
	// This message gets sent from the onload listener in injector.js
	const onMessage = (event) => {
		if (event.target !== window) return;
		if (event.data.message === "set-session-type") {
			sessionType = event.data.type;
			window.removeEventListener("message", onMessage);
		}

		Object.values(onMessageHooks).forEach((hook) => hook(event));
	};

	window.addEventListener("message", onMessage);
})();
