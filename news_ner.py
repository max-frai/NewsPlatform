import spacy, json, sys
nlp = spacy.load("ru_core_news_lg")

from aiohttp import web

async def handle(request):
    results = []
    try:
        print("Ner request")
        nlp = request.app['nlp']

        text = await request.text()
        doc = nlp(text)

        results = []
        for ent in doc.ents:
            results.append([ent.text, ent.label_])
    except Exception as e:
        print(e)

    return web.json_response(results)

app = web.Application()
app['nlp'] = nlp

app.add_routes([web.post('/ner', handle)])

if __name__ == '__main__':
    web.run_app(app, port=2088)