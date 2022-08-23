import React from 'react';
import { useTextAnalyzer } from '../analyzer';
import { Filter } from '../filter';
import ResultsView from './ResultsView';

// Returns true if all of the characters in the string are in either the Hiragana or Katakana Unicode blocks
function is_kana(s: string): boolean {
    for (const char of s) {
        const codepoint = char.codePointAt(0);
        // Hiragana: 0x304x-0x309x
        // Katakana: 0x30Ax-0x30Fx
        if (!(codepoint >= 0x3040 && codepoint < 0x3100)) {
            return false;
        }
    }
    return true;
}

const text = "そして我々を選んだのかもしれない。";
const filter: Filter = [
    [
        { tag: (def, _, ctx) => def.pos.includes(ctx.pos), weight: 1 }
    ],
    [
        { tag: "&arch;", weight: -1},
        { tag: "&rare;", weight: -1}
    ],
    [
        { tag: (def, _, ctx) => def.flags.includes("&uk;") == is_kana(ctx.word), weight: 1 }
    ]
];

export default function App(): JSX.Element {
    let result = useTextAnalyzer(text);

    let content;
    if (result == null) {
        content = <section className='section'>Loading...</section>
    } else {
        content = <section className="section">
            <ResultsView results={result} filter={filter}/>
        </section>
    }

    return <div id="app" className="columns">
        <div className="column is-one-quarter">
            <section className="section">
                Hello, world!
            </section>
        </div>
        <div className="column scroll-y">
            {content}
        </div>
    </div>
}