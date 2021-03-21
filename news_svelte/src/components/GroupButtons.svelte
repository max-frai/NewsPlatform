<script>
    import { createEventDispatcher } from "svelte";
    export let buttons = [];
    export let active = "";
    export let title = "";
    const dispatch = createEventDispatcher();
    $: buttonsChanged(buttons);
    function buttonsChanged(buttons) {
        if (active == "" && buttons.length) {
            active = buttons[0];
        }
    }
    function changed(button) {
        dispatch("activeChanged", {
            active: button,
        });
        active = button;
    }
</script>

<div class="mainButtonWrap">
    {#each buttons as button}
        <div
            class="mainButton"
            on:click={() => changed(button)}
            class:mainButtonActive={button == active}
        >
            {button}
        </div>
    {/each}
    <div class="Title">{title}</div>
</div>

<style type="text/postcss">
    .GrayButtons {
        @apply inline-flex font-medium text-gray-500 text-sm;
    }
    .GrayButtons .GrayButton {
        @apply px-3 py-1 cursor-pointer;
    }
    .GrayButtons .GrayButton:hover {
        @apply bg-gray-300 text-black rounded-full;
    }
    .GrayButtons .Active {
        @apply bg-gray-300 text-black rounded-full;
    }
    .GrayButtons .Title {
        float: right;
    }
</style>
