<script>
    import { onMount } from "svelte";
    import { tweened } from "svelte/motion";
    import { cubicOut } from "svelte/easing";

    export let classes = "";
    export let scroll_koef = 0.007;

    let showLeftShadow = false;
    let showRightShadow = true;
    let scrollHandle = null;
    let scrollStep = 100;
    let scrollDirection = +1;

    let progressStore = null;

    $: animatedScroll($progressStore);

    function animatedScroll(progressValue) {
        if (!scrollHandle || !progressValue) return;

        scrollHandle.scrollLeft =
            scrollHandle.scrollLeft +
            scrollStep * progressValue * scrollDirection;

        if (progressValue >= 1) progressStore = null;
    }

    function shadowLogic() {
        if (!scrollHandle) return;

        showLeftShadow = scrollHandle.scrollLeft != 0;
        showRightShadow =
            scrollHandle.scrollLeft + scrollHandle.clientWidth <
            scrollHandle.scrollWidth - 20;
    }

    function scroll(direction) {
        progressStore = tweened(0, {
            duration: 330,
            easing: cubicOut,
        });
        scrollDirection = direction;
        progressStore.set(1);
    }

    onMount(async () => {
        shadowLogic();
        scrollStep = scrollHandle.scrollWidth * scroll_koef;
        scrollHandle.onscroll = function (ev) {
            shadowLogic();
        };
    });
</script>

{#if showLeftShadow}
    <div class="ScrollMoreShadowLeft" />
    <div class="ScrollArrowWrap left-0 ">
        <div class="ScrollArrow ScrollArrowWL" on:click={() => scroll(-1)}>
            <svg fill="#000" width="12" height="12">
                <path
                    fill-rule="evenodd"
                    d="M8.121.343L2.464 6l5.657 5.657 1.415-1.414L5.293 6l4.243-4.243z"
                />
            </svg>
        </div>
    </div>
{/if}
<div class="ScrollArea {classes}" bind:this={scrollHandle}>
    <slot />
</div>
{#if showRightShadow}
    <div class="ScrollMoreShadowRight" />
    <div class="ScrollArrowWrap right-0 ">
        <div class="ScrollArrow ScrollArrowWR" on:click={() => scroll(+1)}>
            <svg fill="#000" width="12" height="12">
                <path
                    fill-rule="evenodd"
                    d="M3.879 11.657L9.536 6 3.879.343 2.464 1.757 6.707 6l-4.243 4.243z"
                />
            </svg>
        </div>
    </div>
{/if}

<style type="text/postcss">
    .ScrollArea {
        @apply overflow-x-scroll relative flex w-full;
        -ms-overflow-style: none;
        scrollbar-width: none;
    }

    .ScrollArea::-webkit-scrollbar {
        display: none;
    }

    .ScrollMoreShadowRight {
        @apply absolute right-0 top-0 h-full;
        width: 20px;
        right: -20px;
        box-shadow: -8px 0 12px -7px rgba(0, 0, 0, 0.6);
        z-index: 10;
    }
    .ScrollMoreShadowLeft {
        @apply absolute left-0 top-0 h-full;
        width: 20px;
        left: -20px;
        box-shadow: 8px 0 12px -7px rgba(0, 0, 0, 0.6);
        z-index: 10;
    }
    .ScrollArrowWrap {
        @apply absolute top-0 z-10 h-full flex justify-center items-center;
    }

    .ScrollArrow {
        @apply flex justify-center w-10 h-10  rounded-full bg-white  shadow items-center cursor-pointer transition-all duration-300;
    }

    .ScrollArrow:hover {
        @apply shadow-lg;
    }

    .ScrollArrowWL:hover {
        @apply -ml-6;
    }
    .ScrollArrowWL {
        @apply -ml-5;
    }

    .ScrollArrowWR:hover {
        @apply -mr-6;
    }
    .ScrollArrowWR {
        @apply -mr-5;
    }
</style>
