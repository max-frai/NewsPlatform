const reopenTimeouts = [2000, 5000, 10000, 30000, 60000];

export function websocketStore(url, initialValue, socketOptions) {
    let socket;
    let reopenCount = 0;
    const subscriptions = new Set();
    let reopenTimeoutHandler;

    function reopenTimeout() {
        const n = reopenCount;
        reopenCount++;
        return reopenTimeouts[
            n >= reopenTimeouts.length - 1 ? reopenTimeouts.length - 1 : n
        ];
    }

    function close() {
        if (reopenTimeoutHandler) {
            clearTimeout(reopenTimeoutHandler);
        }

        if (socket) {
            socket.close();
            socket = undefined;
        }
    }

    function reopen() {
        close();
        if (subscriptions.size > 0) {
            reopenTimeoutHandler = setTimeout(() => open(), reopenTimeout());
        }
    }

    function open() {
        if (reopenTimeoutHandler) {
            clearTimeout(reopenTimeoutHandler);
            reopenTimeoutHandler = undefined;
        }

        if (socket) {
            return;
        }

        socket = new WebSocket(url, socketOptions);

        socket.onopen = event => reopenCount = 0;
        socket.onclose = event => reopen();

        socket.onmessage = event => {
            try {
                let data = event.data;
                if (data.startsWith('a["')) {
                    data = data.replace('a["', '').replace('}\\"}"]', '}').replace('::', ':').replace(/pid-(\d+):/, '');
                    data = data.replace('{\\"message\\":\\"', '');
                    data = data.replace(/\\\\\\"/gi, '"');
                }
                initialValue = JSON.parse(data);
                console.log(initialValue);
                subscriptions.forEach(subscription => subscription(initialValue));
            } catch (e) {
                subscriptions.forEach(subscription => subscription({}));
            }
        };
    }

    return {
        set(value) {
            open();
            try {
                socket.send(JSON.stringify(value));
            } catch (e) {
                console.log(e);
            }
        },
        subscribe(subscription) {
            open();
            subscription(initialValue);
            subscriptions.add(subscription);
            return () => {
                subscriptions.delete(subscription);
                if (subscriptions.size === 0) {
                    close();
                }
            };
        }
    };
}