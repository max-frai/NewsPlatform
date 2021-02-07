<script>
  import InfoBlock from "./components/InfoBlock.svelte";
  import { _ } from "svelte-i18n";
  import Stocks from "./components/Blocks/Stocks.svelte";
  // import Weather from "./Blocks/Weather.svelte";
  import Air from "./components/Blocks/Air.svelte";
  // import Ato from "./Blocks/Ato.svelte";
  // import CO from "./Blocks/CO.svelte";
  // import Traffic from "./Blocks/Traffic.svelte";
  // import Radiation from "./Blocks/Radiation.svelte";
  import SmartScroll from "./components/SmartScroll.svelte";

  import { WsMainStore, WsInvestingStore } from "./ws_store.js";

  let chartBlocks = [
    // {
    //   id: "39",
    //   title: $_("charts.weather"),
    //   type: Weather,
    // },
    {
      id: "42",
      title: $_("charts.air"),
      type: Air,
    },
    // {
    //   id: "44",
    //   title: $_("charts.co"),
    //   type: CO
    // },
    // {
    //   id: "43",
    //   title: $_("charts.radiation"),
    //   type: Radiation,
    // },
    // {
    //   id: "45",
    //   title: $_("charts.ato"),
    //   type: Ato
    // },
    // {
    //   id: "46",
    //   title: $_("charts.traffic"),
    //   type: Traffic,
    // },
    {
      id: "40",
      title: $_("charts.uahblack"),
      type: Stocks,
    },
    {
      id: "2208",
      title: $_("charts.uah"),
      type: Stocks,
    },
    {
      id: "41",
      title: $_("charts.fuel95"),
      type: Stocks,
    },
    {
      id: "47",
      title: $_("charts.fueldp"),
      type: Stocks,
    },
    {
      id: "8833",
      title: $_("charts.brent"),
      type: Stocks,
    },
    {
      id: "2186",
      title: $_("charts.rub"),
      type: Stocks,
    },
    {
      id: "945629",
      title: $_("charts.btc"),
      type: Stocks,
    },
    {
      id: "8830",
      title: $_("charts.gold"),
      type: Stocks,
    },
    {
      id: "1",
      title: $_("charts.eur"),
      type: Stocks,
    },
    {
      id: "169",
      title: $_("charts.dow"),
      type: Stocks,
    },
    {
      id: "166",
      title: $_("charts.sp"),
      type: Stocks,
    },
  ];

  $: $WsMainStore;
  $: updateLiveInvesting($WsInvestingStore);

  function jsonData(chartId) {
    if ($WsInvestingStore == null || $WsInvestingStore[chartId] == null) {
      return {};
    } else {
      return $WsInvestingStore[chartId];
    }
  }

  function updateLiveInvesting(wsData) {
    if (
      wsData &&
      Object.keys(wsData).length === 0 &&
      wsData.constructor === Object
    ) {
      window.setTimeout(() => {
        let pairs = [8833, 8830, 169, 945629, 2208, 1];

        let subscribeCmd = '{"_event":"bulk-subscribe","tzID":8,"message":"';
        for (let i = 0; i < pairs.length; i++) {
          subscribeCmd += `isOpenPair-${pairs[i]}:%%pid-${pairs[i]}:%%`;
        }
        subscribeCmd += '"}';

        subscribeCmd = subscribeCmd.replace(':%%"}', ':"}');
        // console.log(subscribeCmd);

        function hbInvesting() {
          let hb = '["{"_event":"heartbeat","data":"h"}"]';
          $WsInvestingStore = hb;
        }

        $WsInvestingStore = [subscribeCmd];
        hbInvesting();
        window.setInterval(() => {
          hbInvesting();
        }, 1000 * 2);
      }, 1000);
    }
  }
</script>

<div class="bg-gray-200 rounded-lg">
  <div class="InfoBlockWrap mx-auto md:justify-center">
    <SmartScroll classes="py-2">
      {#each chartBlocks as block}
        <InfoBlock {...block} />
      {/each}
    </SmartScroll>
  </div>
</div>

<style type="text/postcss">
  @import "tailwindcss/components";
  @import "tailwindcss/utilities";
  .InfoBlockWrap {
    @apply flex relative w-full;
  }
</style>
