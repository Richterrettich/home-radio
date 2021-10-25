
async function add_media_source() {
    let nameInput = document.getElementById("name");
    let name = nameInput.value
    let urlInput = document.getElementById("url");
    let url = urlInput.value;
    let mediaTypeSelect = document.getElementById("media_type");
    let media_type = mediaTypeSelect.options[mediaTypeSelect.selectedIndex].value;
    let source = new MediaSource(name, url, media_type);


    try {
        await put("/media", JSON.stringify(source), [{ name: "Content-Type", value: "application/json" }]).then(() => {
            // clear form if everything worked out
            nameInput.value = "";
            urlInput.value = "";
        })
    } catch (error) {
        console.log(error);
        return
    }


}