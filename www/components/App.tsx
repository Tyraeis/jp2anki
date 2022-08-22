import React from 'react';
import { useTextAnalyzer } from '../analyzer';
import ResultsView from './ResultsView';

const text = "そして我々を選んだのかもしれない。";

export default function App(): JSX.Element {
    let result = useTextAnalyzer(text);

    let content;
    if (result == null) {
        content = <section className='section'>Loading...</section>
    } else {
        content = <React.Fragment>
            <section className='section'>
                <ResultsView results={result}/>
            </section>
            <section className='section'>
                <pre>{JSON.stringify(result, undefined, 4)}</pre>
            </section>
        </React.Fragment>
    }

    return <div className="container">
        {content}
    </div>
}