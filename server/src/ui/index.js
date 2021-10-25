window.addEventListener("load", async function () {
    console.log("loading media");
    result = await get("/media");
    media = JSON.parse(result);
    media.forEach(function (item) {
        console.log(item);
        element = document.createElement("option");
        element.value = item.link;
        element.appendChild(document.createTextNode(item.name));
        document.getElementById("radio_links").appendChild(element);
    });

    result = await get("/volume");
    console.log("setting volume to", result);
    volumeSlider = document.getElementById("volume");
    volumeSlider.value = result;
    volumeSlider.addEventListener('change', async function () {
        await put("/volume", volumeSlider.value);
    });
});



function start() {
    radioUrlsSelect = document.getElementById("radio_links");
    url = radioUrlsSelect.options[radioUrlsSelect.selectedIndex].value;
    post("/start", url);
}


function stop() {
    post("/stop");
}

function increaseVolume() {
    post("/increase_volume", 10)
}

function decreaseVolume() {
    post("/decrease_volume", 10)
}

