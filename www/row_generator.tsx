import { AnalyzerResult, Definition, DictionaryEntry } from './analyzer';

export interface DefinitionFilter {
    prioritized_tags: string[],
    deprioritized_tags: string[]
}

function priority(def: Definition, filter_tags: string[]): number {
    for (let idx = 0; idx < filter_tags.length; idx++) {
        for (let tag in def.flags) {
            if (tag == filter_tags[idx]) {
                return 1 - idx / filter_tags.length;
            }
        }
    }
    return 0;
}

export function generate_row(result: AnalyzerResult, filter: DefinitionFilter) {
    let all_definitions: [DictionaryEntry, Definition][] = [];
    for (let entry of result.dict_info) {
        for (let def of entry.definitions) {
            all_definitions.push([entry, def])
        }
    }

    all_definitions.sort(([_e0, a], [_e1, b]) => {
        return priority(b, filter.prioritized_tags)
    });
}