import {Frame, type BaseProps} from './Frame';
import {palette as p} from '../theme';
export type TitleProps = BaseProps & {items: {title: string; detail?: string}[]};
export function TitleCard(props: TitleProps) {
  return <Frame {...props}>{props.items.map((item, i) => <div key={i} style={{margin: '24px 0', textAlign: 'center'}}><div style={{fontSize: 40, color: p.gold}}>{item.title}</div><div style={{fontSize: 28}}>{item.detail}</div></div>)}</Frame>;
}
