<script>
    import { WsMainStore } from "./ws_store.js";
    import { cardUrl } from "./utils.js";
    import { tweened } from "svelte/motion";
    import { linear } from "svelte/easing";
    import { tick } from "svelte";

    let threads = [];
    let marquee = null;
    let threadContainer = null;

    $: updateNews($WsMainStore);
    $: startAnimation(threadContainer);

    let translateX = 0;
    let transitionDuration = 5000;

    const translateProgress = tweened(0, {
        duration: 800,
        easing: linear,
    });

    function swapSibling(node1, node2) {
        node1.parentNode.replaceChild(node1, node2);
        node1.parentNode.insertBefore(node2, node1);
    }

    function startAnimation(container) {
        if (!container) {
            console.log("Container is null, return startAnimation runningLine");
            return;
        }

        let firstElement = container.firstElementChild;
        let lastElement = container.lastElementChild;
        let rect = firstElement.getBoundingClientRect();
        let width = rect.width;

        translateX = -Math.round(width);
        transitionDuration = Math.round(width * 18);

        window.setTimeout(async () => {
            swapSibling(firstElement, lastElement);
            transitionDuration = 0;
            translateX = 0;

            await tick();
            startAnimation(container);
        }, transitionDuration);
    }

    function updateNews(jsonData) {
        if (!jsonData) return;
        if (jsonData.kind != "MostRecentClusterMessage") return;

        jsonData = JSON.parse(jsonData.data);
        threads = jsonData.cluster.threads;
    }

    function stopMarquee() {
        // marquee.stop();
    }
    function startMarquee() {
        // marquee.start();
    }
</script>

{#if threads.length >= 3}
    <div class="Wrap py-2">
        <div class="WhiteShadeoutLeft" />
        <div
            class="transition-all"
            style="transition-timing-function: linear; transition-duration: {transitionDuration}ms;
        transform: translateX({translateX}px)"
            bind:this={marquee}
            on:mouseenter={stopMarquee}
            on:mouseleave={startMarquee}
        >
            <div class="FavoritedMoving" bind:this={threadContainer}>
                {#each threads as thread}
                    <div class="flex-none ">
                        <a href={cardUrl(thread.main_item)}>
                            {thread.main_item.title}
                        </a>
                        &nbsp;&nbsp;
                        <span class="opacity-25">•</span>
                        &nbsp;&nbsp;&nbsp;
                    </div>
                {/each}
            </div>
        </div>
        <div class="WhiteShadeoutRight" />
    </div>
{:else}
    <div class="mb-5" />
{/if}

<style type="text/postcss">
    .Wrap {
        @apply bg-white text-sm w-full tracking-tight py-1 overflow-hidden relative;
    }

    .FavoritedMoving {
        @apply flex;
    }

    .WhiteShadeoutRight {
        z-index: 10;
        @apply absolute right-0 top-0 h-full;
        box-shadow: 0px 0 12px 12px white;
    }
    .WhiteShadeoutLeft {
        z-index: 10;
        @apply absolute left-0 top-0 h-full;
        box-shadow: 0px 0 12px 12px white;
    }
</style>
