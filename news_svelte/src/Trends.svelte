<script>
    import { WsMainStore } from "./ws_store.js";
    let trends = [];
    $: updateTrends($WsMainStore);
    function updateTrends(jsonData) {
        if (!jsonData) return;
        if (jsonData.kind != "TodayTrendsMessage") return;
        jsonData = JSON.parse(jsonData.data);
        trends = jsonData.trends;
        // console.log(trends);
    }
</script>

<h1 class="BlockTitle">Тренды на сегодня</h1>
<div class="GrayBorder p-1 md:p-3 flex flex-wrap text-xs text-center">
    {#each trends as trend}
        <div
            class="mr-1 flex-grow bg-gray-100 rounded-md px-1 md:px-3 py-1 mb-1
        capitalize"
        >
            {trend[0]}
            <span class="text-gray-500">{trend[1]}</span>
        </div>
    {/each}
</div>

<style type="text/postcss">
</style>
