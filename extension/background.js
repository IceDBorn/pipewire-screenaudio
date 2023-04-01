function onResponse(response) {
    let dropdown = document.getElementById('dropdown');

    let allDesktopAudioOption = document.createElement('option');
    let allDesktopAudioText = 'All Desktop Audio';
    allDesktopAudioOption.innerHTML = allDesktopAudioText;
    allDesktopAudioOption.value = allDesktopAudioText;
    dropdown.appendChild(allDesktopAudioOption);

    for (const element of response) {
        let option = document.createElement('option');
        option.innerHTML = element;
        option.value = element;
        dropdown.appendChild(option);
    }
}

function onError(error) {
    console.error(error)
}

//let sending = browser.runtime.sendNativeMessage("screenAudioMicConnector", { cmd: "StartVirtmic", args: [{ node: '' }] });
//let sending = browser.runtime.sendNativeMessage("screenAudioMicConnector", { cmd: "StopVirtmic", args: [{ micPid: 0 }] });
let sending = browser.runtime.sendNativeMessage("screenAudioMicConnector", { cmd: "GetNodes", args:[]} );
sending.then(onResponse, onError);
