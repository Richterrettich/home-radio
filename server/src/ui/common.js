
function put(path, body, headers = []) {
    return makeRequest("PUT", path, body, headers)

}
function post(path, body, headers = []) {
    return makeRequest("POST", path, body, headers)
}

function get(path, headers = []) {
    return makeRequest("GET", path, headers)
}

function makeRequest(method, url, body, headers = []) {
    return new Promise(function (resolve, reject) {
        let xhr = new XMLHttpRequest();
        xhr.open(method, url);

        headers.forEach(header => {
            xhr.setRequestHeader(header.name, header.value);
        });

        xhr.onload = function () {
            if (this.status >= 200 && this.status < 300) {
                resolve(xhr.response);
            } else {
                reject({
                    status: this.status,
                    statusText: xhr.statusText
                });
            }
        };
        xhr.onerror = function () {
            reject({
                status: this.status,
                statusText: xhr.statusText
            });
        };
        xhr.send(body);
    });
}

class MediaSource {
    constructor(name, link, media_type) {
        this.name = name;
        this.link = link;
        this.media_type = media_type;
        this.default_source = false;
    }
}
