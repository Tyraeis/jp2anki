import React from 'react';
import { AnalyzerResult } from '../analyzer';

export default function ResultsView(props: { results: AnalyzerResult[] }): JSX.Element {
    return <div className="table-container">
        <table className='table is-hoverable is-fullwidth is-bordered'>
            <thead>
                <tr>
                    <td>Word</td>
                    <td>Count</td>
                    <td>Reading</td>
                    <td>Part of Speech</td>
                </tr>
            </thead>
            <tbody>
                {props.results.map(row =>
                    <tr>
                        <td>{row.word}</td>
                        <td>{row.count}</td>
                        <td>{row.reading}</td>
                        <td>{row.pos}</td>
                    </tr>
                )}
            </tbody>
        </table>
    </div>
}