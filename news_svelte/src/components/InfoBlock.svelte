<script>
    // import { WsMainStore, WsInvestingStore } from "../ws_store.js";
    // import Stocks from "./Blocks/Stocks.svelte";
    // import Weather from "./components/Blocks/Weather.svelte";
    import Empty from "./Blocks/Empty.svelte";

    export let title = "";
    export let id = -1;
    export let type = Empty;

    let diffBgClass = "bg-white";
    let borderClass = "border-gray-300";
    function bgChanged(event) {
        diffBgClass = event.detail.class;
        borderClass = event.detail.borderClass;
    }
</script>

<div
    class="InfoBlock {diffBgClass} px-2 md:px-3 ml-1 first:ml-3 last:mr-1 GrayBorder {borderClass}
    flex-shrink-0"
>
    <div class="title">{title}</div>
    <div class="body">
        <svelte:component this={type} {id} on:newbg={bgChanged} />
    </div>
</div>

<style type="text/postcss">
    .InfoBlock {
        @apply inline-block py-1 border text-sm rounded-lg cursor-pointer transition-colors duration-500;
    }

    .InfoBlock .title {
        @apply font-medium text-xs opacity-50;
    }
</style>
