import spacy, json, sys
nlp = spacy.load("ru_core_news_lg")

text = ''.join(sys.stdin.readlines())
doc = nlp(text)

results = []
for ent in doc.ents:
    results.append([ent.text, ent.label_])

print(json.dumps(results))