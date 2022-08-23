import React from 'react';
import { AnalyzerResult } from '../analyzer';
import { Filter, find_best_definitions } from '../filter';

export function ResultHeader(): JSX.Element {
    return <tr>
        <th>Word</th>
        <th>Count</th>
        <th>Reading</th>
        <th>Part of Speech</th>
        <th>Definition</th>
        <th>Examples</th>
        <th>Audio URL</th>
    </tr>
}

export function ResultRow({ result, filter }: {
    result: AnalyzerResult,
    filter: Filter
}): JSX.Element {
    let info = find_best_definitions(result, filter);
    return <tr key={result.word}>
        <td>{result.word}</td>
        <td>{result.count}</td>
        <td>{[...info.readings].join(', ')}</td>
        <td>{result.pos}</td>
        <td><div className="content">{info.definitions.map((def, i) => <p>{i+1}. {def.text}</p>)}</div></td>
        <td><div className="content">{info.examples.map(ex => <p>{ex.ja}<br/>{ex.en}</p>)}</div></td>
        <td>{info.audio.map(a => <p><a href={a} target="_blank">{a}</a></p>)}</td>
    </tr>;
}
