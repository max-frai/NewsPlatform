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

new Charts({
	target: document.querySelector('#SvelteCharts')
});
new RunningLine({
	target: document.querySelector('#SvelteRunningLine')
});

// export default app;