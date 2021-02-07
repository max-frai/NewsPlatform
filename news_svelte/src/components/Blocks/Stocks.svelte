<script>
    import GraphLinearGradient from "../GraphLinearGradient.svelte";
    import { WsMainStore, WsInvestingStore } from "../../ws_store.js";
    import { createEventDispatcher } from "svelte";

    export let id = -1;
    export const type = "";

    const dispatch = createEventDispatcher();

    let graphHeight = 28;
    let graphWidth = 100;

    let dataPath = "";
    let dataFill = "";
    let currentValue = "";
    let currentValueNumeric = -1;
    let fillColor = "green";
    let valueLastDiff = 0;
    let valueLastDiffPercentage = 0;
    let graphStep = 0;

    $: graphExtendedHeight = graphHeight + 10;
    $: updateGraphs($WsMainStore);
    $: updateInfo($WsInvestingStore);

    function updateInfo(jsonData) {
        if (!jsonData) return;
        // console.log(jsonData);
        if (jsonData.pid != id.toString()) return;

        let prevCurrentValueNumeric = currentValueNumeric;
        currentValue = makePrettyNumber(jsonData.last_numeric);
        valueLastDiffPercentage = makePrettyPercentage(
            makePrettyNumber(parseFloat(jsonData.pcp.replace("%", "")))
        );
        currentValueNumeric = jsonData.last_numeric;

        if (jsonData.pcp.startsWith("-")) {
            fillColor = "red";
        } else {
            fillColor = "green";
        }

        if (prevCurrentValueNumeric != jsonData.last_numeric) {
            let diffBgClass =
                jsonData.last_numeric > prevCurrentValueNumeric
                    ? "bg-green-100"
                    : "bg-red-100";

            let borderClass =
                jsonData.last_numeric > prevCurrentValueNumeric
                    ? "green"
                    : "red";

            dispatch("newbg", { class: diffBgClass, borderClass });
            window.setTimeout(() => {
                dispatch("newbg", {
                    class: "bg-white",
                    borderClass: "border-gray-300",
                });
            }, 1000);
        }
    }

    function makePrettyNumber(number) {
        let data = new Intl.NumberFormat("ru-RU", {
            maximumFractionDigits: 2,
        }).format(number);
        data = data.replace(/\.(\d+)/, '<span class="opacity-50">.$1</span>');
        return data;
    }

    function makePrettyPercentage(stringNumber) {
        let value = stringNumber + '<span class="opacity-50">%</span>';
        let isNegative = value.startsWith("-");
        let color = isNegative ? "text-red-600" : "text-green-600";

        if (!isNegative && !value.startsWith("+")) {
            value = "+" + value;
        }

        return `<span class="${color}">${value}</span>`;
    }

    function updateGraphs(jsonData) {
        if (!jsonData) return;
        if (jsonData.kind != "ChartsMessage") return;

        jsonData = JSON.parse(jsonData.data);
        jsonData = jsonData.stocks[parseInt(id)];
        if (!jsonData) return;

        currentValue = makePrettyNumber(jsonData[jsonData.length - 1]);

        let last = jsonData[jsonData.length - 1];
        let preLast = jsonData[jsonData.length - 2];

        valueLastDiff = last - preLast;
        valueLastDiffPercentage = makePrettyPercentage(
            `${makePrettyNumber((valueLastDiff * 100) / last)}`
        );

        if (last < preLast) fillColor = "red";

        let min = Math.min(...jsonData);
        jsonData = jsonData.map((i) => i - min);

        let max = Math.max(...jsonData);
        jsonData = jsonData.map((i) => 1 - i / max);

        dataPath = `M 0 ${jsonData[0] * graphHeight + 1}`;
        let x = 0;
        graphStep = graphWidth / jsonData.length;
        for (var i = 1; i < jsonData.length; i++) {
            x += graphStep;
            dataPath += ` L ${x} ${jsonData[i] * graphHeight + 1}`;
        }

        dataFill = `M 0 ${graphExtendedHeight} ${dataPath.replace(
            "M",
            "L"
        )} L ${x} ${graphExtendedHeight}`;
    }
</script>

<div>
    <div class="h-10">
        <svg width={graphWidth - graphStep} height={graphExtendedHeight}>
            <GraphLinearGradient />
            <g>
                <path
                    d={dataPath}
                    stroke="rgba(240, 185, 10, 1)"
                    fill="none"
                    stroke-width="1"
                />
                <path d={dataFill} fill="url(#{fillColor})" />
            </g>
        </svg>
    </div>
    <hr />
    <div class="flex pt-1 text-xs flex-no-wrap justify-between">
        <span class="flex font-medium opacity-75 ">
            {@html currentValue}
        </span>
        <span class="flex">
            {@html valueLastDiffPercentage}
        </span>
    </div>
</div>

<style type="text/postcss">
</style>
