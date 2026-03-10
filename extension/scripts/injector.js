const MESSAGE_NAME = "com.icedborn.pipewirescreenaudioconnector";

const nullthrows = (v) => {
	if (v == null) throw new Error("null");
	return v;
};

function injectCode(src) {
	const script = document.createElement("script");
	script.src = src;
	script.onload = function () {
		console.log("pipewire-screenaudio script injected");

		chrome.runtime
			.sendMessage({ messageName: MESSAGE_NAME, message: "get-session-type" })
			.then(({ type }) => {
				window.postMessage({ message: "set-session-type", type });
			});

		this.remove();
	};

	nullthrows(document.head || document.documentElement).appendChild(script);
}

window.addEventListener("message", ({ data }) => {
	if (data.message === "instance-identifier") {
		chrome.runtime.sendMessage({
			messageName: MESSAGE_NAME,
			message: data.message,
			instanceIdentifier: data.instanceIdentifier,
		});
	}
});

injectCode(chrome.runtime.getURL("/scripts/override-gdm.js"));
