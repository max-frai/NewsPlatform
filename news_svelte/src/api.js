async function FetchApi(url, incoming_options, method) {
    if (!incoming_options) {
        incoming_options = {};
    }

    if (!method) {
        method = "GET"
    }

    var default_options = {
        method,
        headers: {
            'Content-Type': 'application/json'
        }
    }

    default_options = Object.assign(default_options, incoming_options);

    url = "//" + location.hostname + "/" + url;

    return fetch(url, default_options);
}

export async function fetchTweets(period) {
    let api_resp = await FetchApi("tweets/" + period);
    return await api_resp.json();
}