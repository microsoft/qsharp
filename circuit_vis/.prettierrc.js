module.exports = {
    semi: true,
    trailingComma: 'all',
    singleQuote: true,
    endOfLine: 'lf',
    printWidth: 120,
    tabWidth: 4,
    parser: 'typescript',
    overrides: [
        {
            files: ['*.md', '*.json', '*.yml', '*.yaml'],
            options: {
                tabWidth: 2,
            },
        },
    ],
};
