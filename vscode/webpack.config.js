const path = require('path');
const webpack = require('webpack');
const { execSync } = require('child_process');

function getGitCommitHash() {
  try {
    return execSync('git rev-parse --short HEAD', { encoding: 'utf-8' }).trim();
  } catch {
    return 'unknown';
  }
}

/**@type {import('webpack').Configuration}*/
const config = {
  target: 'node', // VSCode 扩展运行在 Node.js 环境
  mode: 'none', // 不进行优化，保留可读性

  entry: './src/extension.ts', // 扩展入口
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'extension.js',
    libraryTarget: 'commonjs2',
    devtoolModuleFilenameTemplate: '../[resource-path]'
  },
  externals: {
    vscode: 'commonjs vscode' // vscode 模块由 VSCode 提供，不打包
  },
  resolve: {
    extensions: ['.ts', '.js'],
    mainFields: ['module', 'main']
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        exclude: /node_modules/,
        use: [
          {
            loader: 'ts-loader',
            options: {
              compilerOptions: {
                module: 'esnext'
              }
            }
          }
        ]
      }
    ]
  },
  devtool: 'nosources-source-map',
  infrastructureLogging: {
    level: "log", // 启用日志以便调试
  },
  // 确保 node_modules 中的依赖被正确打包
  plugins: [
    new webpack.DefinePlugin({
      __GIT_COMMIT_HASH__: JSON.stringify(getGitCommitHash()),
    }),
  ],
  optimization: {
    minimize: false
  }
};

module.exports = config;
