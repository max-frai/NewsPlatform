<script>
    import CovidTimeGraph from "./components/CovidTimeGraph.svelte";
    // import CovidInfoSummary from "./components/CovidInfoSummary.svelte";
</script>

<div class="GrayBorder overflow-hidden">
    <div class="w-full md:w-2/3 md:mt-0 relative flex">
        <CovidTimeGraph />
    </div>
    <!-- <div class="border-t border-gray-300 bg-gray-100 px-0 md:px-3 py-1 md:py-2">
        <CovidInfoSummary />
    </div> -->
</div>
