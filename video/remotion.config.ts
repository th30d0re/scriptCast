import {Config} from '@remotion/cli/config';
import path from 'node:path';
Config.setVideoImageFormat('png');
Config.overrideWebpackConfig(config => {
  // __dirname is @remotion/cli/dist once this config is bundled; Remotion runs from video/.
  const video = path.resolve(process.env.STAGE_VIDEO_DIR ?? process.cwd());
  return {...config, resolve: {...config.resolve,
    alias: {...config.resolve?.alias, '@episode': process.env.STAGE_EPISODE_DIR ?? path.join(video,'episodes/_demo'), '@stage': path.join(video,'src/stage/api.ts'), react: path.join(video,'node_modules/react'), 'react-dom': path.join(video,'node_modules/react-dom'), remotion: path.join(video,'node_modules/remotion'), 'd3-geo': path.join(video,'node_modules/d3-geo')},
    modules: [path.join(video,'node_modules'), ...(config.resolve?.modules ?? ['node_modules'])]}};
});
