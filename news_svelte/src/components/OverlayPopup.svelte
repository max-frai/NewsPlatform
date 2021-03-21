<script>
    export let is_visible = false;
</script>

{#if is_visible}
    <div
        class="Overlay"
        on:click|preventDefault|self={() => (is_visible = false)}
    >
        <div class="container OverlayContent md:flex">
            <div
                class="CloseWindow"
                on:click|self|preventDefault={() => (is_visible = false)}
            >
                закрыть
            </div>
            <slot />
        </div>
    </div>
{/if}

<style type="text/postcss">
    .Overlay {
        @apply fixed w-full h-full bg-black left-0 top-0 bottom-0 right-0 z-50;
        background-color: rgba(0, 0, 0, 0.4);
        overflow-y: scroll;
        overscroll-behavior: contain;
    }
    .OverlayContent {
        @apply rounded-lg shadow-lg my-10 bg-white  mx-auto p-1  border border-gray-400 relative z-50;
        overscroll-behavior: contain;
        max-width: 1024px;
    }
    .OverlayContent img {
        max-width: 100%;
    }
    .CloseWindow {
        @apply absolute right-0 top-0 p-2 px-5 mr-3 mt-3 font-bold text-red-400 cursor-pointer rounded-full z-50;
    }
    .CloseWindow:hover {
        @apply bg-gray-300;
    }
    #ArticleContent :global(iframe),
    #ArticleContent :global(img) {
        max-width: 100% !important;
    }
</style>
