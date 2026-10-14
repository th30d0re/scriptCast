import {Frame} from './Frame';
import {InlineMath} from './Math';
import {palette as p} from '../theme';
export type TableProps = {headline: string; columns: string[]; rows: string[][]; latexCells?: boolean; sources: string};
export function TableCard(props: TableProps) {
  if (!props.columns.length || props.rows.some(row => row.length !== props.columns.length)) throw new Error('Table rows must match the columns');
  const cell = (text: string) => props.latexCells ? <InlineMath text={text} /> : text;
  return <Frame {...props} qr={false}>{scale => <table style={{width: '100%', borderCollapse: 'collapse', tableLayout: 'fixed', fontSize: (props.columns.length > 4 ? 28 : 34) * scale, lineHeight: 1.25, overflowWrap: 'anywhere'}}>
    <thead><tr>{props.columns.map((column, i) => <th key={i} style={{padding: 16 * scale, borderBottom: `3px solid ${p.gold}`, textAlign: 'left', color: p.gold}}>{cell(column)}</th>)}</tr></thead>
    <tbody>{props.rows.map((row, i) => <tr key={i} style={{background: i % 2 ? p.navy2 : 'transparent'}}>{row.map((value, j) => <td key={j} style={{padding: 16 * scale, borderBottom: `1px solid ${p.mute}`, verticalAlign: 'top'}}>{cell(value)}</td>)}</tr>)}</tbody>
  </table>}</Frame>;
}
