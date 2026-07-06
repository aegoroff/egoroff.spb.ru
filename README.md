[![](https://tokei.rs/b1/github/aegoroff/egoroff.spb.ru?category=code)](https://github.com/XAMPPRocky/tokei)

# egoroff.spb.ru

Personal website and blog platform built with Rust backend and Vue.js frontend.

## Overview

This is the source code for [egoroff.spb.ru](https://egoroff.spb.ru), a personal website that includes:
- Blog with posts and announcements
- Portfolio section
- Apache documentation viewer
- Admin interface for content management
- Search functionality
- IndieWeb support (Micropub, Webmention)

## Architecture

The project consists of several components:

### Backend (Rust)
- **egoroff**: Main CLI application with server and migration commands
- **kernel**: Core business logic and data models
- **server**: HTTP server with REST API and template rendering
- **migrate**: Data migration utilities

### Frontend (Vue.js)
- **ui**: Vue 3 application with TypeScript
- Modern UI with Bootstrap and FontAwesome icons
- Admin interface for content management
- Responsive design

### Documentation
- **apache**: Apache documentation in XML format with XSLT stylesheets
- **templates**: Template files for various content types

## Technologies

### Backend
- **Rust** with Tokio async runtime
- **SQLite** for data storage
- **Askama** for template rendering
- **Tower** for HTTP middleware
- **Serde** for serialization

### Frontend
- **Vue.js 3** with TypeScript
- **Bootstrap 5** for styling
- **FontAwesome** for icons
- **Axios** for HTTP requests
- **Vue Router** for navigation

### DevOps
- **Docker** for containerization
- **Python** build scripts for documentation processing
- **Bun** for frontend build process

## Prerequisites

- Rust (latest stable)
- [Bun](https://bun.sh/) (frontend package manager and build)
- Python 3 (Apache documentation build, optional)
- Docker (optional)

## Building

Production assets are embedded into the Rust binary via `rust-embed`, so **build the frontend first** — it writes CSS/JS to `static/dist/`, which the server crate picks up at compile time.

### Local Development

1. **Build the frontend:**
   ```bash
   cd ui
   bun install          # first time or after dependency changes
   bun run build        # production → ../static/dist/
   ```

   For faster iteration during admin UI work:
   ```bash
   bun run devbuild     # development build → ../static/dist/
   bun run serve        # Vue dev server (admin SPA only)
   ```

2. **Build the backend:**
   ```bash
   cd egoroff
   cargo build          # debug
   cargo build --release  # production
   ```

3. **Build Apache documentation (optional):**
   ```bash
   python3 build.py
   ```

### Docker Build

```bash
docker build -t egoroff/egoroff .
```

## Running

### Local Development

1. **Start the server:**
   ```bash
   cd egoroff
   cargo run -- server
   ```

2. **For development with migration:**
   ```bash
   cargo run --features migrating -- server
   ```

### Docker

```bash
docker run -p 4200:4200 -p 4201:4201 egoroff/egoroff server
```

## Configuration

The application uses environment variables for configuration:

- `EGOROFF_HTTP_PORT`: HTTP port (default: 4200)
- `EGOROFF_HTTPS_PORT`: HTTPS port (default: 4201)
- `EGOROFF_DATA_DIR`: Data storage directory
- `EGOROFF_HOME_DIR`: Home directory

## Features

### Blog
- Markdown support for posts
- Tags and categories
- RSS/Atom feeds
- Social sharing

### Portfolio
- Project showcase
- Download section
- Apache documentation viewer

### Admin Interface
- Post management (create, edit, delete)
- Download management
- User management
- Content preview

### IndieWeb Support
- Micropub endpoint for posting
- Webmention support
- Microformats markup

### Search
- Full-text search across posts
- Real-time search results
- Search result highlighting

## Development

### Project Structure

```
egoroff.spb.ru/
├── egoroff/           # Rust workspace
│   ├── egoroff/      # Main CLI application
│   ├── kernel/       # Core business logic
│   ├── server/       # HTTP server
│   └── migrate/      # Migration utilities
├── ui/               # Vue.js frontend
├── apache/           # Apache documentation
├── templates/        # Template files
├── static/           # Static assets
└── build.py          # Build script
```

### Available Commands

**Backend** (from `egoroff/`):

- `cargo build` — build workspace (debug)
- `cargo build --release` — production binary
- `cargo test` — run tests
- `cargo clippy --workspace --all-features -- -W clippy::pedantic` — lint
- `cargo run -- server` — start the web server
- `cargo run -- version` — show version information
- `cargo run --features migrating -- migrate` — run data migrations

**Frontend** (from `ui/`):

- `bun install` — install dependencies
- `bun run build` — production build to `static/dist/`
- `bun run devbuild` — development build to `static/dist/`
- `bun run serve` — admin SPA dev server
- `bun run lint` — ESLint check

### Code Quality

The project enforces strict Rust linting:
- No unsafe code allowed
- Comprehensive error handling
- Async/await patterns
- Type safety throughout
- Run `cargo clippy` and `bun run lint` before committing

## License

This project is licensed under the terms specified in the LICENSE.txt file.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Contact

For questions or issues, please open an issue on GitHub or contact the maintainer.

