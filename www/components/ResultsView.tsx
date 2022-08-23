import React from 'react';
import { AnalyzerResult } from '../analyzer';
import { Filter } from '../filter';
import { ResultHeader, ResultRow } from './ResultRow';

export default function ResultsView({results, filter}: {
    results: AnalyzerResult[],
    filter: Filter
}): JSX.Element {
    return <div className="table-container">
        <table className='table is-hoverable is-fullwidth is-bordered'>
            <thead>
                <ResultHeader/>
            </thead>
            <tbody>
                {results.map(row => <ResultRow result={row} filter={filter}/>)}
            </tbody>
        </table>
    </div>
}