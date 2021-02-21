<script>
    import CountUp from "countup";
    import { onMount } from "svelte";
    import { CovidStore } from "../covid_store.js";
    import { formatInterval } from "../utils.js";
    let globalId = Math.random().toString().replace(".", "");
    export let predictValues = false;
    export let title = "";
    export let confirmed = 0;
    export let recovered = 0;
    export let deaths = 0;
    export let deathsRate = null;
    function startConfirmed() {
        let interval = 1000;
        if ($CovidStore) interval = $CovidStore.confirmedInterval;
        window.setTimeout(() => {
            confirmed += 1;
            startConfirmed();
        }, interval);
    }
    function startRecovered() {
        let interval = 1000;
        if ($CovidStore) interval = $CovidStore.recoveredInterval;
        window.setTimeout(() => {
            recovered += 1;
            startRecovered();
        }, interval);
    }
    if (predictValues) {
        startConfirmed();
        startRecovered();
    }
    let confirmedCount = null;
    let recoveredCount = null;
    let deathsCount = null;
    $: updateAnimated(confirmed, confirmedCount);
    $: updateAnimated(recovered, recoveredCount);
    $: updateAnimated(deaths, deathsCount);
    onMount(async () => {
        confirmedCount = new CountUp("confirmedId-" + globalId, confirmed);
        recoveredCount = new CountUp("recoveredId-" + globalId, recovered);
        deathsCount = new CountUp("deathsId-" + globalId, deaths);
    });
    function updateAnimated(newData, handle) {
        if (!handle) return;
        handle.update(newData);
    }
</script>

<div class="absolute left-0 w-64 h-24">
    <!-- <div class="maintitle">{title}</div> -->

    <div class="body">
        <div class="summaryWrap">
            <div class="title">Случаев</div>
            <div class="value">
                <span id="confirmedId-{globalId}">0</span>
            </div>
        </div>
        <div class="summaryWrap">
            <div class="title">Восстановились</div>
            <div class="value ">
                <span id="recoveredId-{globalId}">0</span>
            </div>
        </div>

        <div class="summaryWrap">
            <div class="title">Смертей</div>
            <div class="value ">
                <span id="deathsId-{globalId}">0</span>
            </div>
        </div>
    </div>
</div>

<style type="text/postcss">
    .maintitle {
        @apply text-sm font-medium text-gray-700 mb-1;
    }
    .title {
        @apply text-xs align-middle text-gray-600;
    }
    .value {
        @apply font-medium text-xs  align-middle pl-1;
    }
    @screen md {
        .title {
            @apply w-2/4;
        }
        .value {
            @apply w-2/4 text-sm;
        }
    }
    .body {
        /* @apply ml-5; */
    }
    .summaryWrap {
        @apply flex w-full content-center items-center;
    }
    sup {
        @apply text-xs opacity-75;
    }
</style>
