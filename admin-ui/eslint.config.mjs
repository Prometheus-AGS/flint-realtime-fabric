import tsParser from "@typescript-eslint/parser";
import tsPlugin from "@typescript-eslint/eslint-plugin";

/**
 * Custom rule: ban `!!process.env["X"]` boolean coercion.
 *
 * `!!process.env["X"]` treats the string "false" as truthy, which silently
 * enables integration tests that should be skipped. Require `=== "true"`.
 */
const noBooleanEnvCoercion = {
    meta: {
        type: "problem",
        docs: {
            description:
                'Require `process.env["X"] === "true"` instead of `!!process.env["X"]`',
        },
        schema: [],
        messages: {
            useTrueComparison:
                'Use `process.env["{{key}}"] === "true"` instead of `!!process.env[…]`. ' +
                'The string "false" is truthy, so `!!process.env["X"]` does not behave as expected.',
        },
        fixable: null,
    },
    create(context) {
        return {
            UnaryExpression(node) {
                if (
                    node.operator === "!" &&
                    node.argument.type === "UnaryExpression" &&
                    node.argument.operator === "!" &&
                    node.argument.argument.type === "MemberExpression"
                ) {
                    const memberExpr = node.argument.argument;
                    const obj = memberExpr.object;
                    if (
                        obj.type === "MemberExpression" &&
                        obj.object.type === "Identifier" &&
                        obj.object.name === "process" &&
                        obj.property.type === "Identifier" &&
                        obj.property.name === "env"
                    ) {
                        const prop = memberExpr.property;
                        const key =
                            prop.type === "Literal"
                                ? String(prop.value)
                                : prop.type === "Identifier"
                                ? prop.name
                                : "X";
                        context.report({
                            node,
                            messageId: "useTrueComparison",
                            data: { key },
                        });
                    }
                }
            },
        };
    },
};

export default [
    {
        files: ["e2e/**/*.ts", "src/**/*.ts", "src/**/*.tsx"],
        languageOptions: {
            parser: tsParser,
            parserOptions: {
                ecmaVersion: "latest",
                sourceType: "module",
            },
        },
        plugins: {
            "@typescript-eslint": tsPlugin,
            local: { rules: { "no-boolean-env-coercion": noBooleanEnvCoercion } },
        },
        rules: {
            ...tsPlugin.configs.recommended.rules,
            "local/no-boolean-env-coercion": "error",
            "@typescript-eslint/no-explicit-any": "error",
        },
    },
];
