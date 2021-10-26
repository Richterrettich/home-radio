var isPlaying = false;

window.addEventListener("load", async function () {
    console.log("loading media");
    result = await get("/media");
    media = JSON.parse(result);
    let default_source = null;
    let currently_playing = null;
    media.forEach(function (item) {
        console.log(item);
        element = document.createElement("option");
        element.value = item.link;
        element.appendChild(document.createTextNode(item.name));
        document.getElementById("radio_links").appendChild(element);
        if (item.default_source) {
            default_source = item;
            if (currently_playing == null) {
                element.selected = true;
            }
        }
        if (item.currently_playing) {
            currently_playing = item;
            element.selected = true;
            isPlaying = true;
            switchButtonState(isPlaying);
        }
    });

    result = await get("/volume");
    console.log("setting volume to", result);
    volumeSlider = document.getElementById("volume");
    volumeSlider.value = result;
    volumeSlider.addEventListener('change', async function () {
        await put("/volume", volumeSlider.value);
    });
});


async function handleMedia() {
    if (isPlaying) {
        await stop()
    } else {
        await start()
    }
}

function switchButtonState(isPlaying) {
    let button = document.getElementById("playback_button");
    button.textContent = isPlaying ? "stop" : "start";
}

async function start() {
    radioUrlsSelect = document.getElementById("radio_links");
    url = radioUrlsSelect.options[radioUrlsSelect.selectedIndex].value;
    await post("/start", url);
    isPlaying = true;
    switchButtonState(isPlaying);
}


async function stop() {
    await post("/stop");
    isPlaying = false;
    switchButtonState(isPlaying);
}

function increaseVolume() {
    post("/increase_volume", 10)
}

function decreaseVolume() {
    post("/decrease_volume", 10)
}

