const path = require('path');
const TerserPlugin = require('terser-webpack-plugin');

const PATHS = {
    entryPoint: path.resolve(__dirname, 'src/index.ts'),
    bundles: path.resolve(__dirname, 'dist'),
};

const config = {
    // Enables production mode built-in optimizations
    mode: 'production',
    // These are the entry point of our library. We tell webpack to use
    // the name we assign later, when creating the bundle. We also use
    // the name to filter the second entry point for applying code
    // minification via UglifyJS
    entry: {
        qviz: [PATHS.entryPoint],
        'qviz.min': [PATHS.entryPoint],
    },
    // The output defines how and where we want the bundles. The special
    // value `[name]` in `filename` tell Webpack to use the name we defined above.
    // The bundled script will be available as a global variable `qviz`.
    output: {
        path: PATHS.bundles,
        filename: '[name].js',
        library: 'qviz',
        libraryTarget: 'umd',
    },
    // Add resolve for `tsx` and `ts` files, otherwise Webpack would
    // only look for common JavaScript file extension (.js)
    resolve: {
        extensions: ['.ts', '.tsx', '.js'],
    },
    // Activate source maps for the bundles in order to preserve the original
    // source when the user debugs the application
    devtool: 'source-map',
    optimization: {
        // Apply minification only on the second bundle by
        // using a RegEx on the name, which must end with `.min.js`
        minimize: true,
        minimizer: [
            new TerserPlugin({
                sourceMap: true,
                include: /\.min\.js$/,
            }),
        ],
    },
    module: {
        // Webpack doesn't understand TypeScript files and a loader is needed.
        // `node_modules` folder is excluded in order to prevent problems with
        // the library dependencies, as well as `__tests__` folders that
        // contain the tests for the library
        rules: [
            {
                test: /\.tsx?$/,
                exclude: /node_modules/,
                use: [
                    {
                        loader: 'ts-loader',
                        options: {
                            // Speeds up compilation and does not build *.d.ts files
                            transpileOnly: true,
                        },
                    },
                ],
            },
        ],
    },
};

module.exports = config;
