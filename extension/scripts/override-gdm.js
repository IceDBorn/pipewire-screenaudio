let sessionType = null;
let iframeResizerUrl = null;
let popupUrl = null;

function overrideGdm () {
  navigator.mediaDevices.chromiumGetDisplayMedia = navigator.mediaDevices.getDisplayMedia

  const getAudioDevice = async (nameOfAudioDevice) => {
    await navigator.mediaDevices.getUserMedia({
      audio: true
    })

    // eslint-disable-next-line promise/param-names
    await new Promise(r => setTimeout(r, 1000))
    const devices = await navigator.mediaDevices.enumerateDevices()
    const audioDevice = devices.find(({
      label
    }) => label === nameOfAudioDevice)
    return audioDevice
  }

  const getDisplayMedia = async () => {
    try {
      const newPopupIframe = document.createElement('iframe');
      const popupIframeId = 'screenaudio__popup';

      newPopupIframe.src = 'about:blank';
      newPopupIframe.frameBorder = "0";
      newPopupIframe.allowTransparency="true";
      newPopupIframe.id = popupIframeId;

      let popupIframeStyle = "";
      popupIframeStyle += "width: 100vw !important;";
      popupIframeStyle += "height: 100vh !important;";
      popupIframeStyle += "top: 0 !important;";
      popupIframeStyle += "left: 0 !important;";
      popupIframeStyle += "display: block !important;";
      popupIframeStyle += "position: fixed !important;";
      popupIframeStyle += "backgroundColor: transparent !important;";
      newPopupIframe.style = popupIframeStyle;

      document.body.appendChild(newPopupIframe);

      const popupIframe = document.getElementById(popupIframeId);
      const iframeDocument = (() => {
        const content = popupIframe.contentWindow || popupIframe.contentDocument
        return content.document || content
      })();

      let bodyStyle = ""
      bodyStyle += "display: flex;"
      bodyStyle += "background-color: color-mix(in srgb, gray, transparent 70%);"
      bodyStyle += "padding: 0;"
      bodyStyle += "margin: 0;"
      iframeDocument.body.style = bodyStyle;

      const iframeResizerScript = document.createElement('script');
      iframeResizerScript.src = iframeResizerUrl;
      iframeDocument.body.appendChild(iframeResizerScript);

      const innerIframe = document.createElement('iframe');

      let innerIframeStyle = "";
      innerIframeStyle += "display: block;"
      innerIframeStyle += "margin: auto;"
      innerIframe.style = innerIframeStyle;

      innerIframe.src = popupUrl;

      iframeDocument.body.appendChild(innerIframe);
    } catch (err) {
      console.error(err)
    }

    let id
    try {
      const myDiscordAudioSink = await getAudioDevice('pipewire-screenaudio')
      id = myDiscordAudioSink.deviceId
    } catch (error) {
      return await navigator.mediaDevices.chromiumGetDisplayMedia({
        video: true,
        audio: false
      })
    }
    const captureSystemAudioStream = await navigator.mediaDevices.getUserMedia({
      audio: {
        // We add our audio constraints here, to get a list of supported constraints use navigator.mediaDevices.getSupportedConstraints();
        // We must capture a microphone, we use default since its the only deviceId that is the same for every Chromium user
        deviceId: {
          exact: id
        },
        // We want auto gain control, noise cancellation and noise suppression disabled so that our stream won't sound bad
        autoGainControl: false,
        echoCancellation: false,
        noiseSuppression: false
        // By default Chromium sets channel count for audio devices to 1, we want it to be stereo in case we find a way for Discord to accept stereo screenshare too
        // channelCount: 2,
        // You can set more audio constraints here, bellow are some examples
        // latency: 0,
        // sampleRate: 48000,
        // sampleSize: 16,
        // volume: 1.0
      }
    })
    const [track] = captureSystemAudioStream.getAudioTracks()
    let fakegdm;
    if (new RegExp('^(.+\.)?discord.com$').test(window.location.host) && sessionType === "wayland") {
      fakegdm = navigator.mediaDevices.chromiumGetDisplayMedia({
        video: true
      })
    }
    const gdm = await navigator.mediaDevices.chromiumGetDisplayMedia({
      video: true,
      audio: true
    })
    gdm.addTrack(track)
    return gdm
  }

  navigator.mediaDevices.getDisplayMedia = getDisplayMedia
}

overrideGdm()

// Store the session type we get (either "x11" or "wayland") into sessionType
// This message gets sent from the onload listener in injector.js
const onMessage = (event) => {
  if (event.target !== window)
    return;
  if (event.data.message === "set-data") {
    sessionType = event.data.sessionType
    iframeResizerUrl = event.data.iframeResizerUrl
    popupUrl = event.data.popupUrl
    window.removeEventListener("message", onMessage);
  }
};

window.addEventListener("message", onMessage);
