import {useState} from 'react';
import {KitSheetIndex} from '../../../video/src/stage/kit/KitSheet';
import {ItemPlayer} from './ItemPlayer';
import {ItemRail} from './ItemRail';
import {ReviewPanel} from './ReviewPanel';
import {emptyItem, useKitReview} from './useKitReview';

export default function KitReviewPage() {
  const [selected, setSelected] = useState(KitSheetIndex[0].id); const {review, updateItem, state} = useKitReview();
  const item = KitSheetIndex.find((candidate) => candidate.id === selected) ?? KitSheetIndex[0];
  return <main className="kit-review"><ItemRail items={KitSheetIndex} review={review} selected={item.id} onSelect={setSelected}/><div className="kit-stage"><ItemPlayer item={item.id} motion={item.motion}/></div><ReviewPanel source={item.source} item={review.items[item.id] ?? emptyItem()} onChange={(patch) => updateItem(item.id, patch)} saveState={state}/></main>;
}
