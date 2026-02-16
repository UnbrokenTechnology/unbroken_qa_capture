# Unbroken QA Capture

A desktop application built with Tauri 2, Vue 3, and Quasar Framework for quality assurance capture workflows.

## Tech Stack

- **Tauri 2.3.0** - Desktop application framework
- **Vue 3.5.18** - Progressive JavaScript framework
- **TypeScript 5.7.3** - Type-safe JavaScript
- **Quasar 2.18.6** - Vue.js component framework
- **Pinia 2.3.1** - State management
- **Vite 6.0.11** - Build tool and dev server
- **Vitest 3.0.7** - Unit testing framework
- **ESLint 9.20.0** - Linting

## Prerequisites

- Node.js 24.x or higher
- npm 11.x or higher
- Rust 1.93.x or higher
- Cargo 1.93.x or higher

### System Dependencies (Linux)

```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

## Project Structure

```
.
├── src/                    # Vue.js frontend source
│   ├── assets/            # Static assets
│   ├── components/        # Vue components
│   ├── stores/            # Pinia stores
│   ├── views/             # Page views
│   ├── App.vue            # Root component
│   ├── main.ts            # Application entry point
│   └── quasar-variables.sass  # Quasar theming
├── src-tauri/             # Tauri (Rust) backend
│   ├── src/               # Rust source code
│   ├── icons/             # Application icons
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── __tests__/             # Test files
├── public/                # Public static files
├── index.html             # HTML entry point
├── vite.config.ts         # Vite configuration
├── vitest.config.ts       # Vitest configuration
├── tsconfig.json          # TypeScript configuration
├── eslint.config.js       # ESLint configuration
└── package.json           # Node.js dependencies
```

## Getting Started

### Install Dependencies

```bash
npm install
```

### Development

Run the application in development mode with hot-reload:

```bash
npm run tauri:dev
```

Or run just the Vite dev server (for frontend-only development):

```bash
npm run dev
```

### Build

Build the application for production:

```bash
npm run tauri:build
```

The built application will be in `src-tauri/target/release`.

### Testing

Run unit tests:

```bash
npm test
```

Run tests with UI:

```bash
npm run test:ui
```

### Linting

Run ESLint:

```bash
npm run lint
```

## Available Scripts

- `npm run dev` - Start Vite dev server
- `npm run build` - Build frontend for production
- `npm run preview` - Preview production build
- `npm run tauri` - Run Tauri CLI commands
- `npm run tauri:dev` - Run app in development mode
- `npm run tauri:build` - Build app for production
- `npm test` - Run tests
- `npm run test:ui` - Run tests with UI
- `npm run lint` - Lint and fix files

## Architecture

### Frontend (Vue 3 + Quasar)

The frontend uses Vue 3's Composition API with TypeScript for type safety. Quasar provides the UI component library and styling framework. Pinia manages application state.

### Backend (Tauri + Rust)

The backend is built with Tauri, which provides a Rust runtime for native system access and a secure bridge to the frontend. Commands defined in Rust can be invoked from the Vue frontend.

### State Management

Pinia stores use the Composition API style for better TypeScript inference and a more intuitive API.

### Testing

Vitest is configured for unit testing with jsdom environment for Vue component testing. Tests are located in the `__tests__` directory.

## Development Guidelines

### Adding a New Store

Create a new file in `src/stores/`:

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useMyStore = defineStore('myStore', () => {
  const state = ref(initialValue)

  function action() {
    // logic here
  }

  return { state, action }
})
```

### Adding a Tauri Command

Add to `src-tauri/src/lib.rs`:

```rust
#[tauri::command]
fn my_command(param: String) -> Result<String, String> {
    Ok(format!("Result: {}", param))
}

// Register in invoke_handler:
.invoke_handler(tauri::generate_handler![greet, my_command])
```

Call from Vue:

```typescript
import { invoke } from '@tauri-apps/api/core'

const result = await invoke<string>('my_command', { param: 'value' })
```

## Next Steps

1. **Generate Application Icons**: Use [Tauri icon generator](https://tauri.app/v1/guides/features/icons) to create proper icon files
2. **Configure Quasar Components**: Import and configure additional Quasar plugins in `src/main.ts`
3. **Set up Routing**: Install and configure Vue Router if multi-page navigation is needed
4. **Configure Tauri Plugins**: Add additional Tauri plugins as needed (filesystem, dialog, etc.)
5. **Set up CI/CD**: Configure automated testing and building
6. **Review Tauri Security**: Configure CSP and permissions in `tauri.conf.json`

## Resources

- [Tauri Documentation](https://tauri.app)
- [Vue 3 Documentation](https://vuejs.org)
- [Quasar Documentation](https://quasar.dev)
- [Pinia Documentation](https://pinia.vuejs.org)
- [Vitest Documentation](https://vitest.dev)
