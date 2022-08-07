const path = require('path')
const HtmlWebpackPlugin = require('html-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin')

module.exports = {
    mode: 'development',
    entry: './www/bootstrap.js',
    devtool: 'source-map',
    output: {
        filename: 'bundle.js',
        path: path.resolve(__dirname, 'dist'),
    },
    module: {
        rules: [
            {
                test: /\.tsx?$/i,
                use: 'ts-loader',
                exclude: /node_modules/,
            },
            {
                test: /\.scss$/i,
                use: [
                    MiniCssExtractPlugin.loader,
                    'css-loader',
                    'sass-loader'
                ]
            }
        ],
    },
    resolve: {
        extensions: ['.tsx', '.ts', '.js'],
    },
    plugins: [
        new HtmlWebpackPlugin({ title: "JP2Anki", template: "template.html" }),
        new WasmPackPlugin({ crateDirectory: path.resolve(__dirname, '.') }),
        new MiniCssExtractPlugin()
    ],
    devServer: {
        static: {
            directory: path.join(__dirname, 'dist')
        },
        port: 8080
    },
    experiments: {
        syncWebAssembly: true
    },
    performance: {
        hints: false
    }
};