import { transpileModule } from "typescript";
import { AnalyzerResult, Definition, DictionaryEntry, Example } from "./analyzer";

export interface FilterItem {
    tag: string | ((def: Definition, entry: DictionaryEntry, ctx: AnalyzerResult) => boolean),
    weight: number
}

export type Filter = FilterItem[][];

export interface FilterResult {
    definitions: Definition[],
    audio: string[],
    examples: Example[],
    readings: Set<string>
};

export function weight(entry: DictionaryEntry, filter: Filter, ctx: AnalyzerResult): number[] {
    return filter.map(items => {
        let weight = 0;
        for (let item of items) {
            for (let definition of entry.definitions) {
                if (typeof item.tag == "string" ? definition.flags.includes(item.tag) : item.tag(definition, entry, ctx)) {
                    weight += item.weight;
                    break;
                }
            }
        }
        return weight;
    });
}

export function lex_cmp(a: number[], b: number[]): number {
    if (a.length != b.length) {
        // if length is not equal, sort shorter list first
        return a.length - b.length
    } else {
        // compare corresponding values of a and b
        for (let i = 0; i < a.length; i++) {
            if (a[i] != b[i]) {
                return a[i] - b[i]
            }
        }
        // all values are equal, so a and b are equal
        return 0;
    }
}

export function find_best_definitions(data: AnalyzerResult, filter: Filter): FilterResult {
    let sorted_entries = data.dict_info
        .map(entry => ({ entry: entry, weight: weight(entry, filter, data) }))
        .sort((a, b) => -lex_cmp(a.weight, b.weight));

    let result: FilterResult = {
        definitions: [],
        audio: [],
        examples: [],
        readings: new Set()
    }

    // edge-case when the entry list is empty
    if (sorted_entries.length == 0) return result;

    let current_weight = sorted_entries[0].weight;
    let need_definitions = true;
    let need_audio = true;
    let need_readings = true;
    for (let { entry, weight } of sorted_entries) {
        if (lex_cmp(current_weight, weight) <= 0) {
            if (need_definitions) {
                result.definitions = result.definitions.concat(entry.definitions);
                result.examples = result.examples.concat(entry.examples);
            }
            if (need_audio && entry.audio != null) {
                result.audio = result.audio.concat(entry.audio);
            }
            if (need_readings) {
                for (const reading of entry.readings) {
                    result.readings.add(reading);
                }
            }
        } else {
            current_weight = weight;
            if (need_definitions && result.definitions.length > 0) {
                need_definitions = false;
            }
            if (need_audio && result.audio.length > 0) {
                need_audio = false;
            }
            if (need_readings && result.readings.size > 0) {
                need_readings = false;
            }
        }
    }

    // Use the Lindera reading if there are no JMDict/WaniKani readings
    if (result.readings.size == 0) result.readings.add(data.reading);
    
    // If one of the readings is identical to the word itself, remove all other readings
    // Example: The dictionary entry for "そした" includes the readings "そした" and "しかした".
    //   This removes the second irrelevant reading.
    for (const reading of result.readings) {
        if (reading == data.word) {
            result.readings.clear();
            result.readings.add(reading);
            break;
        }
    }

    return result;
}
