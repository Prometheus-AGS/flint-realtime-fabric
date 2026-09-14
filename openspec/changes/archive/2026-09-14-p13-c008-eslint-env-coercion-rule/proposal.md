# p13-c008 — Add ESLint + env-var boolean coercion rule to admin-ui

## Summary

`admin-ui/` has no ESLint setup. The Phase 12 reflection identified a
JS boolean coercion trap: `!!process.env["X"]` treats the string `"false"`
as truthy, causing integration tests to silently run when they should skip.

Add ESLint with TypeScript support and a rule that flags `!!process.env[…]`
(or `!!process.env.X`) and requires `process.env["X"] === "true"` instead.
Wire the lint step into `package.json` and into Dagger Stage 7.

## Files to change / create

- `admin-ui/package.json` — add ESLint devDependencies + `lint` script
- `admin-ui/eslint.config.mjs` — new flat-config ESLint config with TypeScript
  support and the custom no-boolean-env rule
- `dagger/codegen.ts` — add `pnpm lint` step in Stage 7 after `pnpm install`

## Specification

### `admin-ui/package.json` additions

Under `devDependencies`:
```json
"eslint": "^9.0.0",
"@typescript-eslint/eslint-plugin": "^8.0.0",
"@typescript-eslint/parser": "^8.0.0"
```

Under `scripts`:
```json
"lint": "eslint e2e/ src/ --ext .ts,.tsx --max-warnings 0"
```

### `admin-ui/eslint.config.mjs`

```javascript
import tsParser from "@typescript-eslint/parser";
import tsPlugin from "@typescript-eslint/eslint-plugin";

/** Custom rule: ban !!process.env["X"] — must use === "true" */
const noBooleanEnvCoercion = {
    meta: {
        type: "problem",
        docs: {
            description: 'Require `process.env["X"] === "true"` instead of `!!process.env["X"]`',
        },
        schema: [],
        messages: {
            useTrueComparison:
                'Use `process.env["{{key}}"] === "true"` instead of `!!process.env[…]`. ' +
                '`!!process.env["X"]` treats the string "false" as truthy.',
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
                    const obj = node.argument.argument.object;
                    if (
                        obj.type === "MemberExpression" &&
                        obj.object.type === "Identifier" &&
                        obj.object.name === "process" &&
                        obj.property.type === "Identifier" &&
                        obj.property.name === "env"
                    ) {
                        const prop = node.argument.argument.property;
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
        files: ["e2e/**/*.ts", "src/**/*.{ts,tsx}"],
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
```

### `dagger/codegen.ts` Stage 7 addition

After the `pnpm install --frozen-lockfile` step in Stage 7, add:

```typescript
.withExec(["pnpm", "lint"])
```

This runs the lint check as part of the standard CI pipeline and fails
Stage 7 if any `!!process.env[…]` usage is introduced.

## Acceptance criteria

1. `pnpm lint` exits 0 with the current codebase (all env accesses already
   use `=== "true"` pattern).
2. A test file containing `!!process.env["SKIP_INTEGRATION"]` causes `pnpm lint`
   to fail with `local/no-boolean-env-coercion` error message.
3. TypeScript errors in `e2e/` and `src/` are caught by `@typescript-eslint`.
4. Dagger Stage 7 fails if lint fails (before the build step).
5. `pnpm typecheck` is unchanged and still works independently.
