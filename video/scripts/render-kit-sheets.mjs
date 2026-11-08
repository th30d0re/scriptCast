#!/usr/bin/env node
/**
 * Renders every KitSheet specimen as a 2x PNG and as a print-quality PDF.
 * The PDF path deliberately prints the Remotion composition DOM in Chrome,
 * rather than wrapping the rendered PNG, so SVG paths and text stay vector.
 */
import {mkdir, access} from 'node:fs/promises';
import {spawn} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import path from 'node:path';

const here = path.dirname(fileURLToPath(import.meta.url));
const video = path.resolve(here, '..');
const output = process.argv[2];
const chrome = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
if (!output) throw new Error('Usage: node scripts/render-kit-sheets.mjs <output-dir>');

const run = (command, args, options = {}) => new Promise((resolve, reject) => {
  const child = spawn(command, args, {cwd: video, stdio: 'inherit', ...options});
  child.on('error', reject);
  child.on('exit', (code) => code === 0 ? resolve() : reject(new Error(`${command} exited ${code}`)));
});

// Kept in sync with KitSheetIndex. Importing a TSX module from Node would add a
// build-time dependency, so the renderer reads the exported IDs from source.
const items = [
  'CitationCard', 'DefinitionCard', 'EvidenceToken', 'GroupDirectionField', 'NeuralField', 'PrecisionGauge', 'PredictiveUpdatePlot', 'QuestionHeader', 'SignalLane', 'SourceLine', 'StageSceneFrame', 'TermBlock', 'TermTriad', 'TrainingTextStrip', 'UsThemSortingField',
  'CardBorder', 'CompactBorder', 'SceneBorder', 'PersonMark', 'NumberBadge', 'PersonIcon', 'ActionIcon', 'SystemIcon', 'WordIcon', 'HeartIcon', 'CausalArrow', 'ScanLines',
  'BootIn', 'DrawLine', 'DrawRect', 'FitBox', 'Flicker', 'StickFigure', 'Trace',
];

const pngDir = path.join(output, 'png');
const pdfDir = path.join(output, 'pdf');
await Promise.all([mkdir(pngDir, {recursive: true}), mkdir(pdfDir, {recursive: true}), access(chrome)]);

// Remotion renders a still straight to PDF (image format 'pdf' prints the page), so text and SVG stay vector.
const browser = `--browser-executable=${chrome}`;
for (const item of items) {
  const props = JSON.stringify({item});
  await run('npx', ['remotion', 'still', 'src/index.ts', 'KitSheet', path.join(pngDir, `${item}.png`), '--scale=2', `--props=${props}`, browser]);
  await run('npx', ['remotion', 'still', 'src/index.ts', 'KitSheet', path.join(pdfDir, `${item}.pdf`), '--image-format=pdf', `--props=${props}`, browser]);
}
