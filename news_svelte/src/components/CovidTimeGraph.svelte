<script>
  import CovidGraphSummary from "./CovidGraphSummary.svelte";

  import { watchResize } from "svelte-watch-resize";

  import { WsMainStore } from "../ws_store.js";
  import { CovidStore } from "../covid_store.js";

  $: updateGraph($WsMainStore);

  let svgWidth = 0;
  let svgHeight = 0;

  // SVG GRAPH ----
  let confirmedAllRects = [];
  let confirmedAllDiffRects = [];

  let recoveredAllRects = [];
  let recoveredAllDiffRects = [];

  let deathsAllRects = [];
  let deathsAllDiffRects = [];

  let confirmedRects = [];
  let confirmedDiffRects = [];

  let recoveredRects = [];
  let recoveredDiffRects = [];

  let deathsRects = [];
  let deathsDiffRects = [];
  // ----------
  let confirmed = [];
  let confirmedDiff = [];

  let recovered = [];
  let recoveredDiff = [];

  let deaths = [];
  let deathsDiff = [];

  let confirmedAll = [];
  let confirmedAllDiff = [];

  let recoveredAll = [];
  let recoveredAllDiff = [];

  let deathsAll = [];
  let deathsAllDiff = [];
  // ---------

  let diffConfirmedUa = 0;
  let diffRecoveredUa = 0;
  let diffDeathsUa = 0;

  let totalConfirmedUa = 0;
  let totalRecoveredUa = 0;
  let totalDeathsUa = 0;

  let totalConfirmedAll = 0;
  let totalRecoveredAll = 0;
  let totalDeathsAll = 0;

  let diffConfirmedAll = 0;
  let diffRecoveredAll = 0;
  let diffDeathsAll = 0;

  let timestamps = [];

  let chartColors = {
    red: "rgb(235, 23, 68)",
    orange: "rgb(255, 159, 64)",
    yellow: "rgb(240, 185, 10)",
    green: "rgb(37,184,100)",
    blue: "rgb(54, 162, 235)",
    purple: "rgb(153, 102, 255)",
    grey: "rgb(201, 203, 207)",
  };

  function updateGraph(jsonData) {
    if (!jsonData) return;
    if (jsonData.kind != "CovidTimeMessage") return;
    jsonData = JSON.parse(jsonData.data);

    // console.log(jsonData);

    timestamps = [];

    confirmed = [];
    confirmedDiff = [];
    confirmedRects = [];
    confirmedDiffRects = [];

    recovered = [];
    recoveredDiff = [];
    recoveredRects = [];
    recoveredDiffRects = [];

    deaths = [];
    deathsDiff = [];
    deathsRects = [];
    deathsDiffRects = [];

    confirmedAll = [];
    confirmedAllRects = [];
    confirmedAllDiff = [];
    confirmedAllDiffRects = [];

    recoveredAll = [];
    recoveredAllRects = [];
    recoveredAllDiff = [];
    recoveredAllDiffRects = [];

    deathsAll = [];
    deathsAllRects = [];
    deathsAllDiff = [];
    deathsAllDiffRects = [];

    for (var i = 0; i < jsonData.confirmed_points.length; i++) {
      let timestamp = jsonData.confirmed_points[i][1];
      let time = new Date(timestamp * 1000);
      timestamps.push(`${time.getDate()}.${time.getMonth() + 1}`);

      confirmed.push(jsonData.confirmed_points[i][0]);
      confirmedDiff.push(
        jsonData.confirmed_points[i][0] -
          jsonData.confirmed_points[i == 0 ? 0 : i - 1][0]
      );

      deaths.push(jsonData.deaths_points[i][0]);
      deathsDiff.push(
        jsonData.deaths_points[i][0] -
          jsonData.deaths_points[i == 0 ? 0 : i - 1][0]
      );

      let recoveredIndex = i;
      if (jsonData.recovered_points[i]) {
        recovered.push(jsonData.recovered_points[i][0]);
      } else if (jsonData.recovered_points[i - 1]) {
        recoveredIndex = i - 1;
        recovered.push(jsonData.recovered_points[i - 1][0]);
      }

      // recoveredDiff.push(
      //   jsonData.recovered_points[i][0] -
      //     jsonData.recovered_points[
      //       i == 0 ? recoveredIndex : recoveredIndex - 1
      //     ][0]
      // );

      confirmedAll.push(jsonData.confirmed_points_all[i][0]);
      confirmedAllDiff.push(
        jsonData.confirmed_points_all[i][0] -
          jsonData.confirmed_points_all[i == 0 ? 0 : i - 1][0]
      );

      deathsAll.push(jsonData.deaths_points_all[i][0]);
      deathsAllDiff.push(
        jsonData.deaths_points_all[i][0] -
          jsonData.deaths_points_all[i == 0 ? 0 : i - 1][0]
      );

      if (jsonData.recovered_points_all[i])
        recoveredAll.push(jsonData.recovered_points_all[i][0]);
      else if (jsonData.recovered_points_all[i - 1])
        recoveredAll.push(jsonData.recovered_points_all[i - 1][0]);

      if (jsonData.recovered_points_all[i]) {
        recoveredAllDiff.push(
          jsonData.recovered_points_all[i][0] -
            jsonData.recovered_points_all[i == 0 ? 0 : i - 1][0]
        );
      } else {
        recoveredAllDiff.push(0);
      }
    }

    // console.log(recoveredAll);

    totalConfirmedUa = jsonData.confirmed_ua;
    totalRecoveredUa = jsonData.recovered_ua;
    totalDeathsUa = jsonData.deaths_ua;

    totalConfirmedAll = jsonData.confirmed_all;
    totalRecoveredAll = jsonData.recovered_all;
    totalDeathsAll = jsonData.deaths_all;

    // Calculate koefs ----
    CovidStore.confirmedDiff(
      confirmedAll[confirmedAll.length - 1] -
        confirmedAll[confirmedAll.length - 2],
      confirmedAll[confirmedAll.length - 2] -
        confirmedAll[confirmedAll.length - 3]
    );

    CovidStore.deathsDiff(
      deathsAll[deathsAll.length - 1] - deathsAll[deathsAll.length - 2],
      deathsAll[deathsAll.length - 2] - deathsAll[deathsAll.length - 3]
    );

    CovidStore.recoveredDiff(
      recoveredAll[recoveredAll.length - 1] -
        recoveredAll[recoveredAll.length - 2],
      recoveredAll[recoveredAll.length - 2] -
        recoveredAll[recoveredAll.length - 3]
    );

    CovidStore.deathRateAll(totalConfirmedAll, totalDeathsAll);
    CovidStore.deathRateUa(totalConfirmedUa, totalDeathsUa);
    // ----------

    // console.log("totalConfirmedUa:");
    // console.log(totalConfirmedUa);

    // console.log(confirmed[confirmed.length - 1]);
    // console.log(confirmed[confirmed.length - 2]);

    diffConfirmedUa = confirmedDiff[confirmedDiff.length - 1];
    // diffConfirmedUa = totalConfirmedUa - confirmed[confirmed.length - 1];
    diffRecoveredUa = totalRecoveredUa - recovered[recovered.length - 1];
    diffDeathsUa = totalDeathsUa - deaths[deaths.length - 1];

    diffConfirmedAll =
      totalConfirmedAll - confirmedAll[confirmedAll.length - 1];
    diffRecoveredAll =
      totalRecoveredAll - recoveredAll[recoveredAll.length - 1];
    diffDeathsAll = totalDeathsAll - deathsAll[deathsAll.length - 1];

    renderGraph();
  }

  function renderGraph() {
    function fillBars(data, maxYValue, fill) {
      let barWidth = svgWidth / data.length;

      let container = [];
      let diffContainer = [];

      for (var i = 0; i < data.length; i++) {
        let height = (data[i] * svgHeight) / maxYValue;
        container.push({
          x: i * barWidth + barWidth,
          y: svgHeight - height,
          width: barWidth,
          height,
          fill,
        });
      }

      return container;
    }

    // --------------

    let maxConfirmedAll = Math.max(...confirmedAll);
    let maxConfirmedAllDiff = Math.max(...confirmedAllDiff);

    confirmedAllRects = fillBars(
      confirmedAll,
      maxConfirmedAll,
      "rgb(255, 223, 138)"
    );
    recoveredAllRects = fillBars(
      recoveredAll,
      maxConfirmedAll,
      "rgb(91, 217, 145)"
    );
    deathsAllRects = fillBars(deathsAll, maxConfirmedAll, chartColors.red);

    confirmedAllDiffRects = fillBars(
      confirmedAllDiff,
      maxConfirmedAllDiff,
      "rgb(255, 223, 138)"
    );
    deathsAllDiffRects = fillBars(
      deathsAllDiff,
      maxConfirmedAllDiff,
      chartColors.red
    );
    recoveredAllDiffRects = fillBars(
      recoveredAllDiff,
      maxConfirmedAllDiff,
      "rgb(91, 217, 145)"
    );

    // --------------

    let maxConfirmed = Math.max(...confirmed);
    let maxConfirmedDiff = Math.max(...confirmedDiff);

    confirmedRects = fillBars(confirmed, maxConfirmed, "rgb(255, 223, 138)");
    recoveredRects = fillBars(recovered, maxConfirmed, "rgb(91, 217, 145)");
    deathsRects = fillBars(deaths, maxConfirmed, chartColors.red);

    confirmedDiffRects = fillBars(
      confirmedDiff,
      maxConfirmedDiff,
      "rgb(255, 223, 138)"
    );
    deathsDiffRects = fillBars(deathsDiff, maxConfirmedDiff, chartColors.red);
    recoveredDiffRects = fillBars(
      recoveredDiff,
      maxConfirmedDiff,
      "rgb(91, 217, 145)"
    );
  }
