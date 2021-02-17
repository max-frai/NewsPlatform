import {
    writable
} from 'svelte/store';

function createStore() {
    const {
        subscribe,
        set,
        update
    } = writable({
        confirmedInterval: 1000,
        deathsInterval: 1000,
        recoveredInterval: 1000,
        deathsRateAll: 0.0,
        deathsRateUa: 0.0,
    });

    return {
        subscribe,
        confirmedDiff: (diffLast, diffPreLast) => update(data => {
            let koef = diffLast / diffPreLast;
            data.confirmedInterval = (60 / ((diffLast * koef) / 24 / 60)) * 1000;
            return data;
        }),
        deathsDiff: (diffLast, diffPreLast) => update(data => {
            let koef = diffLast / diffPreLast;
            data.deathsInterval = (60 / ((diffLast * koef) / 24 / 60)) * 1000;
            return data;
        }),
        recoveredDiff: (diffLast, diffPreLast) => update(data => {
            if (diffLast == 0) diffLast = diffPreLast;
            let koef = diffLast / diffPreLast;
            data.recoveredInterval = (60 / ((diffLast * koef) / 24 / 60)) * 1000;
            return data;
        }),

        deathRateAll: (confirmedAll, deathsAll) => update(data => {
            data.deathsRateAll = deathsAll / confirmedAll * 100;
            return data;
        }),
        deathRateUa: (confirmedUa, deathsUa) => update(data => {
            data.deathsRateUa = deathsUa / confirmedUa * 100;
            return data;
        }),

        reset: () => set({
            confirmedInterval: 1000,
            deathsInterval: 1000,
            recoveredInterval: 1000,
            deathsRateAll: 0.0,
            deathsRateua: 0.0
        })
    };
}

export const CovidStore = createStore();