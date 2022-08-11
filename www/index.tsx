import React from 'react';
import { useState, useEffect } from 'react';
import * as ReactDOM from 'react-dom/client';
import { init, TextAnalyzer } from '../pkg';

init();

type Source = { "WaniKani": number } | { "JMDict": number };

type PartOfSpeech = "Noun" | "Prefix" | "Verb" | "Adjective" | "Adverb"
    | "Adnominal" | "Conjuction" | "Particle" | "AuxiliaryVerb"
    | "Exclamation" | "Symbol" | "Filler" | "Other";

type AnalyzerResult = Record<string, DictionaryEntry>;

interface DictionaryEntry {
    forms: string[],
    source: Source,
    definitions: Definition[],
    audio: string | null,
    readings: string[],
    examples: Example[]
}

interface Definition {
    text: string,
    pos: PartOfSpeech[],
    flags: string[]
}

interface Example {
    for_definition: number | null,
    en: string,
    ja: string
}

async function loadTextAnalyzer(): Promise<TextAnalyzer> {
    const dict_idx_p = fetch("dictionary.idx")
        .then(resp => resp.arrayBuffer());
    const dict_dat_p = fetch("dictionary.dat")
        .then(resp => resp.arrayBuffer());

    const dict_idx = await dict_idx_p;
    const dict_dat = await dict_dat_p;

    return TextAnalyzer.new(
        new Uint8Array(dict_idx),
        new Uint8Array(dict_dat)
    );
}


function useTextAnalyzer(text: string): AnalyzerResult | null {
    const [analyzer, set_analyzer] = useState<TextAnalyzer | null>(null);
    const [result, set_result] = useState<AnalyzerResult | null>(null);
    useEffect(() => {
        loadTextAnalyzer().then(set_analyzer)
    }, []);
    useEffect(() => {
        if (analyzer != null) {
            set_result(analyzer.analyze(text))
        }
    }, [analyzer, text])
    return result;
}

const text = "そして我々を選んだのかもしれない。";

function App(): JSX.Element {
    let result = useTextAnalyzer(text);

    if (result == null) {
        return <div>Loading...</div>
    } else {
        return <div><pre>{JSON.stringify(result, undefined, 4)}</pre></div>
    }
}

const root = ReactDOM.createRoot(document.getElementById("main"));
root.render(<App/>);
