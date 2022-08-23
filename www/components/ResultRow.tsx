import React from 'react';
import { AnalyzerResult } from '../analyzer';
import { Filter, find_best_definitions } from '../filter';

export function ResultHeader(): JSX.Element {
    return <tr>
        <td>Word</td>
        <td>Count</td>
        <td>Reading</td>
        <td>Part of Speech</td>
        <td>Definition</td>
        <td>Examples</td>
        <td>Audio URL</td>
    </tr>
}

export function ResultRow({ result, filter }: {
    result: AnalyzerResult,
    filter: Filter
}): JSX.Element {
    let info = find_best_definitions(result, filter);
    let definitions = info.definitions
        .map(def => def.text)
        .join('<br/>');
    let examples = info.examples
        .map(ex => ex.ja + '<br/>' + ex.en)
        .join('<br/><br/>');
    return <tr key={result.word}>
        <td>{result.word}</td>
        <td>{result.count}</td>
        <td>{[...info.readings].join(', ')}</td>
        <td>{result.pos}</td>
        <td dangerouslySetInnerHTML={{__html: definitions}}></td>
        <td dangerouslySetInnerHTML={{__html: examples}}></td>
        <td>{info.audio == null ? "null": info.audio}</td>
    </tr>;
}
