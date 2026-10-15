import antfu from '@antfu/eslint-config'
import withNuxt from './.nuxt/eslint.config.mjs'

export default withNuxt(
  antfu({
    ignores: ['src-tauri/**', '.output/**', 'dist/**'],
    rules: {
      'antfu/if-newline': 'off',
      'style/brace-style': ['error', '1tbs', { allowSingleLine: true }],
      'style/comma-dangle': ['error', 'never'],
      'vue/comma-dangle': ['error', 'never'],
      'vue/singleline-html-element-content-newline': 'off',
      // `<script setup>` functions and computeds often read values declared further down
      'ts/no-use-before-define': 'off'
    }
  })
)
