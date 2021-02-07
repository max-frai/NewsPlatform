import {
    websocketStore
} from './ws_store_wrap';

// var WS_PROTOCOL = 'wss';
// if ("__buildEnv__" == "dev") {
    var WS_PROTOCOL = 'ws';
// }

export const WsMainStore = websocketStore(WS_PROTOCOL + "://localhost:2087/ws");
export const WsInvestingStore = websocketStore("wss://stream193.forexpros.com/echo/204/fut3y0my/websocket");