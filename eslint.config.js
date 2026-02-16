import js from '@eslint/js'
import pluginVue from 'eslint-plugin-vue'
import * as parserVue from 'vue-eslint-parser'
import * as parserTypeScript from '@typescript-eslint/parser'
import pluginTypeScript from '@typescript-eslint/eslint-plugin'

export default [
  {
    ignores: [
      '**/dist/**',
      '**/node_modules/**',
      '**/src-tauri/**',
      '**/.git/**',
      '**/coverage/**'
    ]
  },
  js.configs.recommended,
  ...pluginVue.configs['flat/recommended'],
  {
    files: ['**/*.{js,mjs,cjs,jsx,ts,tsx,vue}'],
    languageOptions: {
      ecmaVersion: 'latest',
      sourceType: 'module',
      parser: parserVue,
      parserOptions: {
        parser: parserTypeScript,
        ecmaVersion: 'latest',
        sourceType: 'module'
      },
      globals: {
        process: 'readonly',
        __dirname: 'readonly',
        __filename: 'readonly',
        module: 'readonly',
        require: 'readonly'
      }
    },
    plugins: {
      '@typescript-eslint': pluginTypeScript
    },
    rules: {
      'vue/multi-word-component-names': 'off',
      '@typescript-eslint/no-unused-vars': ['error', {
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_'
      }]
    }
  }
]
