import {
    register,
    init,
    getLocaleFromNavigator,
    addMessages,
    _
} from "svelte-i18n";

import ru from "./locales/ru.json";

addMessages('ru', ru);

init({
    initialLocale: 'ru'
});

import Charts from './Charts.svelte';
import RunningLine from './RunningLine.svelte';
import TopNews from './TopNews.svelte';
import Trends from './Trends.svelte';
import Covid from './Covid.svelte';

new Charts({
    target: document.querySelector('#SvelteCharts')
});
new RunningLine({
    target: document.querySelector('#SvelteRunningLine')
});
new TopNews({
    target: document.querySelector('#SvelteTopNews')
});
new Trends({
    target: document.querySelector('#SvelteTrends')
});
new Covid({
    target: document.querySelector('#SvelteCovid')
});

// export default app;