import {Config} from '@remotion/cli/config';
import fs from 'node:fs';
import path from 'node:path';
Config.setVideoImageFormat('png');
Config.overrideWebpackConfig(config => {
  // __dirname is @remotion/cli/dist once this config is bundled; Remotion runs from video/.
  const video = path.resolve(process.env.STAGE_VIDEO_DIR ?? process.cwd());
  // npm workspaces hoist dependencies to the repo root; a standalone checkout keeps them in video/.
  const roots = [path.join(video,'node_modules'), path.join(video,'..','node_modules')];
  const dep = (name: string) => fs.realpathSync(path.join(roots.find(root => fs.existsSync(path.join(root,name))) ?? roots[0], name));
  return {...config, resolve: {...config.resolve,
    symlinks: true,
    alias: {...config.resolve?.alias, '@episode': process.env.STAGE_EPISODE_DIR ?? path.join(video,'episodes/_demo'), '@stage': path.join(video,'src/stage/api.ts'), react: dep('react'), 'react-dom': dep('react-dom'), remotion: dep('remotion'), 'd3-geo': dep('d3-geo')},
    modules: [...roots, ...(config.resolve?.modules ?? ['node_modules'])]}};
});
