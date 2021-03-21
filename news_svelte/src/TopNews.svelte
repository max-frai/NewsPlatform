<script>
  import { WsMainStore } from "./ws_store.js";
  import { cardUrl, crc32 } from "./utils.js";
  import SmartScroll from "./components/SmartScroll.svelte";
  import SvgFilters from "./components/SvgFilters.svelte";
  import { _ } from "svelte-i18n";
  let clusters = [];
  let possibleNews = ["Важные", "Последние"];
  let filters = [
    "duotone-one",
    "duotone-two",
    "duotone-three",
    "duotone-five",
    "duotone-six",
  ];
  let filters2bg = {
    "duotone-one": "248, 227, 172",
    "duotone-two": "242, 167, 147",
    "duotone-three": "188, 190, 225",
    "duotone-five": "165, 211, 214",
    "duotone-six": "218, 188, 225",
  };
  let filters2overlay = {
    "duotone-one":
      "opacity: 0.8; background: linear-gradient(rgba(10, 63, 124, 0), rgb(8, 19, 113));",
    "duotone-two":
      "opacity: 0.6; background: linear-gradient(rgba(0, 0, 0, 0), rgb(61, 10, 112));",
    "duotone-three":
      "opacity: 0.7; background: linear-gradient(rgba(130, 0, 133, 0), rgb(130, 0, 133));",
    "duotone-five":
      "opacity: 0.6; background: linear-gradient(rgba(76, 22, 144, 0), rgb(76, 22, 145));",
    "duotone-six":
      "opacity: 0.6; background: linear-gradient(rgba(20, 62, 143, 0), rgb(71, 76, 203));",
  };
  function categoryTitle(cluster) {
    return $_("topnews." + cluster.category);
  }
  $: updateNews($WsMainStore);
  function updateNews(jsonData) {
    if (!jsonData) return;
    if (jsonData.kind != "PopularClusterMessage") return;
    jsonData = JSON.parse(jsonData.data);
    clusters = jsonData.clusters;
    for (var i = 0; i < clusters.length; i++) {
      let threads = clusters[i].threads;
      for (var j = 0; j < threads.length; ++j) {
        let thread = threads[j];
        let hash = crc32(thread.title);
        thread.filter = filters[hash % filters.length];
      }
    }
  }
</script>

<SvgFilters />

{#each clusters as cluster}
  <h2 class="BlockTitle">{categoryTitle(cluster)}</h2>
  <div class="TopNewsWrap GrayBorder">
    <SmartScroll classes="py-3 px-3" scroll_koef={0.03}>
      {#each cluster.threads as thread}
        <a class="link linkWidth" href={cardUrl(thread.main_item)}>
          <div
            class="item "
            style="background-color: rgb({filters2bg[
              thread.filter
            ]}); filter: url(#{thread.filter});"
          >
            <div
              class="bg-cover w-full h-full"
              style="background-image: url({thread.main_item.og_image})"
            />
          </div>
          <div
            class="absolute w-full h-full left-0 top-0 right-0 bottom-0
              rounded-lg"
            style={filters2overlay[thread.filter]}
          />
          <div class="title pl-4 md:pl-5 pr-4 md:pr-8">
            {@html thread.title}
          </div>
        </a>
      {/each}
    </SmartScroll>
  </div>
{/each}

<style type="text/postcss">
  .TopNewsWrap {
    @apply w-full  relative;
  }
  .TopNewsWrap .link {
    @apply flex-shrink-0 mr-2 relative;
  }
  .linkWidth {
    @apply w-2/3;
  }
  @screen md {
    .linkWidth {
      @apply w-5/12;
    }
  }
  .TopNewsWrap .item {
    @apply w-full h-full inline-block bg-white overflow-hidden rounded-lg relative cursor-pointer relative;
    min-height: 180px;
  }
  .TopNewsWrap .body {
    @apply px-3;
  }
  .TopNewsWrap .title {
    @apply text-base pb-5  font-bold leading-tight bottom-0 left-0 absolute text-white;
  }
</style>
