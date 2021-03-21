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
var charts_handle = document.querySelector('#SvelteCharts');
if (charts_handle) {
    new Charts({
        target: charts_handle
    });
}

import RunningLine from './RunningLine.svelte';
var running_handle = document.querySelector('#SvelteRunningLine');
if (running_handle) {
    new RunningLine({
        target: running_handle
    });
}

import TopNews from './TopNews.svelte';
var top_news_handle = document.querySelector('#SvelteTopNews');
if (top_news_handle) {
    new TopNews({
        target: top_news_handle
    });
}

import Trends from './Trends.svelte';
var trends_handle = document.querySelector('#SvelteTrends');
if (trends_handle) {
    new Trends({
        target: trends_handle
    });
}

import Covid from './Covid.svelte';
var covid_handle = document.querySelector('#SvelteCovid')
if (covid_handle) {
    new Covid({
        target: covid_handle
    });
}

import Tweets from './Tweets.svelte';
var tweets_handle = document.querySelector('#SvelteTweets')
if (tweets_handle) {
    new Tweets({
        target: tweets_handle
    });
}