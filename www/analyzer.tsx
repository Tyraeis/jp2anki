import { useState, useEffect } from 'react';
import { init, TextAnalyzer } from '../pkg';

init();

export type Source = { "WaniKani": number } | { "JMDict": number };

export type PartOfSpeech = "Noun" | "Prefix" | "Verb" | "Adjective" | "Adverb"
    | "Adnominal" | "Conjuction" | "Particle" | "AuxiliaryVerb"
    | "Exclamation" | "Symbol" | "Filler" | "Other";

export interface AnalyzerResult {
    word: string,
    pos: PartOfSpeech,
    reading: string,
    count: number,
    dict_info: DictionaryEntry[]
}

export interface DictionaryEntry {
    forms: string[],
    source: Source,
    definitions: Definition[],
    audio: string[],
    readings: string[],
    examples: Example[]
}

export interface Definition {
    text: string,
    pos: PartOfSpeech[],
    flags: string[]
}

export interface Example {
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

export function useTextAnalyzer(text: string): AnalyzerResult[] | null {
    const [analyzer, set_analyzer] = useState<TextAnalyzer | null>(null);
    const [result, set_result] = useState<AnalyzerResult[] | null>(null);
    useEffect(() => {
        loadTextAnalyzer().then(set_analyzer)
    }, []);
    useEffect(() => {
        if (analyzer != null) {
            const res = analyzer.analyze(text);
            console.log(res);
            set_result(res);
        }
    }, [analyzer, text])
    return result;
}