</script>

<div class="w-full" use:watchResize={renderGraph}>
  <!-- <div
    class="GraphWrap GraphWrapHeight"
    bind:clientWidth={svgWidth}
    bind:clientHeight={svgHeight}>
    <CovidGraphSummary
      title="Весь мир  — всего"
      predictValues={true}
      confirmed={totalConfirmedAll}
      deaths={totalDeathsAll}
      recovered={totalRecoveredAll}
      deathsRate={$CovidStore.deathsRateAll} />
    <svg width="100%" height="100%">
      {#each [...Array(timestamps.length).keys()] as index}
        <rect {...confirmedAllRects[index]} />
        <rect {...recoveredAllRects[index]} />
        <rect {...deathsAllRects[index]} />
      {/each}
    </svg>
  </div> -->

  <div
    class="GraphWrap GraphWrapHeight"
    bind:clientWidth={svgWidth}
    bind:clientHeight={svgHeight}
  >
    <CovidGraphSummary
      title="Коронавирус в Украине"
      confirmed={totalConfirmedUa}
      deaths={totalDeathsUa}
      recovered={totalRecoveredUa}
      deathsRate={$CovidStore.deathsRateUa}
    />
    <svg width="100%" height="100%">
      {#each [...Array(timestamps.length).keys()] as index}
        <rect {...confirmedRects[index]} />
        <rect {...recoveredRects[index]} />
        <rect {...deathsRects[index]} />
      {/each}
    </svg>
  </div>
</div>

<!-- <div class="w-1/2"> -->
<!-- <div class="GraphWrap GraphWrapHeight">
    <CovidGraphSummary
      title="Весь мир — сегодня"
      predictValues={true}
      confirmed={diffConfirmedAll}
      deaths={diffDeathsAll}
      recovered={diffRecoveredAll} />
    <svg width="100%" height="100%">
      {#each [...Array(timestamps.length).keys()] as index}
        <rect {...confirmedAllDiffRects[index]} />
        <rect {...recoveredAllDiffRects[index]} />
        <rect {...deathsAllDiffRects[index]} />
      {/each}
    </svg>
  </div> -->

<!-- <div class="GraphWrap GraphWrapHeight">
    <CovidGraphSummary
      title="Украина — сегодня"
      predictValues={true}
      confirmed={diffConfirmedUa}
      deaths={diffDeathsUa}
      recovered={diffRecoveredUa}
    />
    <svg width="100%" height="100%">
      {#each [...Array(timestamps.length).keys()] as index}
        <rect {...confirmedDiffRects[index]} />
        <rect {...recoveredDiffRects[index]} />
        <rect {...deathsDiffRects[index]} />
      {/each}
    </svg>
  </div>
</div> -->
<style type="text/postcss">
  .GraphWrap {
    @apply relative overflow-hidden w-full;
  }

  .GraphWrapHeight {
    height: 140px;
  }

  @screen md {
    .GraphWrapHeight {
      height: 100px;
    }
  }
</style>
