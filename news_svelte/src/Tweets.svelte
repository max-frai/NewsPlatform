<script>
    import { WsMainStore } from "./ws_store.js";
    import { onMount } from "svelte";
    import { tick } from "svelte";
    import { fetchTweets } from "./api.js";
    import Masonry from "masonry-layout";
    import Heart from "./icons/Heart.svelte";
    import Clock from "./icons/Clock.svelte";
    import Retweet from "./icons/Retweet.svelte";
    import Play from "./icons/Play.svelte";
    import Link from "./icons/Link.svelte";
    import GroupButtons from "./components/GroupButtons.svelte";
    import OverlayPopup from "./components/OverlayPopup.svelte";
    import DateFormat from "dateformat";
    let tweets = [];
    let tweetsCache = {};
    let active = "12 часов";
    let masonryHandle = null;
    let popupTweet = null;
    let title2val = {
        час: "1hr",
        "12 часов": "12hr",
        день: "24hr",
        неделя: "week",
        месяц: "month",
    };
    let val2title = {
        "1hour": "час",
        "12hr": "12 часов",
        "24hr": "день",
        week: "неделя",
        month: "месяц",
    };
    $: updateTweets($WsMainStore);
    function createMasonry() {
        var grid = document.querySelector(".TweetsGrid");
        if (masonryHandle) {
            masonryHandle.destroy();
        }
        masonryHandle = new Masonry(grid, {
            itemSelector: ".MasonryItem",
        });
    }
    async function updateTweets(jsonData) {
        if (!jsonData) return;
        if (jsonData.kind != "TweetsMessage") return;
        jsonData = JSON.parse(jsonData.data);

        for (let i = 0; i < jsonData.tweets.length; i++) {
            let data = jsonData.tweets[i];
            tweetsCache[data.kind] = data.tweets;

            if (val2title[data.kind] == active) {
                tweets = data.tweets;
            }
        }

        await tick();
        createMasonry();
    }
    function tweetCover(tweet) {
        if (!tweet.entities.length) return null;
        for (var i = 0; i < tweet.entities.length; i++) {
            let entity = tweet.entities[i];
            if (entity.kind == "Photo") {
                return entity.media_url;
            }
        }
        return null;
    }
    function tweetVideoThumb(tweet) {
        if (!tweet.entities.length) return null;
        for (var i = 0; i < tweet.entities.length; i++) {
            let entity = tweet.entities[i];
            if (entity.kind == "Video") {
                return entity.url;
            }
        }
        return null;
    }
    function tweetVideoFile(tweet) {
        for (var i = 0; i < tweet.entities.length; i++) {
            let entity = tweet.entities[i];
            if (entity.kind == "Video") {
                return entity.media_url;
            }
        }
        return null;
    }
    function tweetUrls(tweet) {
        if (!tweet.entities.length) return [];
        let urls = [];
        for (var i = 0; i < tweet.entities.length; i++) {
            let entity = tweet.entities[i];
            if (entity.kind == "Url") {
                urls.push(entity.url);
            }
        }
        return urls;
    }
    async function activeChanged(button) {
        active = button.detail.active;
        await loadTweets(active);
    }
    async function loadTweets(active) {
        tweets = tweetsCache[title2val[active]];
        await tick();
        createMasonry();
    }
    // onMount(async () => {
    //     if (!tweets.length) {
    //         await loadTweets(active);
    //     }
    // });
</script>

<div class="container mx-auto">
    <div class="px-3">
        <h2 class="BlockTitle">Популярные твиты за</h2>
        <GroupButtons
            on:activeChanged={activeChanged}
            buttons={Object.keys(title2val)}
            {active}
        />
    </div>
</div>

<hr class="mb-1" />

<OverlayPopup is_visible={popupTweet}>
    <div class="w-full p-5">
        {#if tweetVideoThumb(popupTweet)}
            <video
                autoplay
                controls
                class="w-full"
                src={tweetVideoFile(popupTweet)}
            />
        {:else}
            <img src={tweetCover(popupTweet)} />
        {/if}
        <div class="leading-tight">
            {@html popupTweet.text.replace(/\n/g, "<br>")}
        </div>
    </div>
</OverlayPopup>

<div class="py-1">
    <div class="TweetsGrid container mx-auto relative" style="max-width: 880px">
        {#if Object.keys(tweets).length}
            {#each tweets as tweet}
                <div class="MasonryItem w-full px-5 md:w-1/2 md:px-4 mb-2">
                    <div class="Tweet GrayBorder" data-id={tweet.id}>
                        {#if tweetCover(tweet)}
                            <div
                                on:click={() => (popupTweet = tweet)}
                                class="w-full h-32 bg-cover"
                                style="cursor: zoom-in; background-image: url({tweetCover(
                                    tweet
                                )})"
                            />
                        {/if}
                        {#if tweetVideoThumb(tweet)}
                            <div
                                on:mouseenter={() => (tweet.hover = true)}
                                on:mouseleave={() => (tweet.hover = false)}
                                on:click={() => (popupTweet = tweet)}
                            >
                                {#if tweet.hover}
                                    <div
                                        class="w-full h-32 object-cover relative"
                                        style="cursor: zoom-in"
                                    >
                                        <video
                                            autoplay
                                            class="object-cover h-32 w-full"
                                            src={tweetVideoFile(tweet)}
                                        />
                                    </div>
                                {:else}
                                    <div
                                        class="w-full h-32 bg-cover relative"
                                        style="cursor: zoom-in; background-image: url({tweetVideoThumb(
                                            tweet
                                        )})"
                                    >
                                        <div
                                            style="background: rgba(32,35,51,.7);"
                                            class="rounded-full inline-block p-2 color-white absolute
                        mx-auto my-auto left-0 right-0 top-0 bottom-0 w-12 h-12"
                                        >
                                            <Play />
                                        </div>
                                    </div>
                                {/if}
                            </div>
                        {/if}
                        <div class="p-3 px-4 break-word">
                            <h3 class="mb-1">{tweet.user_name}</h3>
                            {@html tweet.text.replace(/\n/g, "<br>")}
                            {#each tweetUrls(tweet) as url}
                                <a class="TweetLink" href={url} target="_blank"
                                    >{url}</a
                                >
                            {/each}

                            <hr class="my-2" />
                            <div
                                class="FlexAllCenter justify-between opacity-50"
                            >
                                <div class="FlexAllCenter">
                                    <Retweet />
                                    &nbsp; {tweet.retweets}
                                </div>
                                <div class="FlexAllCenter">
                                    <Heart />
                                    &nbsp; {tweet.favorites}
                                </div>
                                <div class="FlexAllCenter">
                                    <Clock />
                                    &nbsp; {DateFormat(
                                        Date.parse(tweet.when.$date),
                                        "HH:MM • dd.mm"
                                    )}
                                </div>
                                <div class="FlexAllCenter">
                                    <a
                                        target="_blank"
                                        href="https://twitter.com/{tweet.user_screenname}/status/{tweet.id}"
                                    >
                                        <Link />
                                    </a>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            {/each}
        {/if}
    </div>
</div>

<style type="text/postcss">
    .Tweet {
        @apply bg-white w-full rounded rounded-lg text-sm leading-tight overflow-hidden;
        /* width: 280px; */
    }
    .TweetsGrid {
        min-height: 400px;
    }
    .TweetLink {
        @apply block text-xs truncate mt-1;
        color: #718096 !important;
    }
</style>
