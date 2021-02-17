import {
    websocketStore
} from './ws_store_wrap';


var host = location.hostname;
var WS_PROTOCOL = 'ws';
if (host != 'localhost') {
    WS_PROTOCOL = 'wss';
}

export const WsMainStore = websocketStore(WS_PROTOCOL + "://" + host + ":2087/ws");
export const WsInvestingStore = websocketStore("wss://stream193.forexpros.com/echo/204/fut3y0my/websocket");